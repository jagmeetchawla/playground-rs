use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use crate::languages::{Lang, RunConfig};
use crate::rustic_manifest;
use crate::{
    active_project_type, projects_dir, save_config, src_dir, workspace_dir, ActiveProject,
    CheckProcess, RunningProcess, StdinHandle,
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
    crate::validate_name(&name)?;
    let project_path = projects_dir(&app).join(&name);
    if project_path.exists() {
        return Err(format!("Project '{}' already exists", name));
    }

    let ptype = project_type.as_deref().unwrap_or("rust");
    let lang = Lang::from_str(ptype);
    lang.scaffold_project(&project_path, &name)
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
    crate::validate_name(&new_name)?;
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
pub fn remove_book(source_tag: String, app: AppHandle) -> Result<Vec<String>, String> {
    let pdir = projects_dir(&app);
    let mut removed = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&pdir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                let source = crate::rustic_manifest::get_project_source(&path);
                if source == source_tag {
                    if let Some(name) = entry.file_name().to_str() {
                        removed.push(name.to_string());
                    }
                    let _ = std::fs::remove_dir_all(&path);
                }
            }
        }
    }
    Ok(removed)
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
    let dst = projects_dir(&app).join(&new_name);
    copy_dir_all(&src, &dst).map_err(|e| format!("Failed to duplicate project: {}", e))?;

    // Clear source and readonly so the duplicate becomes a regular user project
    if let Some(mut manifest) = crate::rustic_manifest::read_manifest(&dst) {
        if !manifest.project.source.is_empty() || manifest.project.readonly {
            manifest.project.source = String::new();
            manifest.project.readonly = false;
            crate::rustic_manifest::write_manifest(&dst, &manifest)
                .map_err(|e| format!("Failed to update manifest: {}", e))?;
        }
    }

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

/// Get the Lang for the active project.
fn active_lang(app: &AppHandle) -> Lang {
    Lang::from_str(&active_project_type(app))
}

#[tauri::command]
pub fn list_playgrounds(app: AppHandle) -> Vec<String> {
    let lang = active_lang(&app);
    let dir = src_dir(&app);
    lang.list_playgrounds(&dir)
}

/// Resolve a playground file path using the Lang dispatch.
fn resolve_playground_path(name: &str, app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let lang = active_lang(app);
    let dir = src_dir(app);
    lang.playground_path(name, &dir)
}

/// Given a playground file path, return the corresponding .saved/ snapshot path.
/// e.g. `src/bin/hello.rs` → `src/bin/.saved/hello.rs`
fn saved_path(playground_path: &std::path::Path) -> std::path::PathBuf {
    let parent = playground_path.parent().unwrap();
    let filename = playground_path.file_name().unwrap();
    parent.join(".saved").join(filename)
}

/// Ensure a .saved/ snapshot exists for a playground file.
/// Called when a tab is opened — creates the snapshot from the current file if missing.
fn ensure_snapshot(playground_path: &std::path::Path) -> Result<(), String> {
    let snap = saved_path(playground_path);
    if !snap.exists() {
        let dir = snap.parent().unwrap();
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        std::fs::copy(playground_path, &snap).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Update the .saved/ snapshot (called on explicit save).
fn update_snapshot(playground_path: &std::path::Path, content: &str) -> Result<(), String> {
    let snap = saved_path(playground_path);
    let dir = snap.parent().unwrap();
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    std::fs::write(&snap, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_playground(name: String, app: AppHandle) -> Result<String, String> {
    let path = resolve_playground_path(&name, &app)?;
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_playground(name: String, content: String, app: AppHandle) -> Result<(), String> {
    let path = resolve_playground_path(&name, &app)?;
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
    update_snapshot(&path, &content)?;
    Ok(())
}

/// Ensure a .saved/ snapshot exists for a playground (called when a tab opens).
#[tauri::command]
pub fn snapshot_playground(name: String, app: AppHandle) -> Result<(), String> {
    let path = resolve_playground_path(&name, &app)?;
    ensure_snapshot(&path)
}

/// Revert a playground to its last saved state (from .saved/ snapshot).
/// Returns the reverted code so the frontend can update the editor.
#[tauri::command]
pub fn revert_playground(name: String, app: AppHandle) -> Result<String, String> {
    let path = resolve_playground_path(&name, &app)?;
    let snap = saved_path(&path);
    if !snap.exists() {
        return Err("No saved snapshot found".to_string());
    }
    let saved_code = std::fs::read_to_string(&snap).map_err(|e| e.to_string())?;
    // Restore the source file to the saved state
    std::fs::write(&path, &saved_code).map_err(|e| e.to_string())?;
    Ok(saved_code)
}

#[tauri::command]
pub fn new_playground(name: String, content: Option<String>, app: AppHandle) -> Result<(), String> {
    let lang = active_lang(&app);
    let (stem, ext) = lang.validate_name(&name)?;
    let dir = src_dir(&app);
    let path = lang.playground_path(&name, &dir)?;
    if path.exists() {
        return Err(format!("'{}' already exists", name));
    }
    let code = content.unwrap_or_else(|| lang.starter_template(&stem, &ext));
    std::fs::write(&path, &code).map_err(|e| e.to_string())?;
    update_snapshot(&path, &code)?;
    Ok(())
}

#[tauri::command]
pub fn rename_playground(old_name: String, new_name: String, app: AppHandle) -> Result<(), String> {
    let old_path = resolve_playground_path(&old_name, &app)?;
    let new_path = resolve_playground_path(&new_name, &app)?;
    if new_path.exists() {
        return Err(format!("'{}' already exists", new_name));
    }
    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())?;
    // Rename snapshot too
    let old_snap = saved_path(&old_path);
    let new_snap = saved_path(&new_path);
    if old_snap.exists() {
        let _ = std::fs::rename(&old_snap, &new_snap);
    }
    Ok(())
}

#[tauri::command]
pub fn delete_playground(name: String, app: AppHandle) -> Result<(), String> {
    let path = resolve_playground_path(&name, &app)?;
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    // Clean up snapshot
    let snap = saved_path(&path);
    if snap.exists() {
        let _ = std::fs::remove_file(&snap);
    }
    Ok(())
}

#[tauri::command]
pub fn duplicate_playground(name: String, app: AppHandle) -> Result<String, String> {
    let lang = active_lang(&app);
    let (stem, ext) = lang.validate_name(&name)?;
    let dir = src_dir(&app);
    let src_path = lang.playground_path(&name, &dir)?;

    // Build new name: Rust uses stem only, file-based langs include extension
    let new_name = match lang {
        Lang::Rust => format!("{}_copy", stem),
        Lang::Clang | Lang::Zig | Lang::Swift => format!("{}_copy.{}", stem, ext),
    };
    let dst_path = lang.playground_path(&new_name, &dir)?;
    std::fs::copy(&src_path, &dst_path).map_err(|e| e.to_string())?;
    Ok(new_name)
}

#[tauri::command]
pub fn workspace_path(app: AppHandle) -> String {
    workspace_dir(&app).to_string_lossy().to_string()
}

// ── Run playground (generic via RunConfig) ──────────────────────────────────

#[tauri::command]
pub async fn run_playground(
    name: String,
    on_output: Channel<serde_json::Value>,
    app: AppHandle,
) -> Result<(), String> {
    let lang = active_lang(&app);
    let workspace = workspace_dir(&app);
    let dir = src_dir(&app);
    let source_path = lang.playground_path(&name, &dir)?;
    let manifest =
        rustic_manifest::ensure_manifest(&workspace).unwrap_or_else(|_| lang.new_manifest());

    let config = lang.build_run_command(&name, &source_path, &workspace, &manifest)?;

    match config {
        RunConfig::Direct {
            program,
            args,
            env,
            cwd,
        } => run_direct(program, args, env, cwd, on_output, app).await,
        RunConfig::CompileThenRun {
            compiler,
            compile_args,
            binary_path,
            env,
            cwd,
        } => {
            run_compile_then_run(
                compiler,
                compile_args,
                binary_path,
                env,
                cwd,
                on_output,
                app,
            )
            .await
        }
    }
}

/// Run a single-command playground (e.g. cargo run, zig run).
async fn run_direct(
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: std::path::PathBuf,
    on_output: Channel<serde_json::Value>,
    app: AppHandle,
) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::process::Command;

    let mut cmd = Command::new(&program);
    cmd.args(&args).current_dir(&cwd);
    for (key, val) in &env {
        cmd.env(key, val);
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", program, e))?;

    // For non-cargo direct runners (e.g. zig run), send a synthetic "Running"
    // marker so the frontend transitions from compiling → running and shows
    // the stdin input field.  Cargo already emits this line itself.
    if !program.contains("cargo") {
        on_output
            .send(serde_json::json!({ "stream": "stderr", "line": format!("     Running `{}`", program) }))
            .ok();
    }

    stream_child_output(&mut child, &on_output, &app).await
}

/// Run a two-step playground: compile, then execute the binary.
async fn run_compile_then_run(
    compiler: String,
    compile_args: Vec<String>,
    binary_path: std::path::PathBuf,
    env: Vec<(String, String)>,
    cwd: std::path::PathBuf,
    on_output: Channel<serde_json::Value>,
    app: AppHandle,
) -> Result<(), String> {
    use std::process::Stdio;
    use tokio::process::Command;

    // Ensure the output directory exists
    if let Some(parent) = binary_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create build dir: {}", e))?;
    }

    // Step 1: Compile
    let compile = Command::new(&compiler)
        .args(&compile_args)
        .current_dir(&cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|e| format!("Failed to start {}: {}", compiler, e))?;

    // Store PID for kill support during compilation
    if let Some(pid) = compile.id() {
        *app.state::<RunningProcess>().0.lock().unwrap() = Some(pid);
    }

    let output = compile
        .wait_with_output()
        .await
        .map_err(|e| e.to_string())?;

    // Stream compile errors/warnings to stderr
    let stderr_text = String::from_utf8_lossy(&output.stderr);
    for line in stderr_text.lines() {
        on_output
            .send(serde_json::json!({ "stream": "stderr", "line": line }))
            .ok();
    }

    if !output.status.success() {
        *app.state::<RunningProcess>().0.lock().unwrap() = None;
        on_output
            .send(serde_json::json!({
                "stream": "complete",
                "code": output.status.code().unwrap_or(1)
            }))
            .ok();
        return Ok(());
    }

    // Signal frontend: compilation done, binary starting.
    on_output
        .send(serde_json::json!({ "stream": "stderr", "line": format!("     Running `{}`", binary_path.display()) }))
        .ok();

    // Step 2: Run the compiled binary
    let mut cmd = Command::new(binary_path.to_str().unwrap());
    cmd.current_dir(&cwd);
    for (key, val) in &env {
        cmd.env(key, val);
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0)
        .spawn()
        .map_err(|e| format!("Failed to run compiled binary: {}", e))?;

    stream_child_output(&mut child, &on_output, &app).await
}

/// Shared logic: store PID, capture stdin, stream stdout/stderr, wait, send complete.
async fn stream_child_output(
    child: &mut tokio::process::Child,
    on_output: &Channel<serde_json::Value>,
    app: &AppHandle,
) -> Result<(), String> {
    if let Some(pid) = child.id() {
        *app.state::<RunningProcess>().0.lock().unwrap() = Some(pid);
    }
    *app.state::<StdinHandle>().0.lock().await = child.stdin.take();

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let ch_out = on_output.clone();
    let stdout_task = tokio::spawn(async move {
        stream_pipe(stdout, "stdout", ch_out).await;
    });

    let ch_err = on_output.clone();
    let stderr_task = tokio::spawn(async move {
        stream_pipe(stderr, "stderr", ch_err).await;
    });

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let _ = tokio::join!(stdout_task, stderr_task);

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

/// Read bytes from a pipe, split on newlines, and send each chunk immediately.
/// Unlike BufReader::lines(), this also flushes partial lines (prompts without
/// a trailing newline) so that `print!("name? ")` appears before stdin blocks.
async fn stream_pipe<R: tokio::io::AsyncRead + Unpin>(
    mut reader: R,
    stream: &str,
    channel: Channel<serde_json::Value>,
) {
    use tokio::io::AsyncReadExt;
    let mut buf = [0u8; 4096];
    let mut leftover = String::new();

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break, // EOF
            Ok(n) => {
                // Append new data to any leftover from the previous read
                leftover.push_str(&String::from_utf8_lossy(&buf[..n]));

                // Emit all complete lines
                while let Some(pos) = leftover.find('\n') {
                    let line = leftover[..pos].to_string();
                    // Strip \r\n → \n
                    let line = line.strip_suffix('\r').unwrap_or(&line);
                    channel
                        .send(serde_json::json!({ "stream": stream, "line": line }))
                        .ok();
                    leftover = leftover[pos + 1..].to_string();
                }

                // Flush any remaining partial line (e.g. a prompt like "name? ")
                if !leftover.is_empty() {
                    channel
                        .send(serde_json::json!({ "stream": stream, "line": &leftover }))
                        .ok();
                    leftover.clear();
                }
            }
            Err(_) => break,
        }
    }

    // Flush any final leftover (shouldn't happen after the loop, but just in case)
    if !leftover.is_empty() {
        channel
            .send(serde_json::json!({ "stream": stream, "line": &leftover }))
            .ok();
    }
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
/// Only works for languages that support live checking — returns immediately for others.
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

    let lang = active_lang(&app);

    // Only languages with live checking support proceed
    if !lang.supports_live_check() {
        on_diagnostics
            .send(serde_json::json!({ "type": "done" }))
            .ok();
        return Ok(());
    }

    crate::validate_name(&name)?;

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

    let cargo = crate::cargo_path();
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

/// Copy a playground's code into a different project.
/// Writes the file to the target project's source directory.
#[tauri::command]
pub fn copy_playground_to_project(
    code: String,
    target_project: String,
    playground_name: String,
    app: AppHandle,
) -> Result<(), String> {
    let target_dir = crate::projects_dir(&app).join(&target_project);
    if !target_dir.exists() {
        return Err(format!("Project '{}' does not exist", target_project));
    }
    let target_type = crate::rustic_manifest::detect_project_type(&target_dir);
    let target_lang = Lang::from_str(&target_type);
    let (_stem, _ext) = target_lang.validate_name(&playground_name)?;

    let src_dir = if let Some(manifest) = crate::rustic_manifest::read_manifest(&target_dir) {
        target_dir.join(&manifest.paths.src)
    } else {
        target_dir.join("src").join("bin")
    };
    let path = target_lang.playground_path(&playground_name, &src_dir)?;
    if path.exists() {
        return Err(format!(
            "'{}' already exists in project '{}'",
            playground_name, target_project
        ));
    }
    std::fs::write(&path, code).map_err(|e| e.to_string())
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
