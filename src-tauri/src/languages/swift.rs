//! Swift language module — flat directory with swiftc compile+run (two-step).

use std::path::Path;

use super::clang::resolve_sdk_path;
use super::RunConfig;
use crate::rustic_manifest::RusticManifest;

// ── Scaffold ─────────────────────────────────────────────────────────────────

/// Create a new Swift project: hello.swift + content/ + rustic.toml.
pub fn scaffold_project(project_path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(project_path)
        .map_err(|e| format!("Failed to create project: {}", e))?;
    std::fs::create_dir_all(project_path.join("content"))
        .map_err(|e| format!("Failed to create content dir: {}", e))?;
    let manifest = crate::rustic_manifest::new_swift_manifest();
    crate::rustic_manifest::write_manifest(project_path, &manifest)?;
    std::fs::write(project_path.join("hello.swift"), starter_template("hello"))
        .map_err(|e| format!("Failed to seed hello.swift: {}", e))?;
    Ok(())
}

// ── Templates ────────────────────────────────────────────────────────────────

/// Starter template for a Swift playground file.
pub fn starter_template(name: &str) -> String {
    format!("import Foundation\n\nprint(\"Hello from {}!\")\n", name)
}

// ── Compiler resolution ──────────────────────────────────────────────────────

/// Resolve the swiftc compiler path. Ships with Xcode CLI tools.
pub(crate) fn resolve_swiftc() -> String {
    // Try xcrun first (canonical way on macOS)
    std::process::Command::new("xcrun")
        .args(["--find", "swiftc"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "swiftc".to_string())
}

// ── Run ──────────────────────────────────────────────────────────────────────

/// Build a RunConfig::CompileThenRun for a Swift playground.
/// swiftc file.swift -o build/name && ./build/name
pub fn build_run_command(
    name: &str,
    source_path: &Path,
    workspace: &Path,
    manifest: &RusticManifest,
) -> Result<RunConfig, String> {
    let (stem, _ext) = super::file_validate_name(name, &["swift"])?;
    let runs_dir = workspace.join("target").join("runs");
    let binary_path = runs_dir.join(&stem);

    let swiftc = resolve_swiftc();

    let mut compile_args: Vec<String> = vec![
        source_path.to_str().unwrap().to_string(),
        "-o".to_string(),
        binary_path.to_str().unwrap().to_string(),
    ];
    // Pass the SDK path so swiftc can find the standard library.
    // Without this, Xcode beta toolchains may target a macOS version
    // whose stdlib isn't bundled in the SDK yet.
    if let Some(sdk) = resolve_sdk_path() {
        compile_args.push("-sdk".to_string());
        compile_args.push(sdk);
    }
    // Append user flags
    compile_args.extend(manifest.build.swiftflags.iter().cloned());

    let content_path = workspace.join(
        crate::rustic_manifest::read_manifest(workspace)
            .map(|m| m.paths.content)
            .unwrap_or_else(|| "content".to_string()),
    );

    Ok(RunConfig::CompileThenRun {
        compiler: swiftc,
        compile_args,
        binary_path,
        env: vec![(
            "PLAYGROUND_CONTENT".to_string(),
            content_path.to_string_lossy().to_string(),
        )],
        cwd: workspace.to_path_buf(),
    })
}
