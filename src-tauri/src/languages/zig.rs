//! Zig language module — flat directory with `zig run` (single-step direct execution).

use std::path::Path;

use super::RunConfig;
use crate::rustic_manifest::RusticManifest;

// ── Scaffold ─────────────────────────────────────────────────────────────────

/// Create a new Zig project: hello.zig + content/ + rustic.toml.
pub fn scaffold_project(project_path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(project_path)
        .map_err(|e| format!("Failed to create project: {}", e))?;
    std::fs::create_dir_all(project_path.join("content"))
        .map_err(|e| format!("Failed to create content dir: {}", e))?;
    let manifest = crate::rustic_manifest::new_zig_manifest();
    crate::rustic_manifest::write_manifest(project_path, &manifest)?;
    std::fs::write(project_path.join("hello.zig"), starter_template("hello"))
        .map_err(|e| format!("Failed to seed hello.zig: {}", e))?;
    Ok(())
}

// ── Templates ────────────────────────────────────────────────────────────────

/// Starter template for a Zig playground file.
pub fn starter_template(name: &str) -> String {
    format!(
        r#"const std = @import("std");

pub fn main() void {{
    std.debug.print("Hello from {name}!\n", .{{}});
}}
"#,
        name = name
    )
}

// ── Zig path resolution ─────────────────────────────────────────────────────

/// Resolve the zig binary path. Tries `which zig` first, falls back to bare "zig".
pub(crate) fn resolve_zig() -> String {
    std::process::Command::new("which")
        .arg("zig")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "zig".to_string())
}

// ── Run ──────────────────────────────────────────────────────────────────────

/// Build a RunConfig::Direct for `zig run file.zig`.
/// Zig is single-step — no separate compile needed.
pub fn build_run_command(
    name: &str,
    source_path: &Path,
    workspace: &Path,
    manifest: &RusticManifest,
) -> Result<RunConfig, String> {
    super::file_validate_name(name, &["zig"])?;

    let zig = resolve_zig();
    let mut args = vec!["run".to_string(), source_path.to_str().unwrap().to_string()];
    // Append user flags (e.g. -O ReleaseSafe)
    args.extend(manifest.build.zigflags.iter().cloned());

    let content_path = workspace.join(
        crate::rustic_manifest::read_manifest(workspace)
            .map(|m| m.paths.content)
            .unwrap_or_else(|| "content".to_string()),
    );

    Ok(RunConfig::Direct {
        program: zig,
        args,
        env: vec![(
            "PLAYGROUND_CONTENT".to_string(),
            content_path.to_string_lossy().to_string(),
        )],
        cwd: workspace.to_path_buf(),
    })
}
