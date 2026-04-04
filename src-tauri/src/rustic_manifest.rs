use std::path::Path;
use tauri::{AppHandle, Manager};

use crate::{cargo_path, projects_dir};

// ── Manifest types ───────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RusticManifest {
    pub project: ProjectInfo,
    #[serde(default)]
    pub paths: PathsInfo,
    #[serde(default)]
    pub build: BuildInfo,
    #[serde(default)]
    pub toolchain: ToolchainInfo,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locked: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ProjectInfo {
    #[serde(rename = "type")]
    pub project_type: String,
    #[serde(default)]
    pub created_with: String,
    /// Where this project came from: "user" (default), "rust_book", "knr_book", "swift_book".
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
    /// Whether the project is read-only (no editing, renaming, or deleting).
    #[serde(default, skip_serializing_if = "is_false")]
    pub readonly: bool,
}

fn is_false(v: &bool) -> bool {
    !v
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

/// Compiler/build flags per language.
/// Clang: cflags/cxxflags. Zig: zigflags. Swift: swiftflags (Phase 4).
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct BuildInfo {
    #[serde(default = "default_cflags")]
    pub cflags: Vec<String>,
    #[serde(default = "default_cxxflags")]
    pub cxxflags: Vec<String>,
    #[serde(default)]
    pub zigflags: Vec<String>,
    #[serde(default)]
    pub swiftflags: Vec<String>,
}

fn default_cflags() -> Vec<String> {
    vec!["-lsqlite3".to_string()]
}

fn default_cxxflags() -> Vec<String> {
    vec!["-std=c++17".to_string(), "-lsqlite3".to_string()]
}

impl Default for BuildInfo {
    fn default() -> Self {
        Self {
            cflags: default_cflags(),
            cxxflags: default_cxxflags(),
            zigflags: vec![],
            swiftflags: vec![],
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zig: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swiftc: Option<String>,
}

// ── Constants ────────────────────────────────────────────────────────────────

const MANIFEST_FILENAME: &str = "rustic.toml";
const APP_VERSION: &str = "0.2";

// ── Supported clang extensions ───────────────────────────────────────────────

#[allow(dead_code)] // used by tests and future language expansion
pub const NATIVE_EXTENSIONS: &[&str] = &["c", "cpp"];

#[allow(dead_code)] // used by tests and future language expansion
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
    // Fallback heuristics
    if project_dir.join("Cargo.toml").exists() {
        return "rust".to_string();
    }
    // Check for .zig or .swift files
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            match entry.path().extension().and_then(|e| e.to_str()) {
                Some("zig") => return "zig".to_string(),
                Some("swift") => return "swift".to_string(),
                _ => {}
            }
        }
    }
    // Default: clang
    "clang".to_string()
}

// ── Manifest generation ──────────────────────────────────────────────────────

/// Create a manifest for a new Rust project.
pub fn new_rust_manifest() -> RusticManifest {
    RusticManifest {
        project: ProjectInfo {
            project_type: "rust".to_string(),
            created_with: APP_VERSION.to_string(),
            source: String::new(),
            readonly: false,
        },
        paths: PathsInfo {
            src: "src/bin".to_string(),
            content: "content".to_string(),
        },
        build: BuildInfo {
            cflags: vec![],
            cxxflags: vec![],
            zigflags: vec![],
            swiftflags: vec![],
        },
        toolchain: detect_rust_toolchain(),
        locked: vec![],
    }
}

/// Create a manifest for a new clang project.
pub fn new_clang_manifest() -> RusticManifest {
    RusticManifest {
        project: ProjectInfo {
            project_type: "clang".to_string(),
            created_with: APP_VERSION.to_string(),
            source: String::new(),
            readonly: false,
        },
        paths: PathsInfo {
            src: ".".to_string(),
            content: "content".to_string(),
        },
        build: BuildInfo::default(),
        toolchain: detect_clang_toolchain(),
        locked: vec![],
    }
}

/// Create a manifest for a new Zig project.
pub fn new_zig_manifest() -> RusticManifest {
    RusticManifest {
        project: ProjectInfo {
            project_type: "zig".to_string(),
            created_with: APP_VERSION.to_string(),
            source: String::new(),
            readonly: false,
        },
        paths: PathsInfo {
            src: ".".to_string(),
            content: "content".to_string(),
        },
        build: BuildInfo {
            cflags: vec![],
            cxxflags: vec![],
            zigflags: vec![],
            swiftflags: vec![],
        },
        toolchain: detect_zig_toolchain(),
        locked: vec![],
    }
}

/// Create a manifest for a new Swift project.
pub fn new_swift_manifest() -> RusticManifest {
    RusticManifest {
        project: ProjectInfo {
            project_type: "swift".to_string(),
            created_with: APP_VERSION.to_string(),
            source: String::new(),
            readonly: false,
        },
        paths: PathsInfo {
            src: ".".to_string(),
            content: "content".to_string(),
        },
        build: BuildInfo {
            cflags: vec![],
            cxxflags: vec![],
            zigflags: vec![],
            swiftflags: vec![],
        },
        toolchain: detect_swift_toolchain(),
        locked: vec![],
    }
}

/// Auto-generate a manifest for a legacy project that doesn't have one.
pub fn generate_legacy_manifest(project_dir: &Path) -> RusticManifest {
    if project_dir.join("Cargo.toml").exists() {
        RusticManifest {
            project: ProjectInfo {
                project_type: "rust".to_string(),
                created_with: String::new(), // unknown — legacy
                source: String::new(),
                readonly: false,
            },
            paths: PathsInfo {
                src: "src/bin".to_string(),
                content: "content".to_string(),
            },
            build: BuildInfo {
                cflags: vec![],
                cxxflags: vec![],
                zigflags: vec![],
                swiftflags: vec![],
            },
            toolchain: detect_rust_toolchain(),
            locked: vec![],
        }
    } else {
        RusticManifest {
            project: ProjectInfo {
                project_type: "clang".to_string(),
                created_with: String::new(),
                source: String::new(),
                readonly: false,
            },
            paths: PathsInfo {
                src: ".".to_string(),
                content: "content".to_string(),
            },
            build: BuildInfo::default(),
            toolchain: detect_clang_toolchain(),
            locked: vec![],
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
        swiftc: None,
    }
}

fn detect_clang_toolchain() -> ToolchainInfo {
    ToolchainInfo {
        rustc: None,
        cargo: None,
        clang: detect_clang_version(),
        zig: None,
        swiftc: None,
    }
}

fn detect_zig_toolchain() -> ToolchainInfo {
    ToolchainInfo {
        rustc: None,
        cargo: None,
        clang: None,
        zig: run_version_command("zig", &["version"]),
        swiftc: None,
    }
}

fn detect_swift_toolchain() -> ToolchainInfo {
    // swiftc --version outputs multiple lines; take the first
    let version = std::process::Command::new("swiftc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()));
    ToolchainInfo {
        rustc: None,
        cargo: None,
        clang: None,
        zig: None,
        swiftc: version,
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

/// Get the source tag for a single project (empty string = user project).
/// Falls back to name-prefix detection for projects created before the source field existed.
pub fn get_project_source(project_dir: &Path) -> String {
    if let Some(manifest) = read_manifest(project_dir) {
        return manifest.project.source;
    }
    String::new()
}

#[tauri::command]
pub fn get_project_type(name: String, app: AppHandle) -> String {
    let project_dir = projects_dir(&app).join(&name);
    detect_project_type(&project_dir)
}

/// Returns a map of project name → readonly flag for all projects.
#[tauri::command]
pub fn get_project_readonly_map(app: AppHandle) -> std::collections::HashMap<String, bool> {
    let pdir = projects_dir(&app);
    let mut result = std::collections::HashMap::new();
    if let Ok(entries) = std::fs::read_dir(&pdir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if let Some(manifest) = read_manifest(&entry.path()) {
                        if manifest.project.readonly {
                            result.insert(name.to_string(), true);
                        }
                    }
                }
            }
        }
    }
    result
}

/// Returns the list of locked playground names for the active project.
#[tauri::command]
pub fn get_locked_playgrounds(app: AppHandle) -> Vec<String> {
    let project = app
        .state::<crate::ActiveProject>()
        .0
        .lock()
        .unwrap()
        .clone();
    let project_dir = projects_dir(&app).join(&project);
    if let Some(manifest) = read_manifest(&project_dir) {
        return manifest.locked;
    }
    vec![]
}

/// Adds or removes a playground from the locked list.
#[tauri::command]
pub fn set_playground_locked(
    playground: String,
    locked: bool,
    app: AppHandle,
) -> Result<(), String> {
    let project = app
        .state::<crate::ActiveProject>()
        .0
        .lock()
        .unwrap()
        .clone();
    let project_dir = projects_dir(&app).join(&project);
    let mut manifest = ensure_manifest(&project_dir)?;
    if locked {
        if !manifest.locked.contains(&playground) {
            manifest.locked.push(playground);
        }
    } else {
        manifest.locked.retain(|n| n != &playground);
    }
    write_manifest(&project_dir, &manifest)
}

/// Returns a map of project name → source tag for all projects.
#[tauri::command]
pub fn get_project_sources(app: AppHandle) -> std::collections::HashMap<String, String> {
    let pdir = projects_dir(&app);
    let mut sources = std::collections::HashMap::new();
    if let Ok(entries) = std::fs::read_dir(&pdir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    let source = get_project_source(&entry.path());
                    if !source.is_empty() {
                        sources.insert(name.to_string(), source);
                    }
                }
            }
        }
    }
    sources
}

#[tauri::command]
pub fn get_project_manifest(name: String, app: AppHandle) -> Result<RusticManifest, String> {
    let project_dir = projects_dir(&app).join(&name);
    if !project_dir.exists() {
        return Err(format!("Project '{}' does not exist", name));
    }
    ensure_manifest(&project_dir)
}

#[tauri::command]
pub fn get_build_flags(app: AppHandle) -> Result<BuildInfo, String> {
    use tauri::Manager;
    let active = app
        .state::<crate::ActiveProject>()
        .0
        .lock()
        .unwrap()
        .clone();
    let project_dir = projects_dir(&app).join(&active);
    let manifest = ensure_manifest(&project_dir)?;
    Ok(manifest.build)
}

#[tauri::command]
pub fn save_build_flags(
    cflags: Vec<String>,
    cxxflags: Vec<String>,
    zigflags: Vec<String>,
    swiftflags: Vec<String>,
    app: AppHandle,
) -> Result<(), String> {
    use tauri::Manager;
    let active = app
        .state::<crate::ActiveProject>()
        .0
        .lock()
        .unwrap()
        .clone();
    let project_dir = projects_dir(&app).join(&active);
    let mut manifest = ensure_manifest(&project_dir)?;
    manifest.build.cflags = cflags;
    manifest.build.cxxflags = cxxflags;
    manifest.build.zigflags = zigflags;
    manifest.build.swiftflags = swiftflags;
    write_manifest(&project_dir, &manifest)
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
    fn new_clang_manifest_has_correct_defaults() {
        let m = new_clang_manifest();
        assert_eq!(m.project.project_type, "clang");
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
type = "clang"
"#;
        let m: RusticManifest = toml::from_str(toml_str).unwrap();
        assert_eq!(m.project.project_type, "clang");
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
        assert_eq!(detect_project_type(dir.path()), "clang");
    }

    #[test]
    fn detect_type_prefers_manifest() {
        let dir = tempfile::tempdir().unwrap();
        // Has both Cargo.toml and rustic.toml saying "clang"
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();
        let m = RusticManifest {
            project: ProjectInfo {
                project_type: "clang".to_string(),
                created_with: "0.2".to_string(),
                source: String::new(),
                readonly: false,
            },
            paths: PathsInfo::default(),
            build: BuildInfo::default(),
            toolchain: ToolchainInfo::default(),
            locked: vec![],
        };
        write_manifest(dir.path(), &m).unwrap();
        assert_eq!(detect_project_type(dir.path()), "clang");
    }

    #[test]
    fn write_and_read_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let m = new_clang_manifest();
        write_manifest(dir.path(), &m).unwrap();
        let m2 = read_manifest(dir.path()).unwrap();
        assert_eq!(m2.project.project_type, "clang");
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
    fn ensure_manifest_creates_for_legacy_clang() {
        let dir = tempfile::tempdir().unwrap();
        let m = ensure_manifest(dir.path()).unwrap();
        assert_eq!(m.project.project_type, "clang");
        assert!(dir.path().join("rustic.toml").exists());
    }

    #[test]
    fn ensure_manifest_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let m = new_clang_manifest();
        write_manifest(dir.path(), &m).unwrap();
        let m2 = ensure_manifest(dir.path()).unwrap();
        assert_eq!(m2.project.created_with, "0.2");
    }

    #[test]
    fn is_supported_extension_works() {
        assert!(is_supported_extension("c"));
        assert!(is_supported_extension("cpp"));
        assert!(!is_supported_extension("zig"));
        assert!(!is_supported_extension("rs"));
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
