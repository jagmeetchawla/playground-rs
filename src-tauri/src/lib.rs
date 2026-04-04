use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

mod cargo_commands;
mod content_commands;
mod export;
pub(crate) mod languages;
mod menu;
mod playground_commands;
pub(crate) mod rustic_manifest;

// ── App state ─────────────────────────────────────────────────────────────────

/// Holds the currently active project name.  Managed via `app.manage()`.
pub(crate) struct ActiveProject(pub(crate) Mutex<String>);

/// PID of the currently running `cargo run` child process (= its PGID, since we
/// call `process_group(0)` at spawn time).  None when nothing is running.
pub(crate) struct RunningProcess(pub(crate) Mutex<Option<u32>>);

/// Stdin handle for the currently running child process.
/// Stored so the frontend can send interactive input via `send_stdin`.
/// Uses `tokio::sync::Mutex` because writes hold the lock across `.await`.
pub(crate) struct StdinHandle(pub(crate) tokio::sync::Mutex<Option<tokio::process::ChildStdin>>);

/// PID of the currently running `cargo check` background process.
/// Used to cancel a previous check when a new one starts.
pub(crate) struct CheckProcess(pub(crate) Mutex<Option<u32>>);

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Config {
    pub(crate) active_project: String,
    #[serde(default)]
    pub(crate) wizard_completed: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Settings {
    pub(crate) font_size: u32,
    pub(crate) font_family: String,
    pub(crate) tab_size: u32,
    #[serde(default = "default_cargo_path")]
    pub(crate) cargo_path: String,
    #[serde(default = "default_theme")]
    pub(crate) theme: String,
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_cargo_path() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{}/.cargo/bin/cargo", home)
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 13,
            font_family: "Menlo".to_string(),
            tab_size: 4,
            cargo_path: default_cargo_path(),
            theme: default_theme(),
        }
    }
}

// ── Storage paths ─────────────────────────────────────────────────────────────

/// Root of all projects — same path in dev and release.
pub(crate) fn projects_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Could not resolve App Support directory")
        .join("projects")
}

/// Active project directory: projects/<active_project_name>/
pub(crate) fn workspace_dir(app: &AppHandle) -> PathBuf {
    let name = app.state::<ActiveProject>().0.lock().unwrap().clone();
    projects_dir(app).join(name)
}

/// Source directory for the active project, read from rustic.toml [paths].src.
/// Falls back to src/bin for Rust projects without a manifest.
pub(crate) fn src_dir(app: &AppHandle) -> PathBuf {
    let workspace = workspace_dir(app);
    if let Some(manifest) = rustic_manifest::read_manifest(&workspace) {
        workspace.join(&manifest.paths.src)
    } else {
        workspace.join("src").join("bin")
    }
}

pub(crate) fn content_dir(app: &AppHandle) -> PathBuf {
    workspace_dir(app).join("content")
}

/// Get the active project's type from rustic.toml.
pub(crate) fn active_project_type(app: &AppHandle) -> String {
    let workspace = workspace_dir(app);
    rustic_manifest::detect_project_type(&workspace)
}

pub(crate) fn config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Could not resolve App Support directory")
        .join("config.json")
}

fn window_state_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Could not resolve App Support directory")
        .join("window-state.json")
}

pub(crate) fn settings_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Could not resolve App Support directory")
        .join("settings.json")
}

// ── Config persistence ────────────────────────────────────────────────────────

pub(crate) fn load_config(app: &AppHandle) -> Config {
    let path = config_path(app);
    if path.exists() {
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(c) = serde_json::from_str::<Config>(&s) {
                return c;
            }
        }
    }
    Config {
        active_project: "default".to_string(),
        wizard_completed: false,
    }
}

pub(crate) fn save_config(app: &AppHandle, active_project: &str) -> Result<(), String> {
    // Preserve wizard_completed from existing config
    let existing = load_config(app);
    let config = Config {
        active_project: active_project.to_string(),
        wizard_completed: existing.wizard_completed,
    };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialise config: {}", e))?;
    std::fs::write(config_path(app), json)
        .map_err(|e| format!("Failed to write config.json: {}", e))
}

// ── Project templates ─────────────────────────────────────────────────────────

pub(crate) fn project_cargo_toml(name: &str) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n# Add dependencies here — every playground can use them.\n[dependencies]\n",
        name
    )
}

/// New playground template — includes the PLAYGROUND_CONTENT hint.
pub(crate) fn playground_template(name: &str) -> String {
    format!(
        "// Files in your content folder are available via:\n// let dir = std::env::var(\"PLAYGROUND_CONTENT\").unwrap_or_default();\n\nfn main() {{\n    println!(\"Hello from {}!\");\n}}\n",
        name
    )
}

// ── Project bootstrap ─────────────────────────────────────────────────────────

/// Ensures the active project has the required directory structure.
/// Creates Cargo.toml + src/bin/hello.rs + content/ if they don't exist yet (for Rust projects).
/// Also ensures a rustic.toml manifest exists (auto-generates for legacy projects).
fn ensure_project(app: &AppHandle) -> Result<(), String> {
    let workspace = workspace_dir(app);

    // Ensure the project directory itself exists
    if !workspace.exists() {
        std::fs::create_dir_all(&workspace)
            .map_err(|e| format!("Failed to create project directory: {}", e))?;
    }

    // For new Rust projects: create Cargo.toml + src/bin BEFORE ensure_manifest,
    // so that legacy detection sees Cargo.toml and correctly picks "rust".
    let cargo_toml = workspace.join("Cargo.toml");
    let bin = workspace.join("src").join("bin");
    if !cargo_toml.exists() && !workspace.join("rustic.toml").exists() {
        // Brand new project — scaffold as Rust by default
        std::fs::create_dir_all(&bin)
            .map_err(|e| format!("Failed to create project dirs: {}", e))?;
        let project_name = app.state::<ActiveProject>().0.lock().unwrap().clone();
        std::fs::write(&cargo_toml, project_cargo_toml(&project_name))
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
        std::fs::write(bin.join("hello.rs"), playground_template("hello"))
            .map_err(|e| format!("Failed to seed hello.rs: {}", e))?;
    }

    // Ensure rustic.toml exists (auto-generates for legacy projects)
    let mut manifest = rustic_manifest::ensure_manifest(&workspace)?;

    // Repair: if Cargo.toml exists but rustic.toml was incorrectly generated as "native"
    // (e.g. from a startup race), fix it.
    if cargo_toml.exists() && manifest.project.project_type == "native" {
        manifest.project.project_type = "rust".to_string();
        rustic_manifest::write_manifest(&workspace, &manifest)?;
    }

    let content = workspace.join(&manifest.paths.content);
    if !content.exists() {
        std::fs::create_dir_all(&content)
            .map_err(|e| format!("Failed to create content dir: {}", e))?;
    }
    Ok(())
}

// ── Validation ────────────────────────────────────────────────────────────────

pub(crate) fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if name.len() > 64 {
        return Err("Name too long (max 64 chars)".into());
    }
    let valid = name.chars().enumerate().all(|(i, c)| {
        if i == 0 {
            c.is_ascii_lowercase()
        } else {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'
        }
    });
    if !valid {
        return Err(format!(
            "'{}' is not a valid name — use lowercase letters, digits, and underscores only",
            name
        ));
    }
    Ok(())
}

/// Validates a content filename: no path separators, no null bytes, not . or ..
pub(crate) fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty() {
        return Err("Filename cannot be empty".into());
    }
    if filename.contains('/') || filename.contains('\\') || filename.contains('\0') {
        return Err(format!("'{}' contains invalid characters", filename));
    }
    if filename == "." || filename == ".." {
        return Err("'.' and '..' are not valid filenames".into());
    }
    Ok(())
}

pub(crate) fn safe_content_path(filename: &str, app: &AppHandle) -> Result<PathBuf, String> {
    validate_filename(filename)?;
    Ok(content_dir(app).join(filename))
}

// ── Content file helpers ──────────────────────────────────────────────────────

pub(crate) fn is_text_file(filename: &str) -> bool {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    matches!(
        ext.as_str(),
        "txt"
            | "md"
            | "rs"
            | "toml"
            | "yaml"
            | "yml"
            | "json"
            | "xml"
            | "html"
            | "htm"
            | "css"
            | "js"
            | "ts"
            | "csv"
            | "log"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "conf"
            | "ini"
            | "env"
    )
}

#[derive(serde::Serialize, Clone)]
pub struct ContentFile {
    pub filename: String,
    pub size_bytes: u64,
    pub is_text: bool,
}

// ── Toolchain ─────────────────────────────────────────────────────────────────

pub(crate) fn cargo_path() -> String {
    let candidates = vec![
        dirs_next::home_dir()
            .map(|h| h.join(".cargo/bin/cargo"))
            .filter(|p| p.exists()),
        which_cargo(),
    ];
    if let Some(c) = candidates.into_iter().flatten().next() {
        return c.to_string_lossy().to_string();
    }
    "cargo".to_string()
}

fn which_cargo() -> Option<PathBuf> {
    std::process::Command::new("which")
        .arg("cargo")
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(PathBuf::from(s))
            }
        })
}

// ── Window state persistence ──────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct SavedTab {
    id: String,
    tab_type: String,         // "playground" | "cargo" | "content"
    filename: Option<String>, // only for content tabs
}

#[derive(serde::Serialize, serde::Deserialize)]
struct WindowState {
    sidebar_visible: bool,
    layout: String, // "bottom" | "right"
    sidebar_w: u32,
    output_h: u32,
    output_w: u32,
    open_tabs: Vec<SavedTab>,
    active_tab: Option<String>,
    window_width: u32,
    window_height: u32,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState {
            sidebar_visible: true,
            layout: "bottom".to_string(),
            sidebar_w: 220,
            output_h: 240,
            output_w: 300,
            open_tabs: vec![],
            active_tab: None,
            window_width: 1280,
            window_height: 800,
        }
    }
}

#[tauri::command]
fn get_window_state(app: AppHandle) -> WindowState {
    let path = window_state_path(&app);
    if path.exists() {
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(ws) = serde_json::from_str::<WindowState>(&s) {
                return ws;
            }
        }
    }
    WindowState::default()
}

#[tauri::command]
fn save_window_state(state: WindowState, app: AppHandle) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&state)
        .map_err(|e| format!("Failed to serialise window state: {}", e))?;
    std::fs::write(window_state_path(&app), json)
        .map_err(|e| format!("Failed to write window-state.json: {}", e))
}

// ── Settings persistence ──────────────────────────────────────────────────────

#[tauri::command]
fn get_settings(app: AppHandle) -> Settings {
    let path = settings_path(&app);
    if path.exists() {
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str::<Settings>(&s) {
                return settings;
            }
        }
    }
    Settings::default()
}

#[tauri::command]
fn save_settings(settings: Settings, app: AppHandle) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialise settings: {}", e))?;
    std::fs::write(settings_path(&app), json)
        .map_err(|e| format!("Failed to write settings.json: {}", e))
}

// ── Book seeding ─────────────────────────────────────────────────────────────

#[tauri::command]
fn seed_book(project_type: String, app: AppHandle) -> Result<Vec<String>, String> {
    let lang = languages::Lang::from_str(&project_type);
    lang.seed_book(&projects_dir(&app))
}

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Ensure App Support directory exists
            let app_data = app
                .path()
                .app_data_dir()
                .expect("Cannot resolve App Support dir");
            std::fs::create_dir_all(&app_data).expect("Cannot create App Support dir");

            // Load persisted config
            let config = load_config(app.handle());

            // Ensure projects directory exists
            let proj_dir = app_data.join("projects");
            std::fs::create_dir_all(&proj_dir).expect("Cannot create projects dir");

            // Resolve active project: use config value if its dir exists,
            // otherwise fall back to the first existing project or "default"
            let active = if proj_dir.join(&config.active_project).exists() {
                config.active_project.clone()
            } else {
                std::fs::read_dir(&proj_dir)
                    .ok()
                    .and_then(|mut d| {
                        d.find(|e| e.as_ref().map(|e| e.path().is_dir()).unwrap_or(false))
                    })
                    .and_then(|e| e.ok())
                    .and_then(|e| e.file_name().into_string().ok())
                    .unwrap_or_else(|| "default".to_string())
            };

            // Register active project in app state BEFORE calling ensure_project
            app.manage(ActiveProject(Mutex::new(active)));
            app.manage(RunningProcess(Mutex::new(None)));
            app.manage(StdinHandle(tokio::sync::Mutex::new(None)));
            app.manage(CheckProcess(Mutex::new(None)));

            // Bootstrap the active project's directory structure if needed
            ensure_project(app.handle()).expect("Failed to initialise project");

            // ── Native macOS menu ─────────────────────────────────────────────
            // Read the initial project list so the Project menu is populated.
            let initial_projects: Vec<String> = {
                let mut names: Vec<String> = std::fs::read_dir(&proj_dir)
                    .map(|d| {
                        d.filter_map(|e| e.ok())
                            .filter(|e| e.path().is_dir())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .collect()
                    })
                    .unwrap_or_default();
                names.sort();
                names
            };
            let active_name = app.state::<ActiveProject>().0.lock().unwrap().clone();
            // On startup we don't know playground count yet; frontend will call rebuild_menu shortly
            let ptype = rustic_manifest::detect_project_type(
                &projects_dir(app.handle()).join(&active_name),
            );
            let is_book = !rustic_manifest::get_project_source(
                &projects_dir(app.handle()).join(&active_name),
            )
            .is_empty();
            // Collect project sources for menu grouping
            let pdir = projects_dir(app.handle());
            let sources: std::collections::HashMap<String, String> = initial_projects
                .iter()
                .filter_map(|name| {
                    let src = rustic_manifest::get_project_source(&pdir.join(name));
                    if src.is_empty() {
                        None
                    } else {
                        Some((name.clone(), src))
                    }
                })
                .collect();
            let menu = menu::build_menu(
                app.handle(),
                &initial_projects,
                &active_name,
                usize::MAX,
                false,
                &ptype,
                is_book,
                &sources,
            )?;
            app.set_menu(menu)?;
            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            // Dynamic project switch items have the prefix "switch_project::"
            if let Some(name) = id.strip_prefix("switch_project::") {
                app.emit("menu:switch-project", name.to_string()).ok();
                return;
            }
            let event_name = match id {
                "new_project" => Some("menu:new-project"),
                "duplicate_project" => Some("menu:duplicate-project"),
                "rename_project" => Some("menu:rename-project"),
                "delete_project" => Some("menu:delete-project"),
                "new_playground" => Some("menu:new"),
                "save" => Some("menu:save"),
                "close_tab" => Some("menu:close-tab"),
                "run_playground" => Some("menu:run"),
                "stop_playground" => Some("menu:stop"),
                "menu_rename_playground" => Some("menu:rename-playground"),
                "menu_delete_playground" => Some("menu:delete-playground"),
                "copy_code" => Some("menu:copy-code"),
                "export_project" => Some("menu:export-project"),
                "show_settings" => Some("menu:settings"),
                "show_help" => Some("menu:help"),
                "show_about" => Some("menu:about"),
                "edit_cut" => Some("menu:edit-cut"),
                "edit_paste" => Some("menu:edit-paste"),
                _ => {
                    // Dynamic book events from language modules
                    let mut book_event = None;
                    for lang in languages::Lang::all() {
                        if let Some(book) = lang.book_info() {
                            if id == book.menu_id {
                                book_event = Some(book.event_name);
                                break;
                            }
                            if id == book.remove_menu_id {
                                book_event = Some(book.remove_event_name);
                                break;
                            }
                            if id == format!("open_book_{}", book.source_tag) {
                                let _ = std::process::Command::new("open").arg(book.url).spawn();
                                break;
                            }
                        }
                    }
                    book_event
                }
            };
            if let Some(name) = event_name {
                app.emit(name, ()).ok();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Project management
            playground_commands::list_projects,
            playground_commands::get_active_project,
            playground_commands::new_project,
            playground_commands::switch_project,
            playground_commands::rename_project,
            playground_commands::delete_project,
            playground_commands::remove_book,
            playground_commands::duplicate_project,
            menu::rebuild_menu,
            seed_book,
            get_window_state,
            save_window_state,
            get_settings,
            save_settings,
            // Playground management
            playground_commands::list_playgrounds,
            playground_commands::load_playground,
            playground_commands::save_playground,
            playground_commands::new_playground,
            playground_commands::rename_playground,
            playground_commands::delete_playground,
            playground_commands::duplicate_playground,
            playground_commands::run_playground,
            playground_commands::kill_playground,
            playground_commands::send_stdin,
            playground_commands::check_playground,
            playground_commands::cancel_check,
            playground_commands::workspace_path,
            playground_commands::copy_playground_to_project,
            // Cargo / toolchain
            cargo_commands::get_cargo_toml,
            cargo_commands::save_cargo_toml,
            cargo_commands::add_dependency,
            cargo_commands::remove_dependency,
            cargo_commands::get_toolchain_info,
            cargo_commands::check_toolchain,
            cargo_commands::complete_wizard,
            // Content files
            content_commands::list_content_files,
            content_commands::create_content_file,
            content_commands::read_content_file,
            content_commands::save_content_file,
            content_commands::delete_content_file,
            content_commands::rename_content_file,
            content_commands::import_content_file,
            content_commands::reveal_in_finder,
            content_commands::get_content_file_path,
            // Export
            export::export_project,
            // Manifest
            rustic_manifest::get_project_type,
            rustic_manifest::get_project_manifest,
            rustic_manifest::get_project_sources,
            rustic_manifest::get_project_readonly_map,
            rustic_manifest::get_locked_playgrounds,
            rustic_manifest::set_playground_locked,
            rustic_manifest::get_build_flags,
            rustic_manifest::save_build_flags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_name ────────────────────────────────────────────────────

    #[test]
    fn validate_name_accepts_simple_lowercase() {
        assert!(validate_name("hello").is_ok());
    }

    #[test]
    fn validate_name_accepts_underscores_and_digits() {
        assert!(validate_name("my_project_2").is_ok());
    }

    #[test]
    fn validate_name_rejects_empty() {
        let err = validate_name("").unwrap_err();
        assert!(err.contains("empty"), "got: {}", err);
    }

    #[test]
    fn validate_name_rejects_uppercase() {
        assert!(validate_name("Hello").is_err());
    }

    #[test]
    fn validate_name_rejects_leading_digit() {
        assert!(validate_name("2fast").is_err());
    }

    #[test]
    fn validate_name_rejects_leading_underscore() {
        assert!(validate_name("_hidden").is_err());
    }

    #[test]
    fn validate_name_rejects_hyphens() {
        assert!(validate_name("my-project").is_err());
    }

    #[test]
    fn validate_name_rejects_spaces() {
        assert!(validate_name("my project").is_err());
    }

    #[test]
    fn validate_name_rejects_path_traversal() {
        assert!(validate_name("../etc/passwd").is_err());
    }

    #[test]
    fn validate_name_rejects_too_long() {
        let long = "a".repeat(65);
        let err = validate_name(&long).unwrap_err();
        assert!(
            err.contains("too long") || err.contains("64"),
            "got: {}",
            err
        );
    }

    #[test]
    fn validate_name_accepts_max_length() {
        let name = "a".repeat(64);
        assert!(validate_name(&name).is_ok());
    }

    #[test]
    fn validate_name_rejects_special_chars() {
        assert!(validate_name("hello!").is_err());
        assert!(validate_name("hello@world").is_err());
        assert!(validate_name("hello.rs").is_err());
    }

    // ── validate_filename ────────────────────────────────────────────────

    #[test]
    fn validate_filename_accepts_normal_files() {
        assert!(validate_filename("readme.md").is_ok());
        assert!(validate_filename("data.csv").is_ok());
        assert!(validate_filename("My File.txt").is_ok());
    }

    #[test]
    fn validate_filename_rejects_empty() {
        assert!(validate_filename("").is_err());
    }

    #[test]
    fn validate_filename_rejects_forward_slash() {
        assert!(validate_filename("path/file.txt").is_err());
    }

    #[test]
    fn validate_filename_rejects_backslash() {
        assert!(validate_filename("path\\file.txt").is_err());
    }

    #[test]
    fn validate_filename_rejects_null_byte() {
        assert!(validate_filename("file\0.txt").is_err());
    }

    #[test]
    fn validate_filename_rejects_dot() {
        assert!(validate_filename(".").is_err());
    }

    #[test]
    fn validate_filename_rejects_dotdot() {
        assert!(validate_filename("..").is_err());
    }

    #[test]
    fn validate_filename_accepts_dotfiles() {
        assert!(validate_filename(".gitignore").is_ok());
        assert!(validate_filename("..hidden").is_ok());
    }

    // ── file_validate_name (native) ────────────────────────────────────────

    #[test]
    fn file_validate_name_accepts_valid_c() {
        let (stem, ext) = languages::file_validate_name("hello.c", &["c", "cpp"]).unwrap();
        assert_eq!(stem, "hello");
        assert_eq!(ext, "c");
    }

    #[test]
    fn file_validate_name_accepts_valid_cpp() {
        let (stem, ext) = languages::file_validate_name("vectors.cpp", &["c", "cpp"]).unwrap();
        assert_eq!(stem, "vectors");
        assert_eq!(ext, "cpp");
    }

    #[test]
    fn file_validate_name_rejects_no_extension() {
        assert!(languages::file_validate_name("hello", &["c", "cpp"]).is_err());
    }

    #[test]
    fn file_validate_name_rejects_unsupported_extension() {
        assert!(languages::file_validate_name("hello.py", &["c", "cpp"]).is_err());
        assert!(languages::file_validate_name("hello.go", &["c", "cpp"]).is_err());
        assert!(languages::file_validate_name("hello.zig", &["c", "cpp"]).is_err());
        assert!(languages::file_validate_name("hello.rs", &["c", "cpp"]).is_err());
    }

    #[test]
    fn file_validate_name_rejects_bad_stem() {
        assert!(languages::file_validate_name("Hello.c", &["c", "cpp"]).is_err());
        assert!(languages::file_validate_name("2fast.c", &["c", "cpp"]).is_err());
        assert!(languages::file_validate_name("my-file.c", &["c", "cpp"]).is_err());
    }

    #[test]
    fn file_validate_name_accepts_underscores_and_digits_in_stem() {
        let (stem, ext) = languages::file_validate_name("my_test_2.cpp", &["c", "cpp"]).unwrap();
        assert_eq!(stem, "my_test_2");
        assert_eq!(ext, "cpp");
    }

    // ── is_text_file ─────────────────────────────────────────────────────

    #[test]
    fn is_text_file_recognises_common_extensions() {
        assert!(is_text_file("readme.md"));
        assert!(is_text_file("data.json"));
        assert!(is_text_file("style.css"));
        assert!(is_text_file("app.js"));
        assert!(is_text_file("main.rs"));
        assert!(is_text_file("Cargo.toml"));
        assert!(is_text_file("config.yaml"));
        assert!(is_text_file("config.yml"));
        assert!(is_text_file("script.sh"));
        assert!(is_text_file("data.csv"));
        assert!(is_text_file("output.log"));
        assert!(is_text_file("notes.txt"));
    }

    #[test]
    fn is_text_file_rejects_binary_extensions() {
        assert!(!is_text_file("image.png"));
        assert!(!is_text_file("photo.jpg"));
        assert!(!is_text_file("archive.zip"));
        assert!(!is_text_file("program.exe"));
        assert!(!is_text_file("data.bin"));
    }

    #[test]
    fn is_text_file_rejects_no_extension() {
        assert!(!is_text_file("Makefile"));
        assert!(!is_text_file("README"));
    }

    #[test]
    fn is_text_file_case_insensitive() {
        assert!(is_text_file("README.MD"));
        assert!(is_text_file("config.JSON"));
        assert!(is_text_file("style.CSS"));
    }

    // ── project_cargo_toml ───────────────────────────────────────────────

    #[test]
    fn project_cargo_toml_contains_project_name() {
        let toml = project_cargo_toml("my_project");
        assert!(toml.contains("name = \"my_project\""));
    }

    #[test]
    fn project_cargo_toml_has_required_sections() {
        let toml = project_cargo_toml("test");
        assert!(toml.contains("[package]"));
        assert!(toml.contains("[dependencies]"));
        assert!(toml.contains("edition = \"2021\""));
    }

    #[test]
    fn project_cargo_toml_parses_as_valid_toml() {
        let toml = project_cargo_toml("valid_project");
        assert!(toml.parse::<toml_edit::DocumentMut>().is_ok());
    }

    // ── playground_template ──────────────────────────────────────────────

    #[test]
    fn playground_template_contains_fn_main() {
        let code = playground_template("test");
        assert!(code.contains("fn main()"));
    }

    #[test]
    fn playground_template_contains_name_in_println() {
        let code = playground_template("my_playground");
        assert!(code.contains("my_playground"));
    }

    #[test]
    fn playground_template_mentions_playground_content() {
        let code = playground_template("test");
        assert!(code.contains("PLAYGROUND_CONTENT"));
    }

    // ── Settings ─────────────────────────────────────────────────────────

    #[test]
    fn settings_default_values() {
        let s = Settings::default();
        assert_eq!(s.font_size, 13);
        assert_eq!(s.font_family, "Menlo");
        assert_eq!(s.tab_size, 4);
        assert_eq!(s.theme, "system");
        assert!(!s.cargo_path.is_empty());
    }

    #[test]
    fn settings_roundtrip_serde() {
        let s = Settings {
            font_size: 16,
            font_family: "Fira Code".to_string(),
            tab_size: 2,
            cargo_path: "/usr/bin/cargo".to_string(),
            theme: "dark".to_string(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s2.font_size, 16);
        assert_eq!(s2.font_family, "Fira Code");
        assert_eq!(s2.tab_size, 2);
        assert_eq!(s2.cargo_path, "/usr/bin/cargo");
        assert_eq!(s2.theme, "dark");
    }

    #[test]
    fn settings_deserialize_with_missing_theme_gets_default() {
        let json =
            r#"{"font_size":13,"font_family":"Menlo","tab_size":4,"cargo_path":"/usr/bin/cargo"}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.theme, "system");
    }

    #[test]
    fn settings_deserialize_with_missing_cargo_path_gets_default() {
        let json = r#"{"font_size":13,"font_family":"Menlo","tab_size":4}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert!(!s.cargo_path.is_empty());
        assert!(s.cargo_path.contains("cargo"));
    }

    // ── Config ───────────────────────────────────────────────────────────

    #[test]
    fn config_roundtrip_serde() {
        let c = Config {
            active_project: "my_project".to_string(),
            wizard_completed: true,
        };
        let json = serde_json::to_string(&c).unwrap();
        let c2: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(c2.active_project, "my_project");
        assert!(c2.wizard_completed);
    }

    #[test]
    fn config_wizard_completed_defaults_to_false() {
        let json = r#"{"active_project":"default"}"#;
        let c: Config = serde_json::from_str(json).unwrap();
        assert!(!c.wizard_completed);
    }

    // ── WindowState ──────────────────────────────────────────────────────

    #[test]
    fn window_state_defaults() {
        let ws = WindowState::default();
        assert!(ws.sidebar_visible);
        assert_eq!(ws.layout, "bottom");
        assert_eq!(ws.sidebar_w, 220);
        assert_eq!(ws.window_width, 1280);
        assert_eq!(ws.window_height, 800);
        assert!(ws.open_tabs.is_empty());
        assert!(ws.active_tab.is_none());
    }

    #[test]
    fn window_state_roundtrip_serde() {
        let ws = WindowState {
            sidebar_visible: false,
            layout: "right".to_string(),
            sidebar_w: 300,
            output_h: 200,
            output_w: 400,
            open_tabs: vec![SavedTab {
                id: "pg:hello".to_string(),
                tab_type: "playground".to_string(),
                filename: None,
            }],
            active_tab: Some("pg:hello".to_string()),
            window_width: 1920,
            window_height: 1080,
        };
        let json = serde_json::to_string(&ws).unwrap();
        let ws2: WindowState = serde_json::from_str(&json).unwrap();
        assert!(!ws2.sidebar_visible);
        assert_eq!(ws2.layout, "right");
        assert_eq!(ws2.open_tabs.len(), 1);
        assert_eq!(ws2.open_tabs[0].id, "pg:hello");
        assert_eq!(ws2.active_tab, Some("pg:hello".to_string()));
    }

    // ── ContentFile ──────────────────────────────────────────────────────

    #[test]
    fn content_file_serializes_correctly() {
        let cf = ContentFile {
            filename: "data.csv".to_string(),
            size_bytes: 1024,
            is_text: true,
        };
        let json = serde_json::to_string(&cf).unwrap();
        assert!(json.contains("\"filename\":\"data.csv\""));
        assert!(json.contains("\"size_bytes\":1024"));
        assert!(json.contains("\"is_text\":true"));
    }
}
