//! Rust language module — Cargo package with src/bin/ playgrounds.

use std::path::{Path, PathBuf};

use super::RunConfig;

// ── Scaffold ─────────────────────────────────────────────────────────────────

/// Create a new Rust project: Cargo.toml + src/bin/hello.rs + content/ + rustic.toml.
pub fn scaffold_project(project_path: &Path, project_name: &str) -> Result<(), String> {
    let bin = project_path.join("src").join("bin");
    std::fs::create_dir_all(&bin).map_err(|e| format!("Failed to create project: {}", e))?;
    std::fs::write(
        project_path.join("Cargo.toml"),
        crate::project_cargo_toml(project_name),
    )
    .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;
    std::fs::write(bin.join("hello.rs"), starter_template("hello"))
        .map_err(|e| format!("Failed to seed hello.rs: {}", e))?;
    std::fs::create_dir_all(project_path.join("content"))
        .map_err(|e| format!("Failed to create content dir: {}", e))?;
    let manifest = crate::rustic_manifest::new_rust_manifest();
    crate::rustic_manifest::write_manifest(project_path, &manifest)?;
    Ok(())
}

// ── Templates ────────────────────────────────────────────────────────────────

/// New playground template with PLAYGROUND_CONTENT hint.
pub fn starter_template(name: &str) -> String {
    format!(
        "// Files in your content folder are available via:\n// let dir = std::env::var(\"PLAYGROUND_CONTENT\").unwrap_or_default();\n\nfn main() {{\n    println!(\"Hello from {}!\");\n}}\n",
        name
    )
}

// ── Listing ──────────────────────────────────────────────────────────────────

/// List .rs files from src/bin, returning stem names (no extension).
pub fn list_playgrounds(bin_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(bin_dir) {
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

// ── Validation ───────────────────────────────────────────────────────────────

/// Validate a Rust playground name: stem only, no extension.
/// Returns (name, "rs") for consistency with the Lang interface.
pub fn validate_name(name: &str) -> Result<(String, String), String> {
    crate::validate_name(name)?;
    Ok((name.to_string(), "rs".to_string()))
}

/// Resolve path to a Rust playground file: bin_dir/name.rs
pub fn playground_path(name: &str, bin_dir: &Path) -> Result<PathBuf, String> {
    crate::validate_name(name)?;
    let path = bin_dir.join(format!("{}.rs", name));
    let resolved_dir = bin_dir.canonicalize().map_err(|e| e.to_string())?;
    let resolved_parent = path
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .unwrap_or_else(|| resolved_dir.clone());
    if resolved_parent != resolved_dir {
        return Err(format!("Path traversal detected for name '{}'", name));
    }
    Ok(path)
}

// ── Run ──────────────────────────────────────────────────────────────────────

/// Build a RunConfig::Direct for `cargo run --bin <name>`.
pub fn build_run_command(name: &str, workspace: &Path) -> Result<RunConfig, String> {
    crate::validate_name(name)?;
    let cargo = crate::cargo_path();
    let playground_target = workspace.join("target").join("playground-runs");
    let content_path = workspace.join(
        crate::rustic_manifest::read_manifest(workspace)
            .map(|m| m.paths.content)
            .unwrap_or_else(|| "content".to_string()),
    );

    Ok(RunConfig::Direct {
        program: cargo,
        args: vec![
            "run".to_string(),
            "--bin".to_string(),
            name.to_string(),
            "--target-dir".to_string(),
            playground_target.to_string_lossy().to_string(),
        ],
        env: vec![(
            "PLAYGROUND_CONTENT".to_string(),
            content_path.to_string_lossy().to_string(),
        )],
        cwd: workspace.to_path_buf(),
    })
}
