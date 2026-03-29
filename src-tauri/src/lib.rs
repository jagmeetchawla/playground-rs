use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri::ipc::Channel;

// ── Storage ───────────────────────────────────────────────────────────────────

/// Default Cargo.toml written into a fresh workspace.
const DEFAULT_CARGO_TOML: &str = r#"[package]
name = "playgrounds"
version = "0.1.0"
edition = "2021"

# Add dependencies here — every playground can use them.
[dependencies]
"#;

/// Seeded hello.rs written on first launch.
const DEFAULT_HELLO: &str = r#"fn main() {
    println!("Hello, world!");
}
"#;

/// Returns the Cargo workspace root used by the app.
///
/// Dev mode  → project source tree (so `cargo tauri dev` uses src/bin/ as-is)
/// Production → ~/Library/Application Support/com.playground-rs.app/workspace/
///              Written by the app, never inside the read-only .app bundle.
fn workspace_dir(app: &AppHandle) -> PathBuf {
    if cfg!(debug_assertions) {
        // CARGO_MANIFEST_DIR = src-tauri/ at compile time → parent = project root
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

/// Ensures the production workspace exists on first launch.
/// Creates the directory structure, Cargo.toml, and seeds hello.rs.
/// No-op if the workspace already exists.
fn ensure_workspace(app: &AppHandle) -> Result<(), String> {
    if cfg!(debug_assertions) {
        return Ok(()); // dev mode: project tree already exists
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

// ── Name validation ───────────────────────────────────────────────────────────

/// Validates a playground name is a safe Rust identifier: [a-z][a-z0-9_]*
/// Rejects anything that could be used for path traversal (.., /, \, etc.)
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

/// Resolves and verifies the playground path stays inside bin_dir.
/// Two-layer defence: name whitelist + canonicalized path check.
fn safe_playground_path(name: &str, app: &AppHandle) -> Result<PathBuf, String> {
    validate_name(name)?;
    let dir = bin_dir(app);
    let path = dir.join(format!("{}.rs", name));
    // Canonicalize to catch symlink-based escapes
    let resolved_dir = dir.canonicalize().map_err(|e| e.to_string())?;
    let resolved_parent = path.parent()
        .and_then(|p| p.canonicalize().ok())
        .unwrap_or_else(|| resolved_dir.clone());
    if resolved_parent != resolved_dir {
        return Err(format!("Path traversal detected for name '{}'", name));
    }
    Ok(path)
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

// ── Commands ──────────────────────────────────────────────────────────────────

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
    let template = format!(
        "fn main() {{\n    println!(\"Hello from {}!\");\n}}\n",
        name
    );
    std::fs::write(&path, template).map_err(|e| e.to_string())
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

    // Separate target dir — never conflicts with cargo tauri dev's lock on target/
    let playground_target = workspace.join("target").join("playground-runs");

    let mut child = Command::new(&cargo)
        .args(["run", "--bin", &name, "--target-dir", playground_target.to_str().unwrap()])
        .current_dir(&workspace)
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

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialise the production workspace on first launch.
            // Creates ~/Library/Application Support/.../workspace/ with
            // Cargo.toml and a seeded hello.rs if it doesn't exist yet.
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
