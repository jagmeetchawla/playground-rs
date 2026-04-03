//! Per-language modules for Rustic Playground.
//!
//! Each supported language has its own module implementing language-specific
//! behavior (scaffold, run, export, toolchain detection). The `Lang` enum
//! dispatches to the right module via exhaustive match — the compiler ensures
//! every arm is handled when a new language is added.

pub mod rust;

use std::path::{Path, PathBuf};

use crate::rustic_manifest::RusticManifest;

// ── Lang enum ────────────────────────────────────────────────────────────────

/// All supported project types. Adding a variant here will cause compile errors
/// at every `match` site until the new language is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Native,
    // Zig,   — Phase 3
    // Swift, — Phase 4
}

impl Lang {
    pub fn from_str(s: &str) -> Self {
        match s {
            "rust" => Lang::Rust,
            "native" => Lang::Native,
            _ => Lang::Rust, // fallback
        }
    }

    pub fn project_type(&self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Native => "native",
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Lang::Rust => &["rs"],
            Lang::Native => &["c", "cpp"],
        }
    }

    /// Where source files live relative to project root.
    pub fn src_dir(&self) -> &'static str {
        match self {
            Lang::Rust => "src/bin",
            Lang::Native => ".",
        }
    }

    /// Whether this language supports live error checking.
    pub fn supports_live_check(&self) -> bool {
        match self {
            Lang::Rust => true,
            Lang::Native => false,
        }
    }

    /// List playground names from the source directory.
    pub fn list_playgrounds(&self, src_dir: &Path) -> Vec<String> {
        match self {
            Lang::Rust => rust::list_playgrounds(src_dir),
            Lang::Native => file_list_playgrounds(src_dir, self.extensions()),
        }
    }

    /// Validate a playground name. Returns (stem, extension).
    /// For Rust: stem = name, extension = "rs".
    /// For file-based languages: parsed from "name.ext".
    pub fn validate_name(&self, name: &str) -> Result<(String, String), String> {
        match self {
            Lang::Rust => rust::validate_name(name),
            Lang::Native => file_validate_name(name, self.extensions()),
        }
    }

    /// Resolve full path to a playground file within the given source directory.
    pub fn playground_path(&self, name: &str, src_dir: &Path) -> Result<PathBuf, String> {
        match self {
            Lang::Rust => rust::playground_path(name, src_dir),
            Lang::Native => file_playground_path(name, src_dir, self.extensions()),
        }
    }

    /// Create a new project's directory structure and seed files.
    pub fn scaffold_project(&self, project_path: &Path, project_name: &str) -> Result<(), String> {
        match self {
            Lang::Rust => rust::scaffold_project(project_path, project_name),
            Lang::Native => {
                // Inline for now — will move to native.rs in Phase 2
                std::fs::create_dir_all(project_path)
                    .map_err(|e| format!("Failed to create project: {}", e))?;
                std::fs::create_dir_all(project_path.join("content"))
                    .map_err(|e| format!("Failed to create content dir: {}", e))?;
                let manifest = crate::rustic_manifest::new_native_manifest();
                crate::rustic_manifest::write_manifest(project_path, &manifest)?;
                std::fs::write(
                    project_path.join("hello.c"),
                    native_starter_template("hello", "c"),
                )
                .map_err(|e| format!("Failed to seed hello.c: {}", e))?;
                Ok(())
            }
        }
    }

    /// Generate starter code for a new playground file.
    pub fn starter_template(&self, name: &str, ext: &str) -> String {
        match self {
            Lang::Rust => rust::starter_template(name),
            Lang::Native => native_starter_template(name, ext),
        }
    }

    /// Generate a new manifest for this language type.
    pub fn new_manifest(&self) -> RusticManifest {
        match self {
            Lang::Rust => crate::rustic_manifest::new_rust_manifest(),
            Lang::Native => crate::rustic_manifest::new_native_manifest(),
        }
    }

    /// Build the run configuration for a playground.
    pub fn build_run_command(
        &self,
        name: &str,
        source_path: &Path,
        workspace: &Path,
        manifest: &RusticManifest,
    ) -> Result<RunConfig, String> {
        match self {
            Lang::Rust => rust::build_run_command(name, workspace),
            Lang::Native => native_build_run_command(name, source_path, workspace, manifest),
        }
    }
}

// ── RunConfig ────────────────────────────────────────────────────────────────

/// How to run a playground. Generic runners in playground_commands.rs handle
/// both variants with shared output streaming.
pub enum RunConfig {
    /// Single command: cargo run, zig run
    Direct {
        program: String,
        args: Vec<String>,
        env: Vec<(String, String)>,
        cwd: PathBuf,
    },
    /// Two-step: compile first, then run binary
    CompileThenRun {
        compiler: String,
        compile_args: Vec<String>,
        binary_path: PathBuf,
        env: Vec<(String, String)>,
        cwd: PathBuf,
    },
}

// ── ToolInfo ─────────────────────────────────────────────────────────────────

/// Result of detecting a single tool (cargo, clang, zig, swiftc, etc.)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

// ── Shared FileLanguage helpers ──────────────────────────────────────────────
// Used by Native, and later Zig and Swift — flat-directory languages where
// playground files live at the project root with their extension in the name.

/// List playground files by scanning a directory for files with matching extensions.
pub fn file_list_playgrounds(dir: &Path, extensions: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|f| {
                std::path::Path::new(f)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| extensions.contains(&e))
                    .unwrap_or(false)
            })
            .collect(),
        Err(_) => vec![],
    };
    names.sort();
    names
}

/// Validate a file-based playground name: stem must be [a-z][a-z0-9_]*,
/// extension must be in the allowed set. Returns (stem, extension).
pub fn file_validate_name(name: &str, extensions: &[&str]) -> Result<(String, String), String> {
    let path = std::path::Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("'{}' has no valid stem", name))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| format!("'{}' has no extension", name))?;

    crate::validate_name(stem)?;
    if !extensions.contains(&ext) {
        return Err(format!(
            "Unsupported extension '.{}' — use {}",
            ext,
            extensions
                .iter()
                .map(|e| format!(".{}", e))
                .collect::<Vec<_>>()
                .join(" or ")
        ));
    }
    Ok((stem.to_string(), ext.to_string()))
}

/// Resolve path for a file-based playground. Includes traversal check.
pub fn file_playground_path(
    name: &str,
    dir: &Path,
    extensions: &[&str],
) -> Result<PathBuf, String> {
    file_validate_name(name, extensions)?;
    let path = dir.join(name);
    if let Ok(resolved_dir) = dir.canonicalize() {
        let resolved_parent = path
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .unwrap_or_else(|| resolved_dir.clone());
        if resolved_parent != resolved_dir {
            return Err(format!("Path traversal detected for '{}'", name));
        }
    }
    Ok(path)
}

// ── Native-specific helpers (will move to native.rs in Phase 2) ──��──────────

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
        _ => format!("// Hello from {}!\n", name),
    }
}

/// Resolve the clang compiler path via xcrun, falling back to /usr/bin/clang.
fn resolve_clang() -> String {
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
fn resolve_sdk_path() -> Option<String> {
    std::process::Command::new("xcrun")
        .args(["--show-sdk-path"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Build a RunConfig for a native (C/C++) playground.
fn native_build_run_command(
    name: &str,
    source_path: &Path,
    workspace: &Path,
    manifest: &RusticManifest,
) -> Result<RunConfig, String> {
    let (stem, ext) = file_validate_name(name, &["c", "cpp"])?;
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
