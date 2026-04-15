//! Per-language modules for Rustic Playground.
//!
//! Each supported language has its own module implementing language-specific
//! behavior (scaffold, run, export, toolchain detection). The `Lang` enum
//! dispatches to the right module via exhaustive match — the compiler ensures
//! every arm is handled when a new language is added.

pub mod clang;
mod knr_book;
pub mod rust;
mod rust_book;
pub mod swift;
mod swift_book;
pub mod zig;

use std::path::{Path, PathBuf};

use crate::rustic_manifest::RusticManifest;

// ── BookInfo ────────────────────────────────────────────────────────────────

/// Metadata for a language's companion book. Used to build dynamic menu items,
/// event mappings, and frontend toast messages.
pub struct BookInfo {
    pub book_name: &'static str,
    pub menu_id: &'static str,
    pub remove_menu_id: &'static str,
    pub event_name: &'static str,
    pub remove_event_name: &'static str,
    /// Value written into rustic.toml `project.source` (e.g. "rust_book").
    pub source_tag: &'static str,
    pub url: &'static str,
}

// ── Lang enum ────────────────────────────────────────────────────────────────

/// All supported project types. Adding a variant here will cause compile errors
/// at every `match` site until the new language is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Clang,
    Zig,
    Swift,
}

impl Lang {
    pub fn from_str(s: &str) -> Self {
        match s {
            "rust" => Lang::Rust,
            "clang" => Lang::Clang,
            "zig" => Lang::Zig,
            "swift" => Lang::Swift,
            _ => Lang::Rust, // fallback
        }
    }

    #[allow(dead_code)]
    pub fn project_type(&self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Clang => "clang",
            Lang::Zig => "zig",
            Lang::Swift => "swift",
        }
    }

    /// All language variants, in display order.
    pub fn all() -> &'static [Lang] {
        &[Lang::Rust, Lang::Clang, Lang::Zig, Lang::Swift]
    }

    /// Returns book info if this language has a companion book.
    pub fn book_info(&self) -> Option<BookInfo> {
        match self {
            Lang::Rust => Some(BookInfo {
                book_name: "The Rust Programming Language (2024 Edition)",
                menu_id: "seed_rust_book",
                remove_menu_id: "remove_rust_book",
                event_name: "menu:rust-book",
                remove_event_name: "menu:remove-rust-book",
                source_tag: "rust_book",
                url: "https://doc.rust-lang.org/book/",
            }),
            Lang::Clang => Some(BookInfo {
                book_name: "The C Programming Language (K&&R)",
                menu_id: "seed_knr_book",
                remove_menu_id: "remove_knr_book",
                event_name: "menu:knr-book",
                remove_event_name: "menu:remove-knr-book",
                source_tag: "knr_book",
                url: "https://en.wikipedia.org/wiki/The_C_Programming_Language",
            }),
            Lang::Zig => None,
            Lang::Swift => Some(BookInfo {
                book_name: "The Swift Programming Language",
                menu_id: "seed_swift_book",
                remove_menu_id: "remove_swift_book",
                event_name: "menu:swift-book",
                remove_event_name: "menu:remove-swift-book",
                source_tag: "swift_book",
                url: "https://docs.swift.org/swift-book/",
            }),
        }
    }

    /// Seed companion book chapters into the projects directory.
    /// Returns the list of newly created project names.
    pub fn seed_book(&self, projects_dir: &Path) -> Result<Vec<String>, String> {
        match self {
            Lang::Rust => rust_book::seed_book(projects_dir),
            Lang::Clang => knr_book::seed_book(projects_dir),
            Lang::Swift => swift_book::seed_book(projects_dir),
            Lang::Zig => Err("Zig has no companion book".to_string()),
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Lang::Rust => &["rs"],
            Lang::Clang => &["c", "cpp"],
            Lang::Zig => &["zig"],
            Lang::Swift => &["swift"],
        }
    }

    /// Where source files live relative to project root.
    #[allow(dead_code)]
    pub fn src_dir(&self) -> &'static str {
        match self {
            Lang::Rust => "src/bin",
            Lang::Clang | Lang::Zig | Lang::Swift => ".",
        }
    }

    /// Whether this language supports live error checking.
    pub fn supports_live_check(&self) -> bool {
        match self {
            Lang::Rust => true,
            Lang::Clang | Lang::Zig | Lang::Swift => false,
        }
    }

    /// List playground names from the source directory.
    pub fn list_playgrounds(&self, src_dir: &Path) -> Vec<String> {
        match self {
            Lang::Rust => rust::list_playgrounds(src_dir),
            Lang::Clang | Lang::Zig | Lang::Swift => {
                file_list_playgrounds(src_dir, self.extensions())
            }
        }
    }

    /// Validate a playground name. Returns (stem, extension).
    pub fn validate_name(&self, name: &str) -> Result<(String, String), String> {
        match self {
            Lang::Rust => rust::validate_name(name),
            Lang::Clang | Lang::Zig | Lang::Swift => file_validate_name(name, self.extensions()),
        }
    }

    /// Resolve full path to a playground file within the given source directory.
    pub fn playground_path(&self, name: &str, src_dir: &Path) -> Result<PathBuf, String> {
        match self {
            Lang::Rust => rust::playground_path(name, src_dir),
            Lang::Clang | Lang::Zig | Lang::Swift => {
                file_playground_path(name, src_dir, self.extensions())
            }
        }
    }

    /// Create a new project's directory structure and seed files.
    pub fn scaffold_project(&self, project_path: &Path, project_name: &str) -> Result<(), String> {
        match self {
            Lang::Rust => rust::scaffold_project(project_path, project_name),
            Lang::Clang => clang::scaffold_project(project_path),
            Lang::Zig => zig::scaffold_project(project_path),
            Lang::Swift => swift::scaffold_project(project_path),
        }
    }

    /// Generate starter code for a new playground file.
    pub fn starter_template(&self, name: &str, ext: &str) -> String {
        match self {
            Lang::Rust => rust::starter_template(name),
            Lang::Clang => clang::starter_template(name, ext),
            Lang::Zig => zig::starter_template(name),
            Lang::Swift => swift::starter_template(name),
        }
    }

    /// Generate a new manifest for this language type.
    pub fn new_manifest(&self) -> RusticManifest {
        match self {
            Lang::Rust => crate::rustic_manifest::new_rust_manifest(),
            Lang::Clang => crate::rustic_manifest::new_clang_manifest(),
            Lang::Zig => crate::rustic_manifest::new_zig_manifest(),
            Lang::Swift => crate::rustic_manifest::new_swift_manifest(),
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
            Lang::Clang => clang::build_run_command(name, source_path, workspace, manifest),
            Lang::Zig => zig::build_run_command(name, source_path, workspace, manifest),
            Lang::Swift => swift::build_run_command(name, source_path, workspace, manifest),
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
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

// ── Shared FileLanguage helpers ──────────────────────────────────────────────
// Used by Clang, Zig, and Swift — flat-directory languages where playground
// files live at the project root with their extension in the name.

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
