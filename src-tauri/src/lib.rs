use std::path::PathBuf;
use std::sync::Mutex;
use tauri::ipc::Channel;
use tauri::menu::{
    CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{AppHandle, Emitter, Manager};

mod book_chapters;

// ── App state ─────────────────────────────────────────────────────────────────

/// Holds the currently active project name.  Managed via `app.manage()`.
struct ActiveProject(Mutex<String>);

/// PID of the currently running `cargo run` child process (= its PGID, since we
/// call `process_group(0)` at spawn time).  None when nothing is running.
struct RunningProcess(Mutex<Option<u32>>);

/// Stdin handle for the currently running child process.
/// Stored so the frontend can send interactive input via `send_stdin`.
/// Uses `tokio::sync::Mutex` because writes hold the lock across `.await`.
struct StdinHandle(tokio::sync::Mutex<Option<tokio::process::ChildStdin>>);

/// PID of the currently running `cargo check` background process.
/// Used to cancel a previous check when a new one starts.
struct CheckProcess(Mutex<Option<u32>>);

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    active_project: String,
    #[serde(default)]
    wizard_completed: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct Settings {
    font_size: u32,
    font_family: String,
    tab_size: u32,
    #[serde(default = "default_cargo_path")]
    cargo_path: String,
    #[serde(default = "default_theme")]
    theme: String,
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

fn window_state_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Could not resolve App Support directory")
        .join("window-state.json")
}

fn settings_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .expect("Could not resolve App Support directory")
        .join("settings.json")
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
    Config {
        active_project: "default".to_string(),
        wizard_completed: false,
    }
}

fn save_config(app: &AppHandle, active_project: &str) -> Result<(), String> {
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
        std::fs::write(
            workspace.join("Cargo.toml"),
            project_cargo_toml(&project_name),
        )
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
    let resolved_parent = path
        .parent()
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

fn cargo_path() -> String {
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
    std::fs::create_dir_all(&bin).map_err(|e| format!("Failed to create project: {}", e))?;
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
    std::fs::remove_dir_all(&project_path).map_err(|e| format!("Failed to delete project: {}", e))
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
        if name.to_string_lossy() == "target" {
            continue;
        }
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
fn new_playground(name: String, content: Option<String>, app: AppHandle) -> Result<(), String> {
    let path = safe_playground_path(&name, &app)?;
    if path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    let code = content.unwrap_or_else(|| playground_template(&name));
    std::fs::write(&path, code).map_err(|e| e.to_string())
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
        .args([
            "run",
            "--bin",
            &name,
            "--target-dir",
            playground_target.to_str().unwrap(),
        ])
        .current_dir(&workspace)
        .env("PLAYGROUND_CONTENT", &content_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0) // own process group so kill() hits cargo + spawned binary
        .spawn()
        .map_err(|e| format!("Failed to start cargo: {}", e))?;

    // Store PID (= PGID) so kill_playground can send signals to the whole group.
    if let Some(pid) = child.id() {
        *app.state::<RunningProcess>().0.lock().unwrap() = Some(pid);
    }

    // Store stdin handle so the frontend can send interactive input.
    *app.state::<StdinHandle>().0.lock().await = child.stdin.take();

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let ch_out = on_output.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            ch_out
                .send(serde_json::json!({ "stream": "stdout", "line": line }))
                .ok();
        }
    });

    let ch_err = on_output.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            ch_err
                .send(serde_json::json!({ "stream": "stderr", "line": line }))
                .ok();
        }
    });

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let _ = tokio::join!(stdout_task, stderr_task);

    // Process finished — clear stored PID and stdin handle.
    *app.state::<RunningProcess>().0.lock().unwrap() = None;
    *app.state::<StdinHandle>().0.lock().await = None;

    on_output
        .send(serde_json::json!({
            "stream": "complete",
            "code": status.code().unwrap_or(-1)
        }))
        .ok();

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

/// Run `cargo check` in the background and stream diagnostics to the frontend.
/// Cancels any previously running check before starting a new one.
#[tauri::command]
async fn check_playground(
    name: String,
    code: String,
    on_diagnostics: Channel<serde_json::Value>,
    app: AppHandle,
) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    validate_name(&name)?;

    // Save the code first so cargo check sees the latest version
    let workspace = workspace_dir(&app);
    let file = workspace
        .join("src")
        .join("bin")
        .join(format!("{}.rs", name));
    std::fs::write(&file, &code).map_err(|e| format!("Failed to save: {}", e))?;

    // Cancel any previous check
    let prev_pid = app.state::<CheckProcess>().0.lock().unwrap().take();
    if let Some(pid) = prev_pid {
        let _ = tokio::process::Command::new("kill")
            .args(["-TERM", &format!("-{}", pid)])
            .status()
            .await;
    }

    let cargo = cargo_path();
    let check_target = workspace.join("target").join("check-runs");

    let mut child = Command::new(&cargo)
        .args([
            "check",
            "--bin",
            &name,
            "--message-format",
            "json",
            "--target-dir",
            check_target.to_str().unwrap(),
        ])
        .current_dir(&workspace)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|e| format!("Failed to start cargo check: {}", e))?;

    if let Some(pid) = child.id() {
        *app.state::<CheckProcess>().0.lock().unwrap() = Some(pid);
    }

    let stdout = child.stdout.take().unwrap();
    let mut lines = BufReader::new(stdout).lines();

    // Parse cargo's JSON output for compiler messages
    while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
            if msg.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                if let Some(message) = msg.get("message") {
                    let severity = message
                        .get("level")
                        .and_then(|l| l.as_str())
                        .unwrap_or("error");

                    let text = message
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("");

                    // Extract the primary span
                    if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                        if let Some(span) = spans
                            .iter()
                            .find(|s| s.get("is_primary").and_then(|p| p.as_bool()) == Some(true))
                        {
                            on_diagnostics
                                .send(serde_json::json!({
                                    "type": "diagnostic",
                                    "severity": severity,
                                    "message": text,
                                    "line": span.get("line_start").and_then(|n| n.as_u64()).unwrap_or(1),
                                    "col": span.get("column_start").and_then(|n| n.as_u64()).unwrap_or(1),
                                    "end_line": span.get("line_end").and_then(|n| n.as_u64()).unwrap_or(1),
                                    "end_col": span.get("column_end").and_then(|n| n.as_u64()).unwrap_or(1),
                                }))
                                .ok();
                        }
                    }
                }
            }
        }
    }

    let _ = child.wait().await;
    *app.state::<CheckProcess>().0.lock().unwrap() = None;

    on_diagnostics
        .send(serde_json::json!({ "type": "done" }))
        .ok();

    Ok(())
}

/// Cancel any in-flight `cargo check` background process.
#[tauri::command]
async fn cancel_check(app: AppHandle) -> Result<(), String> {
    let maybe_pid = app.state::<CheckProcess>().0.lock().unwrap().take();
    if let Some(pid) = maybe_pid {
        let _ = tokio::process::Command::new("kill")
            .args(["-TERM", &format!("-{}", pid)])
            .status()
            .await;
    }
    Ok(())
}

/// Send a line of input to the running playground's stdin.
#[tauri::command]
async fn send_stdin(line: String, app: AppHandle) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;

    let state = app.state::<StdinHandle>();
    let mut guard = state.0.lock().await;
    if let Some(ref mut stdin) = *guard {
        let data = format!("{}\n", line);
        stdin
            .write_all(data.as_bytes())
            .await
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;
        Ok(())
    } else {
        Err("No running process".to_string())
    }
}

// ── Cargo.toml commands ───────────────────────────────────────────────────────

#[tauri::command]
fn get_cargo_toml(app: AppHandle) -> Result<String, String> {
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_cargo_toml(content: String, app: AppHandle) -> Result<(), String> {
    // Validate TOML before saving
    content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Invalid TOML: {}", e))?;
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

/// Validate a crate name: must start with a letter, contain only [a-zA-Z0-9_-].
fn validate_crate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Crate name cannot be empty".to_string());
    }
    if !name.chars().next().unwrap().is_ascii_alphabetic() {
        return Err("Crate name must start with a letter".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "Crate name can only contain letters, digits, hyphens, and underscores".to_string(),
        );
    }
    Ok(())
}

/// Validate a version string: basic semver-ish check.
fn validate_version(version: &str) -> Result<(), String> {
    if version == "*" {
        return Ok(());
    }
    // Allow optional leading operator: ^, ~, =, >=, <=, >
    let v = version
        .trim_start_matches(|c: char| "^~=><".contains(c))
        .trim();
    if v.is_empty() {
        return Err("Version cannot be empty after operator".to_string());
    }
    // Each dot-separated part must be numeric or *
    for part in v.split('.') {
        if part != "*" && part.parse::<u64>().is_err() {
            return Err(format!("Invalid version component: '{}'", part));
        }
    }
    Ok(())
}

/// Add a dependency to the active project's Cargo.toml.
/// Accepts the current editor content so it's always in sync.
/// Returns the updated file content (format-preserving).
#[tauri::command]
fn add_dependency(
    content: String,
    name: String,
    version: String,
    app: AppHandle,
) -> Result<String, String> {
    validate_crate_name(&name)?;
    // Skip version validation for inline table specs like { version = "1", features = [...] }
    if !version.trim_start().starts_with('{') {
        validate_version(&version)?;
    }

    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Invalid Cargo.toml: {}", e))?;

    // Ensure [dependencies] table exists
    if !doc.contains_key("dependencies") {
        doc["dependencies"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    // Check if already present
    if doc["dependencies"].get(&name).is_some() {
        return Err(format!("'{}' is already in [dependencies]", name));
    }

    // If version looks like an inline table (e.g. `{ version = "1", features = ["full"] }`),
    // parse it as TOML value; otherwise treat it as a plain version string.
    if version.trim_start().starts_with('{') {
        let tmp = format!("x = {}", version);
        let tmp_doc = tmp
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| format!("Invalid dependency spec: {}", e))?;
        doc["dependencies"][&name] = tmp_doc["x"].clone();
    } else {
        doc["dependencies"][&name] = toml_edit::value(&version);
    }

    let updated = doc.to_string();
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::write(&path, &updated).map_err(|e| e.to_string())?;
    Ok(updated)
}

/// Remove a dependency from the active project's Cargo.toml.
/// Accepts the current editor content so it's always in sync.
/// Returns the updated file content.
#[tauri::command]
fn remove_dependency(content: String, name: String, app: AppHandle) -> Result<String, String> {
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Invalid Cargo.toml: {}", e))?;

    if let Some(deps) = doc.get_mut("dependencies").and_then(|d| d.as_table_mut()) {
        if deps.remove(&name).is_none() {
            return Err(format!("'{}' not found in [dependencies]", name));
        }
    } else {
        return Err("No [dependencies] table found".to_string());
    }

    let updated = doc.to_string();
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::write(&path, &updated).map_err(|e| e.to_string())?;
    Ok(updated)
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

/// Comprehensive toolchain check for the setup wizard.
/// Returns status of rustup, cargo, rustc, and installed toolchains.
#[tauri::command]
fn check_toolchain(app: AppHandle) -> serde_json::Value {
    let config = load_config(&app);

    // Check rustup
    let rustup_installed = std::process::Command::new("rustup")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let rustup_version = if rustup_installed {
        std::process::Command::new("rustup")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // Check cargo — use the user's configured path from settings
    let settings = {
        let path = settings_path(&app);
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
                .unwrap_or_default()
        } else {
            Settings::default()
        }
    };
    let cargo = if settings.cargo_path.is_empty() {
        cargo_path()
    } else {
        settings.cargo_path.clone()
    };
    let cargo_output = std::process::Command::new(&cargo)
        .arg("--version")
        .output()
        .ok();
    let cargo_installed = cargo_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let cargo_version = cargo_output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    // Check rustc
    let rustc_output = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok();
    let rustc_installed = rustc_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let rustc_version = rustc_output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    // Get active toolchain if rustup is available
    let active_toolchain = if rustup_installed {
        std::process::Command::new("rustup")
            .args(["show", "active-toolchain"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // Get installed toolchains list
    let installed_toolchains: Vec<String> = if rustup_installed {
        std::process::Command::new("rustup")
            .args(["toolchain", "list"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| {
                s.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    // Check for essential components
    let has_rustfmt = std::process::Command::new("rustfmt")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_clippy = std::process::Command::new("cargo-clippy")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let all_good = cargo_installed && rustc_installed;

    serde_json::json!({
        "wizard_completed": config.wizard_completed,
        "all_good": all_good,
        "rustup": {
            "installed": rustup_installed,
            "version": rustup_version,
        },
        "cargo": {
            "installed": cargo_installed,
            "path": cargo,
            "version": cargo_version,
        },
        "rustc": {
            "installed": rustc_installed,
            "version": rustc_version,
        },
        "active_toolchain": active_toolchain,
        "installed_toolchains": installed_toolchains,
        "components": {
            "rustfmt": has_rustfmt,
            "clippy": has_clippy,
        }
    })
}

/// Mark the toolchain wizard as completed so it doesn't show again.
#[tauri::command]
fn complete_wizard(app: AppHandle) -> Result<(), String> {
    let existing = load_config(&app);
    let config = Config {
        active_project: existing.active_project,
        wizard_completed: true,
    };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialise config: {}", e))?;
    std::fs::write(config_path(&app), json)
        .map_err(|e| format!("Failed to write config.json: {}", e))
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
            ContentFile {
                filename,
                size_bytes,
                is_text,
            }
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
fn rename_content_file(
    old_filename: String,
    new_filename: String,
    app: AppHandle,
) -> Result<(), String> {
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
    let filename = src
        .file_name()
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
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&filename);
        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
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

// ── Export ────────────────────────────────────────────────────────────────────

/// The exact main.rs from the v0.1 CLI playground runner (commit 231314e),
/// with PLAYGROUND_CONTENT env var added for content file support.
const CLI_MAIN_RS: &str = r##"use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, exit};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "playground",
    about = "Run a Rust playground file from src/bin/",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available playgrounds
    #[command(alias = "ls")]
    List,

    /// Print version information
    #[command(alias = "v")]
    Version,

    /// Run a playground by name (default when no subcommand given)
    #[command(external_subcommand)]
    Run(Vec<String>),
}

fn list_playgrounds() -> Vec<String> {
    let dir = PathBuf::from("src/bin");
    if !dir.exists() {
        return vec![];
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.extension()?.to_str()? == "rs" {
                path.file_stem()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

fn print_list() {
    let playgrounds = list_playgrounds();
    if playgrounds.is_empty() {
        println!("No playgrounds found. Add a .rs file to src/bin/.");
    } else {
        println!("Available playgrounds:\n");
        for name in &playgrounds {
            println!("  {}", name);
        }
    }
}

fn pick_playground() -> String {
    let playgrounds = list_playgrounds();

    if playgrounds.is_empty() {
        eprintln!("No playgrounds found. Add a .rs file to src/bin/.");
        exit(1);
    }

    println!("Available playgrounds:\n");
    for (i, name) in playgrounds.iter().enumerate() {
        println!("  [{}] {}", i + 1, name);
    }

    loop {
        print!("\nPick a playground: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if let Ok(n) = input.parse::<usize>() {
            if n >= 1 && n <= playgrounds.len() {
                return playgrounds[n - 1].clone();
            }
        } else if playgrounds.contains(&input.to_string()) {
            return input.to_string();
        }

        eprintln!("Invalid choice. Enter a number (1-{}) or a name.", playgrounds.len());
    }
}

fn run_playground(name: &str, args: &[String]) {
    let bin_path = PathBuf::from("src/bin").join(format!("{}.rs", name));
    if !bin_path.exists() {
        eprintln!("Error: playground `{}` not found (looked for {})", name, bin_path.display());
        exit(1);
    }

    // Set PLAYGROUND_CONTENT so playgrounds can find their content files
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let content_path = std::path::Path::new(&manifest_dir).join("content");

    let status = Command::new("cargo")
        .args(["run", "--bin", name, "--"])
        .args(args)
        .env("PLAYGROUND_CONTENT", &content_path)
        .status()
        .expect("failed to invoke cargo");

    exit(status.code().unwrap_or(0));
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let name = pick_playground();
            run_playground(&name, &[]);
        }
        Some(Commands::List) => print_list(),
        Some(Commands::Version) => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Run(args)) => {
            run_playground(&args[0], &args[1..]);
        }
    }
}
"##;

/// Export the active project as a standalone CLI playground (v0.1 CLI style).
/// Creates dest/<project>/ with merged Cargo.toml (deps + clap), src/main.rs,
/// src/bin/*.rs, and content/ files.
#[tauri::command]
fn export_project(dest: String, app: AppHandle) -> Result<String, String> {
    let workspace = workspace_dir(&app);
    let project_name = app.state::<ActiveProject>().0.lock().unwrap().clone();

    let export_dir = PathBuf::from(&dest).join(&project_name);
    let export_src = export_dir.join("src");
    let export_bin = export_src.join("bin");
    std::fs::create_dir_all(&export_bin).map_err(|e| e.to_string())?;

    // Read original Cargo.toml for edition and deps
    let orig_toml =
        std::fs::read_to_string(workspace.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let doc = orig_toml
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;

    let edition = doc
        .get("package")
        .and_then(|p| p.get("edition"))
        .and_then(|e| e.as_str())
        .unwrap_or("2021");

    // Build merged Cargo.toml: project deps + clap + default-run
    let mut export_toml = toml_edit::DocumentMut::new();
    export_toml["package"] = toml_edit::Item::Table(toml_edit::Table::new());
    export_toml["package"]["name"] = toml_edit::value(&project_name);
    export_toml["package"]["version"] = toml_edit::value("0.1.0");
    export_toml["package"]["edition"] = toml_edit::value(edition);
    export_toml["package"]["default-run"] = toml_edit::value(&project_name);

    // Merge deps: start with original, ensure clap is present
    let mut deps_table = if let Some(deps) = doc.get("dependencies") {
        deps.clone()
    } else {
        toml_edit::Item::Table(toml_edit::Table::new())
    };
    if deps_table.get("clap").is_none() {
        let mut clap_table = toml_edit::InlineTable::new();
        clap_table.insert("version", "4".into());
        let mut features = toml_edit::Array::new();
        features.push("derive");
        clap_table.insert("features", toml_edit::Value::Array(features));
        deps_table["clap"] = toml_edit::Item::Value(toml_edit::Value::InlineTable(clap_table));
    }
    export_toml["dependencies"] = deps_table;

    std::fs::write(export_dir.join("Cargo.toml"), export_toml.to_string())
        .map_err(|e| e.to_string())?;

    // Write the CLI main.rs (the v0.1 runner with clap)
    std::fs::write(export_src.join("main.rs"), CLI_MAIN_RS).map_err(|e| e.to_string())?;

    // Copy all playground files from src/bin/
    let bin_src = workspace.join("src").join("bin");
    if bin_src.exists() {
        for entry in std::fs::read_dir(&bin_src)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        {
            std::fs::copy(entry.path(), export_bin.join(entry.file_name()))
                .map_err(|e| e.to_string())?;
        }
    }

    // Copy content files if they exist
    let content_src = content_dir(&app);
    if content_src.exists() {
        let content_dest = export_dir.join("content");
        if let Ok(entries) = std::fs::read_dir(&content_src) {
            for entry in entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            {
                std::fs::create_dir_all(&content_dest).map_err(|e| e.to_string())?;
                std::fs::copy(entry.path(), content_dest.join(entry.file_name()))
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(export_dir.to_string_lossy().to_string())
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

// ── Menu builder ─────────────────────────────────────────────────────────────

/// Builds the full macOS menu bar.  Called once on startup and again via
/// `rebuild_projects_menu` whenever the project list or active project changes.
fn build_menu<R: tauri::Runtime>(
    handle: &impl tauri::Manager<R>,
    projects: &[String],
    active: &str,
    playground_count: usize,
) -> tauri::Result<tauri::menu::Menu<R>> {
    let app_submenu = SubmenuBuilder::new(handle, "Rustic Playground")
        .item(&PredefinedMenuItem::about(handle, None, None)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("show_settings", "Settings…")
                .accelerator("CmdOrCtrl+,")
                .build(handle)?,
        )
        .separator()
        .item(&PredefinedMenuItem::hide(handle, None)?)
        .item(&PredefinedMenuItem::hide_others(handle, None)?)
        .item(&PredefinedMenuItem::show_all(handle, None)?)
        .separator()
        .item(&PredefinedMenuItem::quit(handle, None)?)
        .build()?;

    // Build the dynamic per-project check items first so they outlive the builder.
    let check_items: Vec<tauri::menu::CheckMenuItem<R>> = projects
        .iter()
        .map(|name| {
            CheckMenuItemBuilder::with_id(format!("switch_project::{}", name), name.as_str())
                .checked(name.as_str() == active)
                .build(handle)
        })
        .collect::<tauri::Result<Vec<_>>>()?;

    let new_project_item = MenuItemBuilder::with_id("new_project", "New Project…")
        .accelerator("CmdOrCtrl+Shift+N")
        .build(handle)?;
    let rename_project_item =
        MenuItemBuilder::with_id("rename_project", "Rename Project…").build(handle)?;
    let delete_project_item = MenuItemBuilder::with_id("delete_project", "Delete Project…")
        .enabled(projects.len() > 1)
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
        .item(
            &MenuItemBuilder::with_id("new_playground", "New Playground")
                .accelerator("CmdOrCtrl+N")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("save", "Save")
                .accelerator("CmdOrCtrl+S")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("close_tab", "Close Tab")
                .accelerator("CmdOrCtrl+W")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("menu_delete_playground", "Delete Playground…")
                .enabled(playground_count > 0)
                .build(handle)?,
        )
        .build()?;

    let run_menu = SubmenuBuilder::new(handle, "Run")
        .item(
            &MenuItemBuilder::with_id("run_playground", "Run")
                .accelerator("CmdOrCtrl+R")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("stop_playground", "Stop")
                .accelerator("CmdOrCtrl+.")
                .build(handle)?,
        )
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

    let help_menu = SubmenuBuilder::new(handle, "Help")
        .item(
            &MenuItemBuilder::with_id("show_help", "Playground Help…")
                .accelerator("CmdOrCtrl+Shift+/")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("seed_rust_book", "Load Rust Book Examples…")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("show_about", "About Rustic Playground").build(handle)?)
        .build()?;

    MenuBuilder::new(handle)
        .item(&app_submenu)
        .item(&project_menu)
        .item(&playground_menu)
        .item(&run_menu)
        .item(&edit_menu)
        .item(&help_menu)
        .build()
}

#[tauri::command]
fn rebuild_menu(
    projects: Vec<String>,
    active: String,
    playground_count: usize,
    app: AppHandle,
) -> Result<(), String> {
    let menu = build_menu(&app, &projects, &active, playground_count).map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    Ok(())
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
            let menu = build_menu(app.handle(), &initial_projects, &active_name, usize::MAX)?;
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
                "rename_project" => Some("menu:rename-project"),
                "delete_project" => Some("menu:delete-project"),
                "new_playground" => Some("menu:new"),
                "save" => Some("menu:save"),
                "close_tab" => Some("menu:close-tab"),
                "run_playground" => Some("menu:run"),
                "stop_playground" => Some("menu:stop"),
                "menu_delete_playground" => Some("menu:delete-playground"),
                "show_settings" => Some("menu:settings"),
                "show_help" => Some("menu:help"),
                "show_about" => Some("menu:about"),
                "seed_rust_book" => Some("menu:rust-book"),
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
            rebuild_menu,
            book_chapters::seed_rust_book,
            get_window_state,
            save_window_state,
            get_settings,
            save_settings,
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
            send_stdin,
            check_playground,
            cancel_check,
            workspace_path,
            // Cargo / toolchain
            get_cargo_toml,
            save_cargo_toml,
            add_dependency,
            remove_dependency,
            get_toolchain_info,
            check_toolchain,
            complete_wizard,
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
            // Export
            export_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
