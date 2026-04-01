use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri::ipc::Channel;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};

// ── App state ─────────────────────────────────────────────────────────────────

/// Holds the currently active project name.  Managed via `app.manage()`.
struct ActiveProject(Mutex<String>);

/// PID of the currently running `cargo run` child process (= its PGID, since we
/// call `process_group(0)` at spawn time).  None when nothing is running.
struct RunningProcess(Mutex<Option<u32>>);

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    active_project: String,
}

// ── Storage paths ─────────────────────────────────────────────────────────────

/// Root of all projects — same path in dev and release.
fn projects_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Could not resolve App Support directory")
        .join("projects")
}

/// Active project directory: projects/<active_project_name>/
fn workspace_dir(app: &AppHandle) -> PathBuf {
    let name = app.state::<ActiveProject>().0.lock().unwrap().clone();
    projects_dir(app).join(name)
}

fn bin_dir(app: &AppHandle) -> PathBuf {
    workspace_dir(app).join("src").join("bin")
}

fn content_dir(app: &AppHandle) -> PathBuf {
    workspace_dir(app).join("content")
}

fn config_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Could not resolve App Support directory")
        .join("config.json")
}

// ── Config persistence ────────────────────────────────────────────────────────

fn load_config(app: &AppHandle) -> Config {
    let path = config_path(app);
    if path.exists() {
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(c) = serde_json::from_str::<Config>(&s) {
                return c;
            }
        }
    }
    Config { active_project: "default".to_string() }
}

fn save_config(app: &AppHandle, active_project: &str) -> Result<(), String> {
    let config = Config { active_project: active_project.to_string() };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialise config: {}", e))?;
    std::fs::write(config_path(app), json)
        .map_err(|e| format!("Failed to write config.json: {}", e))
}

// ── Project templates ─────────────────────────────────────────────────────────

fn project_cargo_toml(name: &str) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n# Add dependencies here — every playground can use them.\n[dependencies]\n",
        name
    )
}

/// New playground template — includes the PLAYGROUND_CONTENT hint.
fn playground_template(name: &str) -> String {
    format!(
        "// Files in your content folder are available via:\n// let dir = std::env::var(\"PLAYGROUND_CONTENT\").unwrap_or_default();\n\nfn main() {{\n    println!(\"Hello from {}!\");\n}}\n",
        name
    )
}

// ── Project bootstrap ─────────────────────────────────────────────────────────

/// Ensures the active project has the required directory structure.
/// Creates Cargo.toml + src/bin/hello.rs + content/ if they don't exist yet.
fn ensure_project(app: &AppHandle) -> Result<(), String> {
    let workspace = workspace_dir(app);
    let bin = bin_dir(app);
    let content = content_dir(app);

    if !bin.exists() {
        std::fs::create_dir_all(&bin)
            .map_err(|e| format!("Failed to create project dirs: {}", e))?;
        let project_name = app.state::<ActiveProject>().0.lock().unwrap().clone();
        std::fs::write(workspace.join("Cargo.toml"), project_cargo_toml(&project_name))
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
        std::fs::write(bin.join("hello.rs"), playground_template("hello"))
            .map_err(|e| format!("Failed to seed hello.rs: {}", e))?;
    }
    if !content.exists() {
        std::fs::create_dir_all(&content)
            .map_err(|e| format!("Failed to create content dir: {}", e))?;
    }
    Ok(())
}

// ── Validation ────────────────────────────────────────────────────────────────

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if name.len() > 64 {
        return Err("Name too long (max 64 chars)".into());
    }
    let valid = name.chars().enumerate().all(|(i, c)| {
        if i == 0 { c.is_ascii_lowercase() }
        else { c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' }
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
fn validate_filename(filename: &str) -> Result<(), String> {
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

fn safe_playground_path(name: &str, app: &AppHandle) -> Result<PathBuf, String> {
    validate_name(name)?;
    let dir = bin_dir(app);
    let path = dir.join(format!("{}.rs", name));
    let resolved_dir = dir.canonicalize().map_err(|e| e.to_string())?;
    let resolved_parent = path.parent()
        .and_then(|p| p.canonicalize().ok())
        .unwrap_or_else(|| resolved_dir.clone());
    if resolved_parent != resolved_dir {
        return Err(format!("Path traversal detected for name '{}'", name));
    }
    Ok(path)
}

fn safe_content_path(filename: &str, app: &AppHandle) -> Result<PathBuf, String> {
    validate_filename(filename)?;
    Ok(content_dir(app).join(filename))
}

// ── Content file helpers ──────────────────────────────────────────────────────

fn is_text_file(filename: &str) -> bool {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    matches!(ext.as_str(),
        "txt" | "md" | "rs" | "toml" | "yaml" | "yml" | "json" | "xml"
        | "html" | "htm" | "css" | "js" | "ts" | "csv" | "log" | "sh"
        | "bash" | "zsh" | "fish" | "conf" | "ini" | "env"
    )
}

#[derive(serde::Serialize, Clone)]
pub struct ContentFile {
    pub filename: String,
    pub size_bytes: u64,
    pub is_text: bool,
}

// ── Toolchain ─────────────────────────────────────────────────────────────────

fn cargo_path() -> String {
    let candidates = vec![
        dirs_next::home_dir()
            .map(|h| h.join(".cargo/bin/cargo"))
            .filter(|p| p.exists()),
        which_cargo(),
    ];
    for c in candidates.into_iter().flatten() {
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
            if s.is_empty() { None } else { Some(PathBuf::from(s)) }
        })
}

// ── Project commands ──────────────────────────────────────────────────────────

#[tauri::command]
fn list_projects(app: AppHandle) -> Result<Vec<String>, String> {
    let dir = projects_dir(&app);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    Ok(names)
}

#[tauri::command]
fn get_active_project(app: AppHandle) -> String {
    app.state::<ActiveProject>().0.lock().unwrap().clone()
}

#[tauri::command]
fn new_project(name: String, app: AppHandle) -> Result<(), String> {
    validate_name(&name)?;
    let project_path = projects_dir(&app).join(&name);
    if project_path.exists() {
        return Err(format!("Project '{}' already exists", name));
    }
    let bin = project_path.join("src").join("bin");
    std::fs::create_dir_all(&bin)
        .map_err(|e| format!("Failed to create project: {}", e))?;
    std::fs::write(project_path.join("Cargo.toml"), project_cargo_toml(&name))
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
    std::fs::write(bin.join("hello.rs"), playground_template("hello"))
        .map_err(|e| format!("Failed to seed hello.rs: {}", e))?;
    std::fs::create_dir_all(project_path.join("content"))
        .map_err(|e| format!("Failed to create content dir: {}", e))?;
    Ok(())
}

#[tauri::command]
fn switch_project(name: String, app: AppHandle) -> Result<(), String> {
    let project_path = projects_dir(&app).join(&name);
    if !project_path.exists() {
        return Err(format!("Project '{}' does not exist", name));
    }
    *app.state::<ActiveProject>().0.lock().unwrap() = name.clone();
    save_config(&app, &name)
}

#[tauri::command]
fn rename_project(old_name: String, new_name: String, app: AppHandle) -> Result<(), String> {
    validate_name(&new_name)?;
    let old_path = projects_dir(&app).join(&old_name);
    let new_path = projects_dir(&app).join(&new_name);
    if !old_path.exists() {
        return Err(format!("Project '{}' does not exist", old_name));
    }
    if new_path.exists() {
        return Err(format!("Project '{}' already exists", new_name));
    }
    std::fs::rename(&old_path, &new_path)
        .map_err(|e| format!("Failed to rename project: {}", e))?;
    // If this is the active project, update in-memory state and config
    let is_active = *app.state::<ActiveProject>().0.lock().unwrap() == old_name;
    if is_active {
        *app.state::<ActiveProject>().0.lock().unwrap() = new_name.clone();
        save_config(&app, &new_name)?;
    }
    Ok(())
}

#[tauri::command]
fn delete_project(name: String, app: AppHandle) -> Result<(), String> {
    let project_path = projects_dir(&app).join(&name);
    if !project_path.exists() {
        return Err(format!("Project '{}' does not exist", name));
    }
    std::fs::remove_dir_all(&project_path)
        .map_err(|e| format!("Failed to delete project: {}", e))
}

#[tauri::command]
fn duplicate_project(name: String, app: AppHandle) -> Result<String, String> {
    let src = projects_dir(&app).join(&name);
    if !src.exists() {
        return Err(format!("Project '{}' does not exist", name));
    }
    // Find an available name
    let mut new_name = format!("{}_copy", name);
    let mut i = 2usize;
    while projects_dir(&app).join(&new_name).exists() {
        new_name = format!("{}_copy{}", name, i);
        i += 1;
    }
    copy_dir_all(&src, &projects_dir(&app).join(&new_name))
        .map_err(|e| format!("Failed to duplicate project: {}", e))?;
    Ok(new_name)
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        // Skip build artifacts
        if name.to_string_lossy() == "target" { continue; }
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(&name))?;
        } else {
            std::fs::copy(entry.path(), dst.join(&name))?;
        }
    }
    Ok(())
}

// ── Playground commands ───────────────────────────────────────────────────────

#[tauri::command]
fn list_playgrounds(app: AppHandle) -> Vec<String> {
    let dir = bin_dir(&app);
    let mut names: Vec<String> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|f| f.ends_with(".rs"))
            .map(|f| f.trim_end_matches(".rs").to_string())
            .collect(),
        Err(_) => vec![],
    };
    names.sort();
    names
}

#[tauri::command]
fn load_playground(name: String, app: AppHandle) -> Result<String, String> {
    let path = safe_playground_path(&name, &app)?;
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_playground(name: String, content: String, app: AppHandle) -> Result<(), String> {
    let path = safe_playground_path(&name, &app)?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn new_playground(name: String, app: AppHandle) -> Result<(), String> {
    let path = safe_playground_path(&name, &app)?;
    if path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    std::fs::write(&path, playground_template(&name)).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_playground(old_name: String, new_name: String, app: AppHandle) -> Result<(), String> {
    let old_path = safe_playground_path(&old_name, &app)?;
    let new_path = safe_playground_path(&new_name, &app)?;
    if new_path.exists() {
        return Err(format!("'{}' already exists", new_name));
    }
    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_playground(name: String, app: AppHandle) -> Result<(), String> {
    let path = safe_playground_path(&name, &app)?;
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn duplicate_playground(name: String, app: AppHandle) -> Result<String, String> {
    let src = safe_playground_path(&name, &app)?;
    let new_name = format!("{}_copy", name);
    let dst = safe_playground_path(&new_name, &app)?;
    std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
    Ok(new_name)
}

#[tauri::command]
fn workspace_path(app: AppHandle) -> String {
    workspace_dir(&app).to_string_lossy().to_string()
}

#[tauri::command]
async fn run_playground(
    name: String,
    on_output: Channel<serde_json::Value>,
    app: AppHandle,
) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    validate_name(&name)?;

    let cargo = cargo_path();
    let workspace = workspace_dir(&app);
    let playground_target = workspace.join("target").join("playground-runs");
    let content_path = content_dir(&app);

    let mut child = Command::new(&cargo)
        .args(["run", "--bin", &name, "--target-dir", playground_target.to_str().unwrap()])
        .current_dir(&workspace)
        .env("PLAYGROUND_CONTENT", &content_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)   // own process group so kill() hits cargo + spawned binary
        .spawn()
        .map_err(|e| format!("Failed to start cargo: {}", e))?;

    // Store PID (= PGID) so kill_playground can send signals to the whole group.
    if let Some(pid) = child.id() {
        *app.state::<RunningProcess>().0.lock().unwrap() = Some(pid);
    }

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let ch_out = on_output.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            ch_out.send(serde_json::json!({ "stream": "stdout", "line": line })).ok();
        }
    });

    let ch_err = on_output.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            ch_err.send(serde_json::json!({ "stream": "stderr", "line": line })).ok();
        }
    });

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let _ = tokio::join!(stdout_task, stderr_task);

    // Process finished — clear stored PID.
    *app.state::<RunningProcess>().0.lock().unwrap() = None;

    on_output.send(serde_json::json!({
        "stream": "complete",
        "code": status.code().unwrap_or(-1)
    })).ok();

    Ok(())
}

/// Kill the currently running cargo process (and its spawned binary) by sending
/// SIGTERM to the whole process group, waiting 300 ms, then SIGKILL.
#[tauri::command]
async fn kill_playground(app: AppHandle) -> Result<(), String> {
    let maybe_pid = app.state::<RunningProcess>().0.lock().unwrap().take();
    if let Some(pid) = maybe_pid {
        // Send SIGTERM to the entire process group (-<pgid>)
        let _ = tokio::process::Command::new("kill")
            .args(["-TERM", &format!("-{}", pid)])
            .status()
            .await;
        // Give processes 300 ms to clean up, then force-kill.
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let _ = tokio::process::Command::new("kill")
            .args(["-KILL", &format!("-{}", pid)])
            .status()
            .await;
    }
    Ok(())
}

// ── Cargo.toml commands ───────────────────────────────────────────────────────

#[tauri::command]
fn get_cargo_toml(app: AppHandle) -> Result<String, String> {
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_cargo_toml(content: String, app: AppHandle) -> Result<(), String> {
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_toolchain_info() -> serde_json::Value {
    let path = cargo_path();
    let version = std::process::Command::new(&path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "cargo (not found)".to_string())
        .trim()
        .to_string();
    serde_json::json!({ "path": path, "version": version })
}

// ── Content file commands ─────────────────────────────────────────────────────

#[tauri::command]
fn list_content_files(app: AppHandle) -> Result<Vec<ContentFile>, String> {
    let dir = content_dir(&app);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut files: Vec<ContentFile> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| {
            let filename = e.file_name().to_string_lossy().to_string();
            let size_bytes = e.metadata().map(|m| m.len()).unwrap_or(0);
            let is_text = is_text_file(&filename);
            ContentFile { filename, size_bytes, is_text }
        })
        .collect();
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(files)
}

#[tauri::command]
fn create_content_file(filename: String, app: AppHandle) -> Result<(), String> {
    let path = safe_content_path(&filename, &app)?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    if path.exists() {
        return Err(format!("'{}' already exists", filename));
    }
    std::fs::write(&path, "").map_err(|e| e.to_string())
}

#[tauri::command]
fn read_content_file(filename: String, app: AppHandle) -> Result<String, String> {
    let path = safe_content_path(&filename, &app)?;
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_content_file(filename: String, content: String, app: AppHandle) -> Result<(), String> {
    let path = safe_content_path(&filename, &app)?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_content_file(filename: String, app: AppHandle) -> Result<(), String> {
    let path = safe_content_path(&filename, &app)?;
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_content_file(old_filename: String, new_filename: String, app: AppHandle) -> Result<(), String> {
    let old_path = safe_content_path(&old_filename, &app)?;
    let new_path = safe_content_path(&new_filename, &app)?;
    if new_path.exists() {
        return Err(format!("'{}' already exists", new_filename));
    }
    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_content_file(src_path: String, app: AppHandle) -> Result<String, String> {
    let src = std::path::Path::new(&src_path);
    let filename = src.file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| "Invalid source path".to_string())?
        .to_string();
    validate_filename(&filename)?;

    let dir = content_dir(&app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Resolve name collision with _1, _2, … suffix
    let mut final_name = filename.clone();
    let mut counter = 1u32;
    while dir.join(&final_name).exists() {
        let stem = std::path::Path::new(&filename)
            .file_stem().and_then(|s| s.to_str()).unwrap_or(&filename);
        let ext = std::path::Path::new(&filename)
            .extension().and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        final_name = format!("{}_{}{}", stem, counter, ext);
        counter += 1;
    }

    let dst = dir.join(&final_name);
    std::fs::copy(src, &dst).map_err(|e| e.to_string())?;
    Ok(final_name)
}

#[tauri::command]
fn reveal_in_finder(path: String) -> Result<(), String> {
    std::process::Command::new("open")
        .args(["-R", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_content_file_path(filename: String, app: AppHandle) -> Result<String, String> {
    let path = safe_content_path(&filename, &app)?;
    Ok(path.to_string_lossy().to_string())
}

// ── Menu builder ─────────────────────────────────────────────────────────────

/// Builds the full macOS menu bar.  Called once on startup and again via
/// `rebuild_projects_menu` whenever the project list or active project changes.
fn build_menu<R: tauri::Runtime>(
    handle: &impl tauri::Manager<R>,
    projects: &[String],
    active: &str,
) -> tauri::Result<tauri::menu::Menu<R>> {
    let app_submenu = SubmenuBuilder::new(handle, "Rust Playground")
        .item(&PredefinedMenuItem::about(handle, None, None)?)
        .separator()
        .item(&PredefinedMenuItem::hide(handle, None)?)
        .item(&PredefinedMenuItem::hide_others(handle, None)?)
        .item(&PredefinedMenuItem::show_all(handle, None)?)
        .separator()
        .item(&PredefinedMenuItem::quit(handle, None)?)
        .build()?;

    // Build the dynamic per-project check items first so they outlive the builder.
    let check_items: Vec<tauri::menu::CheckMenuItem<R>> = projects.iter()
        .map(|name| {
            CheckMenuItemBuilder::with_id(
                format!("switch_project::{}", name),
                name.as_str(),
            )
            .checked(name.as_str() == active)
            .build(handle)
        })
        .collect::<tauri::Result<Vec<_>>>()?;

    let new_project_item = MenuItemBuilder::with_id("new_project", "New Project…")
        .accelerator("CmdOrCtrl+Shift+N").build(handle)?;
    let rename_project_item = MenuItemBuilder::with_id("rename_project", "Rename Project…")
        .build(handle)?;
    let delete_project_item = MenuItemBuilder::with_id("delete_project", "Delete Project…")
        .build(handle)?;

    let mut proj_builder = SubmenuBuilder::new(handle, "Project")
        .item(&new_project_item)
        .separator();
    for item in &check_items {
        proj_builder = proj_builder.item(item);
    }
    let project_menu = proj_builder
        .separator()
        .item(&rename_project_item)
        .item(&delete_project_item)
        .build()?;

    let playground_menu = SubmenuBuilder::new(handle, "Playground")
        .item(&MenuItemBuilder::with_id("new_playground", "New Playground")
            .accelerator("CmdOrCtrl+N").build(handle)?)
        .separator()
        .item(&MenuItemBuilder::with_id("save", "Save")
            .accelerator("CmdOrCtrl+S").build(handle)?)
        .separator()
        .item(&MenuItemBuilder::with_id("close_tab", "Close Tab")
            .accelerator("CmdOrCtrl+W").build(handle)?)
        .build()?;

    let run_menu = SubmenuBuilder::new(handle, "Run")
        .item(&MenuItemBuilder::with_id("run_playground", "Run")
            .accelerator("CmdOrCtrl+R").build(handle)?)
        .item(&MenuItemBuilder::with_id("stop_playground", "Stop")
            .accelerator("CmdOrCtrl+.").build(handle)?)
        .build()?;

    let edit_menu = SubmenuBuilder::new(handle, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .separator()
        .select_all()
        .build()?;

    MenuBuilder::new(handle)
        .item(&app_submenu)
        .item(&project_menu)
        .item(&playground_menu)
        .item(&run_menu)
        .item(&edit_menu)
        .build()
}

#[tauri::command]
fn rebuild_projects_menu(projects: Vec<String>, active: String, app: AppHandle) -> Result<(), String> {
    let menu = build_menu(&app, &projects, &active).map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    Ok(())
}

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Ensure App Support directory exists
            let app_data = app.path()
                .app_data_dir()
                .expect("Cannot resolve App Support dir");
            std::fs::create_dir_all(&app_data)
                .expect("Cannot create App Support dir");

            // Load persisted config
            let config = load_config(app.handle());

            // Ensure projects directory exists
            let proj_dir = app_data.join("projects");
            std::fs::create_dir_all(&proj_dir)
                .expect("Cannot create projects dir");

            // Resolve active project: use config value if its dir exists,
            // otherwise fall back to the first existing project or "default"
            let active = if proj_dir.join(&config.active_project).exists() {
                config.active_project.clone()
            } else {
                std::fs::read_dir(&proj_dir).ok()
                    .and_then(|mut d| d.find(|e| {
                        e.as_ref().map(|e| e.path().is_dir()).unwrap_or(false)
                    }))
                    .and_then(|e| e.ok())
                    .and_then(|e| e.file_name().into_string().ok())
                    .unwrap_or_else(|| "default".to_string())
            };

            // Register active project in app state BEFORE calling ensure_project
            app.manage(ActiveProject(Mutex::new(active)));
            app.manage(RunningProcess(Mutex::new(None)));

            // Bootstrap the active project's directory structure if needed
            ensure_project(app.handle())
                .expect("Failed to initialise project");

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
            let menu = build_menu(app.handle(), &initial_projects, &active_name)?;
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
                "new_project"      => Some("menu:new-project"),
                "rename_project"   => Some("menu:rename-project"),
                "delete_project"   => Some("menu:delete-project"),
                "new_playground"   => Some("menu:new"),
                "save"             => Some("menu:save"),
                "close_tab"        => Some("menu:close-tab"),
                "run_playground"   => Some("menu:run"),
                "stop_playground"  => Some("menu:stop"),
                _ => None,
            };
            if let Some(name) = event_name {
                app.emit(name, ()).ok();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Project management
            list_projects,
            get_active_project,
            new_project,
            switch_project,
            rename_project,
            delete_project,
            duplicate_project,
            rebuild_projects_menu,
            // Playground management
            list_playgrounds,
            load_playground,
            save_playground,
            new_playground,
            rename_playground,
            delete_playground,
            duplicate_playground,
            run_playground,
            kill_playground,
            workspace_path,
            // Cargo / toolchain
            get_cargo_toml,
            save_cargo_toml,
            get_toolchain_info,
            // Content files
            list_content_files,
            create_content_file,
            read_content_file,
            save_content_file,
            delete_content_file,
            rename_content_file,
            import_content_file,
            reveal_in_finder,
            get_content_file_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
