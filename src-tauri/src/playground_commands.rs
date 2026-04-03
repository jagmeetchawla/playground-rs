use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use crate::{
    bin_dir, cargo_path, content_dir, playground_template, project_cargo_toml, projects_dir,
    rustic_manifest::{self, write_manifest},
    safe_playground_path, save_config, validate_name, workspace_dir, ActiveProject, CheckProcess,
    RunningProcess, StdinHandle,
};

// ── Project commands ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_projects(app: AppHandle) -> Result<Vec<String>, String> {
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
pub fn get_active_project(app: AppHandle) -> String {
    app.state::<ActiveProject>().0.lock().unwrap().clone()
}

#[tauri::command]
pub fn new_project(
    name: String,
    project_type: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    validate_name(&name)?;
    let project_path = projects_dir(&app).join(&name);
    if project_path.exists() {
        return Err(format!("Project '{}' already exists", name));
    }

    let ptype = project_type.as_deref().unwrap_or("rust");

    match ptype {
        "native" => {
            std::fs::create_dir_all(&project_path)
                .map_err(|e| format!("Failed to create project: {}", e))?;
            std::fs::create_dir_all(project_path.join("content"))
                .map_err(|e| format!("Failed to create content dir: {}", e))?;
            let manifest = rustic_manifest::new_native_manifest();
            write_manifest(&project_path, &manifest)?;
            std::fs::write(
                project_path.join("hello.c"),
                native_starter_template("hello", "c"),
            )
            .map_err(|e| format!("Failed to seed hello.c: {}", e))?;
        }
        _ => {
            // Rust project (default)
            let bin = project_path.join("src").join("bin");
            std::fs::create_dir_all(&bin)
                .map_err(|e| format!("Failed to create project: {}", e))?;
            std::fs::write(project_path.join("Cargo.toml"), project_cargo_toml(&name))
                .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
            std::fs::write(bin.join("hello.rs"), playground_template("hello"))
                .map_err(|e| format!("Failed to seed hello.rs: {}", e))?;
            std::fs::create_dir_all(project_path.join("content"))
                .map_err(|e| format!("Failed to create content dir: {}", e))?;
            let manifest = rustic_manifest::new_rust_manifest();
            write_manifest(&project_path, &manifest)?;
        }
    }

    Ok(())
}

/// Starter template for native project files, based on language extension.
pub fn native_starter_template(name: &str, ext: &str) -> String {
    match ext {
        "c" => format!(
            "#include <stdio.h>\n\nint main() {{\n    printf(\"Hello from {}!\\n\");\n    return 0;\n}}\n",
            name
        ),
        "cpp" => format!(
            "#include <iostream>\n\nint main() {{\n    std::cout << \"Hello from {}!\" << std::endl;\n    return 0;\n}}\n",
            name
        ),
        "zig" => format!(
            "const std = @import(\"std\");\n\npub fn main() !void {{\n    const stdout = std.io.getStdOut().writer();\n    try stdout.print(\"Hello from {}!\\n\", .{{}});\n}}\n",
            name
        ),
        "rs" => format!(
            "fn main() {{\n    println!(\"Hello from {}!\");\n}}\n",
            name
        ),
        _ => format!("// Hello from {}!\n", name),
    }
}

#[tauri::command]
pub fn switch_project(name: String, app: AppHandle) -> Result<(), String> {
    let project_path = projects_dir(&app).join(&name);
    if !project_path.exists() {
        return Err(format!("Project '{}' does not exist", name));
    }
    *app.state::<ActiveProject>().0.lock().unwrap() = name.clone();
    save_config(&app, &name)
}

#[tauri::command]
pub fn rename_project(old_name: String, new_name: String, app: AppHandle) -> Result<(), String> {
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
pub fn delete_project(name: String, app: AppHandle) -> Result<(), String> {
    let project_path = projects_dir(&app).join(&name);
    if !project_path.exists() {
        return Err(format!("Project '{}' does not exist", name));
    }
    std::fs::remove_dir_all(&project_path).map_err(|e| format!("Failed to delete project: {}", e))
}

#[tauri::command]
pub fn duplicate_project(name: String, app: AppHandle) -> Result<String, String> {
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
pub fn list_playgrounds(app: AppHandle) -> Vec<String> {
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
pub fn load_playground(name: String, app: AppHandle) -> Result<String, String> {
    let path = safe_playground_path(&name, &app)?;
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_playground(name: String, content: String, app: AppHandle) -> Result<(), String> {
    let path = safe_playground_path(&name, &app)?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn new_playground(name: String, content: Option<String>, app: AppHandle) -> Result<(), String> {
    let path = safe_playground_path(&name, &app)?;
    if path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    let code = content.unwrap_or_else(|| playground_template(&name));
    std::fs::write(&path, code).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_playground(old_name: String, new_name: String, app: AppHandle) -> Result<(), String> {
    let old_path = safe_playground_path(&old_name, &app)?;
    let new_path = safe_playground_path(&new_name, &app)?;
    if new_path.exists() {
        return Err(format!("'{}' already exists", new_name));
    }
    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_playground(name: String, app: AppHandle) -> Result<(), String> {
    let path = safe_playground_path(&name, &app)?;
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn duplicate_playground(name: String, app: AppHandle) -> Result<String, String> {
    let src = safe_playground_path(&name, &app)?;
    let new_name = format!("{}_copy", name);
    let dst = safe_playground_path(&new_name, &app)?;
    std::fs::copy(&src, &dst).map_err(|e| e.to_string())?;
    Ok(new_name)
}

#[tauri::command]
pub fn workspace_path(app: AppHandle) -> String {
    workspace_dir(&app).to_string_lossy().to_string()
}

#[tauri::command]
pub async fn run_playground(
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
pub async fn kill_playground(app: AppHandle) -> Result<(), String> {
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
pub async fn check_playground(
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
pub async fn cancel_check(app: AppHandle) -> Result<(), String> {
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
pub async fn send_stdin(line: String, app: AppHandle) -> Result<(), String> {
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
