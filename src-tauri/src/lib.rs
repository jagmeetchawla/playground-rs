use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri::ipc::Channel;

/// Returns the project root (where src/bin/ lives).
/// In dev: current working directory (cargo tauri dev runs from project root).
/// In production: resolved relative to app resources.
fn project_root(app: &AppHandle) -> PathBuf {
    // In dev mode, CARGO_MANIFEST_DIR points to src-tauri/ — go up one level
    if cfg!(debug_assertions) {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest.parent().unwrap().to_path_buf()
    } else {
        app.path().resource_dir().unwrap()
    }
}

fn bin_dir(app: &AppHandle) -> PathBuf {
    project_root(app).join("src").join("bin")
}

fn cargo_path() -> String {
    // Check common locations in order
    let candidates = vec![
        dirs_next::home_dir()
            .map(|h| h.join(".cargo/bin/cargo"))
            .filter(|p| p.exists()),
        which_cargo(),
    ];
    for c in candidates.into_iter().flatten() {
        return c.to_string_lossy().to_string();
    }
    "cargo".to_string() // fallback to PATH
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

// ── Commands ─────────────────────────────────────────────────────────────────

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
    let path = bin_dir(&app).join(format!("{}.rs", name));
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_playground(name: String, content: String, app: AppHandle) -> Result<(), String> {
    let path = bin_dir(&app).join(format!("{}.rs", name));
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn new_playground(name: String, app: AppHandle) -> Result<(), String> {
    let path = bin_dir(&app).join(format!("{}.rs", name));
    if path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    let template = format!(
        "fn main() {{\n    println!(\"Hello from {}!\");\n}}\n",
        name
    );
    std::fs::write(&path, template).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_playground(old_name: String, new_name: String, app: AppHandle) -> Result<(), String> {
    let old_path = bin_dir(&app).join(format!("{}.rs", old_name));
    let new_path = bin_dir(&app).join(format!("{}.rs", new_name));
    if new_path.exists() {
        return Err(format!("'{}' already exists", new_name));
    }
    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_playground(name: String, app: AppHandle) -> Result<(), String> {
    let path = bin_dir(&app).join(format!("{}.rs", name));
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn duplicate_playground(name: String, app: AppHandle) -> Result<String, String> {
    let src = bin_dir(&app).join(format!("{}.rs", name));
    let new_name = format!("{}_copy", name);
    let dst = bin_dir(&app).join(format!("{}.rs", new_name));
    std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
    Ok(new_name)
}

#[tauri::command]
async fn run_playground(name: String, on_output: Channel<serde_json::Value>, app: AppHandle) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    let cargo = cargo_path();
    let root = project_root(&app);

    // Confirm channel works and show what paths are being used
    on_output.send(serde_json::json!({ "stream": "info", "line": format!("cargo: {}", cargo) })).ok();
    on_output.send(serde_json::json!({ "stream": "info", "line": format!("root:  {:?}", root) })).ok();

    // Use a separate target dir so playground builds never block the
    // cargo lock held by `cargo tauri dev` on the main target/ directory.
    let playground_target = root.join("target").join("playground-runs");

    let mut child = Command::new(&cargo)
        .args(["run", "--bin", &name, "--target-dir", playground_target.to_str().unwrap()])
        .current_dir(&root)
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

    on_output.send(serde_json::json!({ "stream": "complete", "code": status.code().unwrap_or(-1) })).ok();
    Ok(())
}

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            list_playgrounds,
            load_playground,
            save_playground,
            new_playground,
            rename_playground,
            delete_playground,
            duplicate_playground,
            run_playground,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
