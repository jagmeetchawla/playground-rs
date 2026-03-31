use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri::ipc::Channel;

// ── Storage ───────────────────────────────────────────────────────────────────

const DEFAULT_CARGO_TOML: &str = r#"[package]
name = "playgrounds"
version = "0.1.0"
edition = "2021"

# Add dependencies here — every playground can use them.
[dependencies]
"#;

const DEFAULT_HELLO: &str = r#"fn main() {
    println!("Hello, world!");
}
"#;

/// New playground template — includes the PLAYGROUND_CONTENT hint.
fn playground_template(name: &str) -> String {
    format!(
        "// Files in your content folder are available via:\n// let dir = std::env::var(\"PLAYGROUND_CONTENT\").unwrap_or_default();\n\nfn main() {{\n    println!(\"Hello from {}!\");\n}}\n",
        name
    )
}

fn workspace_dir(app: &AppHandle) -> PathBuf {
    if cfg!(debug_assertions) {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest.parent().unwrap().to_path_buf()
    } else {
        app.path()
            .app_data_dir()
            .expect("Could not resolve App Support directory")
            .join("workspace")
    }
}

fn bin_dir(app: &AppHandle) -> PathBuf {
    workspace_dir(app).join("src").join("bin")
}

fn content_dir(app: &AppHandle) -> PathBuf {
    workspace_dir(app).join("content")
}

fn ensure_workspace(app: &AppHandle) -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Ok(());
    }
    let workspace = workspace_dir(app);
    let bin = bin_dir(app);
    if !bin.exists() {
        std::fs::create_dir_all(&bin)
            .map_err(|e| format!("Failed to create workspace: {}", e))?;
        std::fs::write(workspace.join("Cargo.toml"), DEFAULT_CARGO_TOML)
            .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
        std::fs::write(bin.join("hello.rs"), DEFAULT_HELLO)
            .map_err(|e| format!("Failed to seed hello.rs: {}", e))?;
    }
    Ok(())
}

// ── Name / filename validation ────────────────────────────────────────────────

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Playground name cannot be empty".into());
    }
    if name.len() > 64 {
        return Err("Playground name too long (max 64 chars)".into());
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

/// Returns the content file path, validating the filename to block traversal.
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

// ── Playground commands ───────────────────────────────────────────────────────

#[tauri::command]
fn list_playgrounds(app: AppHandle) -> Vec<String> {
    let dir = bin_dir(&app);
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("Cannot read {:?}", dir))
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|f| f.ends_with(".rs"))
        .map(|f| f.trim_end_matches(".rs").to_string())
        .collect();
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
    // Note: content folder is intentionally NOT deleted — v1.4 limitation
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
        // Inject content folder path — readable by the running binary
        .env("PLAYGROUND_CONTENT", &content_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start cargo: {}", e))?;

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

    on_output.send(serde_json::json!({
        "stream": "complete",
        "code": status.code().unwrap_or(-1)
    })).ok();

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

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            ensure_workspace(app.handle())
                .expect("Failed to initialise playground workspace");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_playgrounds,
            load_playground,
            save_playground,
            new_playground,
            rename_playground,
            delete_playground,
            duplicate_playground,
            run_playground,
            workspace_path,
            get_cargo_toml,
            save_cargo_toml,
            get_toolchain_info,
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
