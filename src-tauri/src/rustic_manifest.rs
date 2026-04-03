use std::path::Path;
use tauri::AppHandle;

use crate::{cargo_path, projects_dir};

// ── Manifest types ───────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RusticManifest {
    pub project: ProjectInfo,
    #[serde(default)]
    pub paths: PathsInfo,
    #[serde(default)]
    pub toolchain: ToolchainInfo,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProjectInfo {
    #[serde(rename = "type")]
    pub project_type: String,
    #[serde(default)]
    pub created_with: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PathsInfo {
    pub src: String,
    pub content: String,
}

impl Default for PathsInfo {
    fn default() -> Self {
        Self {
            src: "src/bin".to_string(),
            content: "content".to_string(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub struct ToolchainInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rustc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zig: Option<String>,
}

// ── Constants ────────────────────────────────────────────────────────────────

const MANIFEST_FILENAME: &str = "rustic.toml";
const APP_VERSION: &str = "0.2";

// ── Supported native extensions ──────────────────────────────────────────────

#[allow(dead_code)] // used in upcoming playground CRUD for native projects
pub const NATIVE_EXTENSIONS: &[&str] = &["c", "cpp", "zig", "rs"];

#[allow(dead_code)] // used in upcoming playground CRUD for native projects
pub fn is_supported_extension(ext: &str) -> bool {
    NATIVE_EXTENSIONS.contains(&ext)
}

// ── Read / write ─────────────────────────────────────────────────────────────

/// Read rustic.toml from a project directory. Returns None if the file doesn't exist.
pub fn read_manifest(project_dir: &Path) -> Option<RusticManifest> {
    let path = project_dir.join(MANIFEST_FILENAME);
    let content = std::fs::read_to_string(&path).ok()?;
    toml::from_str(&content).ok()
}

/// Write rustic.toml to a project directory.
pub fn write_manifest(project_dir: &Path, manifest: &RusticManifest) -> Result<(), String> {
    let content = toml::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    std::fs::write(project_dir.join(MANIFEST_FILENAME), content)
        .map_err(|e| format!("Failed to write rustic.toml: {}", e))
}

// ── Project type detection ───────────────────────────────────────────────────

/// Determine project type. Reads rustic.toml first, then falls back to heuristics.
pub fn detect_project_type(project_dir: &Path) -> String {
    // Primary: read manifest
    if let Some(manifest) = read_manifest(project_dir) {
        return manifest.project.project_type;
    }
    // Fallback: Cargo.toml present → rust
    if project_dir.join("Cargo.toml").exists() {
        return "rust".to_string();
    }
    // Otherwise: native
    "native".to_string()
}

// ── Manifest generation ──────────────────────────────────────────────────────

/// Create a manifest for a new Rust project.
pub fn new_rust_manifest() -> RusticManifest {
    RusticManifest {
        project: ProjectInfo {
            project_type: "rust".to_string(),
            created_with: APP_VERSION.to_string(),
        },
        paths: PathsInfo {
            src: "src/bin".to_string(),
            content: "content".to_string(),
        },
        toolchain: detect_rust_toolchain(),
    }
}

/// Create a manifest for a new native project.
pub fn new_native_manifest() -> RusticManifest {
    RusticManifest {
        project: ProjectInfo {
            project_type: "native".to_string(),
            created_with: APP_VERSION.to_string(),
        },
        paths: PathsInfo {
            src: ".".to_string(),
            content: "content".to_string(),
        },
        toolchain: detect_native_toolchain(),
    }
}

/// Auto-generate a manifest for a legacy project that doesn't have one.
pub fn generate_legacy_manifest(project_dir: &Path) -> RusticManifest {
    if project_dir.join("Cargo.toml").exists() {
        RusticManifest {
            project: ProjectInfo {
                project_type: "rust".to_string(),
                created_with: String::new(), // unknown — legacy
            },
            paths: PathsInfo {
                src: "src/bin".to_string(),
                content: "content".to_string(),
            },
            toolchain: detect_rust_toolchain(),
        }
    } else {
        RusticManifest {
            project: ProjectInfo {
                project_type: "native".to_string(),
                created_with: String::new(),
            },
            paths: PathsInfo {
                src: ".".to_string(),
                content: "content".to_string(),
            },
            toolchain: detect_native_toolchain(),
        }
    }
}

/// Ensure a project has a rustic.toml. If missing, auto-generate one.
pub fn ensure_manifest(project_dir: &Path) -> Result<RusticManifest, String> {
    if let Some(manifest) = read_manifest(project_dir) {
        return Ok(manifest);
    }
    let manifest = generate_legacy_manifest(project_dir);
    write_manifest(project_dir, &manifest)?;
    Ok(manifest)
}

// ── Toolchain detection ──────────────────────────────────────────────────────

fn detect_rust_toolchain() -> ToolchainInfo {
    let cargo_bin = cargo_path();
    let cargo_dir = std::path::Path::new(&cargo_bin)
        .parent()
        .unwrap_or(std::path::Path::new(""));

    let rustc_bin = cargo_dir.join("rustc");
    let rustc_bin = if rustc_bin.exists() {
        rustc_bin.to_string_lossy().to_string()
    } else {
        "rustc".to_string()
    };

    ToolchainInfo {
        cargo: run_version_command(&cargo_bin, &["--version"]),
        rustc: run_version_command(&rustc_bin, &["--version"]),
        clang: None,
        zig: None,
    }
}

fn detect_native_toolchain() -> ToolchainInfo {
    ToolchainInfo {
        rustc: run_version_command("rustc", &["--version"]),
        cargo: None,
        clang: detect_clang_version(),
        zig: detect_zig_version(),
    }
}

fn detect_clang_version() -> Option<String> {
    // Try xcrun first (works with just Command Line Tools)
    if let Some(clang_path) = std::process::Command::new("xcrun")
        .args(["--find", "clang"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        return run_version_command(&clang_path, &["--version"]).map(|v| {
            // clang --version outputs multiple lines; take the first
            v.lines().next().unwrap_or(&v).to_string()
        });
    }
    // Fallback: try /usr/bin/clang
    run_version_command("/usr/bin/clang", &["--version"])
        .map(|v| v.lines().next().unwrap_or(&v).to_string())
}

fn detect_zig_version() -> Option<String> {
    run_version_command("zig", &["version"])
}

fn run_version_command(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

// ── Tauri commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_project_type(name: String, app: AppHandle) -> String {
    let project_dir = projects_dir(&app).join(&name);
    detect_project_type(&project_dir)
}

#[tauri::command]
pub fn get_project_manifest(name: String, app: AppHandle) -> Result<RusticManifest, String> {
    let project_dir = projects_dir(&app).join(&name);
    if !project_dir.exists() {
        return Err(format!("Project '{}' does not exist", name));
    }
    ensure_manifest(&project_dir)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rust_manifest_has_correct_defaults() {
        let m = new_rust_manifest();
        assert_eq!(m.project.project_type, "rust");
        assert_eq!(m.project.created_with, "0.2");
        assert_eq!(m.paths.src, "src/bin");
        assert_eq!(m.paths.content, "content");
    }

    #[test]
    fn new_native_manifest_has_correct_defaults() {
        let m = new_native_manifest();
        assert_eq!(m.project.project_type, "native");
        assert_eq!(m.project.created_with, "0.2");
        assert_eq!(m.paths.src, ".");
        assert_eq!(m.paths.content, "content");
    }

    #[test]
    fn manifest_roundtrip_toml() {
        let m = new_rust_manifest();
        let toml_str = toml::to_string_pretty(&m).unwrap();
        let m2: RusticManifest = toml::from_str(&toml_str).unwrap();
        assert_eq!(m2.project.project_type, "rust");
        assert_eq!(m2.paths.src, "src/bin");
        assert_eq!(m2.paths.content, "content");
    }

    #[test]
    fn manifest_deserialize_minimal() {
        let toml_str = r#"
[project]
type = "native"
"#;
        let m: RusticManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(m.project.project_type, "native");
        assert_eq!(m.project.created_with, "");
        // Defaults kick in for paths
        assert_eq!(m.paths.src, "src/bin");
        assert_eq!(m.paths.content, "content");
    }

    #[test]
    fn detect_type_with_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(detect_project_type(dir.path()), "rust");
    }

    #[test]
    fn detect_type_without_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("hello.c"), "int main() {}").unwrap();
        assert_eq!(detect_project_type(dir.path()), "native");
    }

    #[test]
    fn detect_type_prefers_manifest() {
        let dir = tempfile::tempdir().unwrap();
        // Has both Cargo.toml and rustic.toml saying "native"
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let m = RusticManifest {
            project: ProjectInfo {
                project_type: "native".to_string(),
                created_with: "0.2".to_string(),
            },
            paths: PathsInfo::default(),
            toolchain: ToolchainInfo::default(),
        };
        write_manifest(dir.path(), &m).unwrap();
        assert_eq!(detect_project_type(dir.path()), "native");
    }

    #[test]
    fn write_and_read_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let m = new_native_manifest();
        write_manifest(dir.path(), &m).unwrap();
        let m2 = read_manifest(dir.path()).unwrap();
        assert_eq!(m2.project.project_type, "native");
        assert_eq!(m2.paths.src, ".");
    }

    #[test]
    fn ensure_manifest_creates_for_legacy_rust() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let m = ensure_manifest(dir.path()).unwrap();
        assert_eq!(m.project.project_type, "rust");
        // File should now exist
        assert!(dir.path().join("rustic.toml").exists());
    }

    #[test]
    fn ensure_manifest_creates_for_legacy_native() {
        let dir = tempfile::tempdir().unwrap();
        let m = ensure_manifest(dir.path()).unwrap();
        assert_eq!(m.project.project_type, "native");
        assert!(dir.path().join("rustic.toml").exists());
    }

    #[test]
    fn ensure_manifest_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let m = new_native_manifest();
        write_manifest(dir.path(), &m).unwrap();
        let m2 = ensure_manifest(dir.path()).unwrap();
        assert_eq!(m2.project.created_with, "0.2");
    }

    #[test]
    fn is_supported_extension_works() {
        assert!(is_supported_extension("c"));
        assert!(is_supported_extension("cpp"));
        assert!(is_supported_extension("zig"));
        assert!(is_supported_extension("rs"));
        assert!(!is_supported_extension("py"));
        assert!(!is_supported_extension("go"));
        assert!(!is_supported_extension(""));
    }

    #[test]
    fn legacy_manifest_has_empty_created_with() {
        let dir = tempfile::tempdir().unwrap();
        let m = generate_legacy_manifest(dir.path());
        assert!(m.project.created_with.is_empty());
    }
}
