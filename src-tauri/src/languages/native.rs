//! Native (C/C++) language module — flat directory with clang/clang++ compilation.

use std::path::Path;

use super::RunConfig;
use crate::rustic_manifest::RusticManifest;

// ── Scaffold ─────────────────────────────────────────────────────────────────

/// Create a new native project: hello.c + content/ + rustic.toml.
pub fn scaffold_project(project_path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(project_path)
        .map_err(|e| format!("Failed to create project: {}", e))?;
    std::fs::create_dir_all(project_path.join("content"))
        .map_err(|e| format!("Failed to create content dir: {}", e))?;
    let manifest = crate::rustic_manifest::new_native_manifest();
    crate::rustic_manifest::write_manifest(project_path, &manifest)?;
    std::fs::write(project_path.join("hello.c"), starter_template("hello", "c"))
        .map_err(|e| format!("Failed to seed hello.c: {}", e))?;
    Ok(())
}

// ── Templates ────────────────────────────────────────────────────────────────

/// Starter template for native project files, based on language extension.
pub fn starter_template(name: &str, ext: &str) -> String {
    match ext {
        "c" => format!(
            "#include <stdio.h>\n\nint main() {{\n    printf(\"Hello from {}!\\n\");\n    return 0;\n}}\n",
            name
        ),
        "cpp" => format!(
            "#include <iostream>\n\nint main() {{\n    std::cout << \"Hello from {}!\" << std::endl;\n    return 0;\n}}\n",
            name
        ),
        _ => format!("// Hello from {}!\n", name),
    }
}

// ── Compiler resolution ──────────────────────────────────────────────────────

/// Resolve the clang compiler path via xcrun, falling back to /usr/bin/clang.
pub(crate) fn resolve_clang() -> String {
    std::process::Command::new("xcrun")
        .args(["--find", "clang"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "/usr/bin/clang".to_string())
}

/// Resolve the macOS SDK path via `xcrun --show-sdk-path`.
/// Needed because Tauri app bundles get a minimal environment where
/// clang can't find system headers (stdio.h etc.) without -isysroot.
pub(crate) fn resolve_sdk_path() -> Option<String> {
    std::process::Command::new("xcrun")
        .args(["--show-sdk-path"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ── Run ──────────────────────────────────────────────────────────────────────

/// Build a RunConfig::CompileThenRun for a native C/C++ playground.
pub fn build_run_command(
    name: &str,
    source_path: &Path,
    workspace: &Path,
    manifest: &RusticManifest,
) -> Result<RunConfig, String> {
    let (stem, ext) = super::file_validate_name(name, &["c", "cpp"])?;
    let runs_dir = workspace.join("target").join("runs");
    let binary_path = runs_dir.join(&stem);

    let (compiler, user_flags): (String, &[String]) = match ext.as_str() {
        "c" => (resolve_clang(), &manifest.build.cflags),
        "cpp" => {
            let clang = resolve_clang();
            let clangpp = clang.replace("clang", "clang++");
            (clangpp, &manifest.build.cxxflags)
        }
        _ => return Err(format!("Unsupported extension: {}", ext)),
    };

    let mut compile_args: Vec<String> = vec![
        source_path.to_str().unwrap().to_string(),
        "-o".to_string(),
        binary_path.to_str().unwrap().to_string(),
    ];
    if let Some(sdk) = resolve_sdk_path() {
        compile_args.push("-isysroot".to_string());
        compile_args.push(sdk);
    }
    compile_args.extend(user_flags.iter().cloned());

    let content_path = workspace.join(
        crate::rustic_manifest::read_manifest(workspace)
            .map(|m| m.paths.content)
            .unwrap_or_else(|| "content".to_string()),
    );

    Ok(RunConfig::CompileThenRun {
        compiler,
        compile_args,
        binary_path,
        env: vec![(
            "PLAYGROUND_CONTENT".to_string(),
            content_path.to_string_lossy().to_string(),
        )],
        cwd: workspace.to_path_buf(),
    })
}
