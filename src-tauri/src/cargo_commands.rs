use tauri::AppHandle;

use crate::{cargo_path, config_path, load_config, settings_path, workspace_dir, Config, Settings};

#[tauri::command]
pub fn get_cargo_toml(app: AppHandle) -> Result<String, String> {
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_cargo_toml(content: String, app: AppHandle) -> Result<(), String> {
    // Validate TOML before saving
    content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Invalid TOML: {}", e))?;
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

/// Validate a crate name: must start with a letter, contain only [a-zA-Z0-9_-].
fn validate_crate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Crate name cannot be empty".to_string());
    }
    if !name.chars().next().unwrap().is_ascii_alphabetic() {
        return Err("Crate name must start with a letter".to_string());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "Crate name can only contain letters, digits, hyphens, and underscores".to_string(),
        );
    }
    Ok(())
}

/// Validate a version string: basic semver-ish check.
fn validate_version(version: &str) -> Result<(), String> {
    if version == "*" {
        return Ok(());
    }
    // Allow optional leading operator: ^, ~, =, >=, <=, >
    let v = version
        .trim_start_matches(|c: char| "^~=><".contains(c))
        .trim();
    if v.is_empty() {
        return Err("Version cannot be empty after operator".to_string());
    }
    // Each dot-separated part must be numeric or *
    for part in v.split('.') {
        if part != "*" && part.parse::<u64>().is_err() {
            return Err(format!("Invalid version component: '{}'", part));
        }
    }
    Ok(())
}

/// Add a dependency to the active project's Cargo.toml.
/// Accepts the current editor content so it's always in sync.
/// Returns the updated file content (format-preserving).
#[tauri::command]
pub fn add_dependency(
    content: String,
    name: String,
    version: String,
    app: AppHandle,
) -> Result<String, String> {
    validate_crate_name(&name)?;
    // Skip version validation for inline table specs like { version = "1", features = [...] }
    if !version.trim_start().starts_with('{') {
        validate_version(&version)?;
    }

    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Invalid Cargo.toml: {}", e))?;

    // Ensure [dependencies] table exists
    if !doc.contains_key("dependencies") {
        doc["dependencies"] = toml_edit::Item::Table(toml_edit::Table::new());
    }

    // Check if already present
    if doc["dependencies"].get(&name).is_some() {
        return Err(format!("'{}' is already in [dependencies]", name));
    }

    // If version looks like an inline table (e.g. `{ version = "1", features = ["full"] }`),
    // parse it as TOML value; otherwise treat it as a plain version string.
    if version.trim_start().starts_with('{') {
        let tmp = format!("x = {}", version);
        let tmp_doc = tmp
            .parse::<toml_edit::DocumentMut>()
            .map_err(|e| format!("Invalid dependency spec: {}", e))?;
        doc["dependencies"][&name] = tmp_doc["x"].clone();
    } else {
        doc["dependencies"][&name] = toml_edit::value(&version);
    }

    let updated = doc.to_string();
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::write(&path, &updated).map_err(|e| e.to_string())?;
    Ok(updated)
}

/// Remove a dependency from the active project's Cargo.toml.
/// Accepts the current editor content so it's always in sync.
/// Returns the updated file content.
#[tauri::command]
pub fn remove_dependency(content: String, name: String, app: AppHandle) -> Result<String, String> {
    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Invalid Cargo.toml: {}", e))?;

    if let Some(deps) = doc.get_mut("dependencies").and_then(|d| d.as_table_mut()) {
        if deps.remove(&name).is_none() {
            return Err(format!("'{}' not found in [dependencies]", name));
        }
    } else {
        return Err("No [dependencies] table found".to_string());
    }

    let updated = doc.to_string();
    let path = workspace_dir(&app).join("Cargo.toml");
    std::fs::write(&path, &updated).map_err(|e| e.to_string())?;
    Ok(updated)
}

#[tauri::command]
pub fn get_toolchain_info() -> serde_json::Value {
    let path = cargo_path();
    // RUSTUP_AUTO_INSTALL=0 — same reason as in check_toolchain: this is a
    // read-only probe, we don't want the rustup proxy to silently re-install
    // a missing default toolchain as a side effect.
    let version = std::process::Command::new(&path)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "cargo (not found)".to_string())
        .trim()
        .to_string();
    serde_json::json!({ "path": path, "version": version })
}

/// Comprehensive toolchain check for the setup wizard.
/// Returns status of rustup, cargo, rustc, and installed toolchains.
#[tauri::command]
pub fn check_toolchain(app: AppHandle) -> serde_json::Value {
    let config = load_config(&app);

    // Resolve cargo and its bin directory for sibling tools.
    // macOS app bundles get a minimal PATH that excludes ~/.cargo/bin,
    // so we must use absolute paths for all Rust toolchain binaries.
    let settings = {
        let path = settings_path(&app);
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
                .unwrap_or_default()
        } else {
            Settings::default()
        }
    };
    let cargo = if settings.cargo_path.is_empty() {
        cargo_path()
    } else {
        settings.cargo_path.clone()
    };
    let cargo_dir = std::path::Path::new(&cargo)
        .parent()
        .unwrap_or(std::path::Path::new(""))
        .to_path_buf();

    // Helper: resolve a tool name to an absolute path via the cargo bin dir
    let tool_path = |name: &str| -> String {
        let abs = cargo_dir.join(name);
        if abs.exists() {
            abs.to_string_lossy().to_string()
        } else {
            name.to_string() // fall back to bare name (PATH lookup)
        }
    };

    // Check Xcode Command Line Tools via `xcode-select -p` — the official
    // Apple API for querying the active developer directory. Returns the
    // path (e.g. /Library/Developer/CommandLineTools) on success, or exit
    // code 2 when no developer directory is configured.
    //
    // This probe is safe: `xcode-select -p` does NOT trigger Apple's
    // "Install Command Line Developer Tools" dialog (confirmed via manual
    // testing and log stream instrumentation on Sequoia, 2026-04-09).
    //
    // Note: on vanilla Macs, macOS may show an "Install Command Line
    // Developer Tools" dialog at app launch. This is triggered by Apple's
    // WKWebView framework (used by Tauri) probing for developer tools
    // during WebView initialization — NOT by our code. We cannot prevent
    // it from our side. The dialog is harmless: users can dismiss it with
    // "Not Now" and use the in-app toolchain installer when ready.
    let clt_output = std::process::Command::new("xcode-select")
        .arg("-p")
        .output()
        .ok();
    let clt_path_str: Option<String> = clt_output
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|p| !p.is_empty() && std::path::Path::new(p).exists());
    let xcode_clt_installed = clt_path_str.is_some();

    // Check rustup — single subprocess spawn, extract both presence and
    // version from the same output. (Previous version ran `rustup --version`
    // twice, which was pure waste of ~30-50ms per check_toolchain call.)
    let rustup_bin = tool_path("rustup");
    let rustup_output = std::process::Command::new(&rustup_bin)
        .arg("--version")
        .output()
        .ok();
    let rustup_installed = rustup_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let rustup_version = if rustup_installed {
        rustup_output
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // Check cargo. RUSTUP_AUTO_INSTALL=0 prevents the rustup proxy from
    // silently re-downloading a missing default toolchain just because we
    // asked for its version — we want to *observe* the broken state, not heal
    // it. Same flag applied to rustc, rustfmt, cargo-clippy, and rustup show
    // below for the same reason.
    let cargo_output = std::process::Command::new(&cargo)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .arg("--version")
        .output()
        .ok();
    let cargo_installed = cargo_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let cargo_version = cargo_output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    // Check rustc
    let rustc_bin = tool_path("rustc");
    let rustc_output = std::process::Command::new(&rustc_bin)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .arg("--version")
        .output()
        .ok();
    let rustc_installed = rustc_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let rustc_version = rustc_output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    // Get active toolchain if rustup is available
    let active_toolchain = if rustup_installed {
        std::process::Command::new(&rustup_bin)
            .env("RUSTUP_AUTO_INSTALL", "0")
            .args(["show", "active-toolchain"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // (Previously ran `rustup toolchain list` here, but the resulting
    // installed_toolchains field was dead code on the frontend — declared
    // in the TS type but never read or rendered. Removing that call saves
    // another ~50-100ms per check_toolchain, since rustup walks ~/.rustup
    // to compose the list.)

    // Check for essential components
    let has_rustfmt = std::process::Command::new(tool_path("rustfmt"))
        .env("RUSTUP_AUTO_INSTALL", "0")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_clippy = std::process::Command::new(tool_path("cargo-clippy"))
        .env("RUSTUP_AUTO_INSTALL", "0")
        .arg("--version")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check clang (C/C++ toolchain via Xcode Command Line Tools).
    //
    // CRITICAL: every probe in this block is guarded by xcode_clt_installed.
    // On a vanilla macOS, `xcrun`, `/usr/bin/clang`, and `swiftc` are SHIM
    // binaries that, when invoked, automatically pop Apple's "Install
    // Command Line Tools" dialog. That's a side effect we explicitly do NOT
    // want from a read-only status check — it would trigger Apple's
    // installer the moment the user opens the app, before they've clicked
    // anything. Skip the calls entirely when CLT is absent and just report
    // clang as not-installed.
    let (clang_path, clang_installed, clang_version) = if xcode_clt_installed {
        let path = std::process::Command::new("xcrun")
            .args(["--find", "clang"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "/usr/bin/clang".to_string());

        let output = std::process::Command::new(&path).arg("--version").output().ok();
        let installed = output.as_ref().map(|o| o.status.success()).unwrap_or(false);
        let version = output
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().next().map(|l| l.trim().to_string()));
        (path, installed, version)
    } else {
        (String::new(), false, None)
    };

    // Check zig
    let zig_output = std::process::Command::new("zig")
        .arg("version")
        .output()
        .ok();
    let zig_installed = zig_output
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let zig_version = zig_output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());
    let zig_path = if zig_installed {
        std::process::Command::new("which")
            .arg("zig")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        None
    };

    // Check swiftc (ships with Xcode CLI tools, same install as clang).
    // Same CLT-shim guard as clang above — invoking `swiftc` on a vanilla
    // macOS pops Apple's installer dialog. Skip the call when CLT is absent.
    let (swiftc_installed, swiftc_version, swiftc_path) = if xcode_clt_installed {
        let output = std::process::Command::new("swiftc").arg("--version").output().ok();
        let installed = output.as_ref().map(|o| o.status.success()).unwrap_or(false);
        let version = output
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .and_then(|s| s.lines().next().map(|l| l.trim().to_string()));
        let path = if installed {
            std::process::Command::new("xcrun")
                .args(["--find", "swiftc"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        (installed, version, path)
    } else {
        (false, None, String::new())
    };

    // `all_good` now requires CLT too — without Xcode Command Line Tools,
    // cargo can compile but fails at the link step (no cc, no SDK).
    let all_good = xcode_clt_installed && cargo_installed && rustc_installed;

    // Derive Rust toolchain state for the installer/repair flow.
    // States are mutually exclusive and ordered by severity:
    //   clt_missing       → Xcode Command Line Tools not installed → xcode-select --install
    //                       (must run BEFORE rustup install; Rust can't link without the SDK)
    //   not_installed     → no rustup binary at all → run sh.rustup.rs installer
    //   no_default        → rustup present but no active toolchain → rustup default stable
    //   missing_components → toolchain works but rustfmt/clippy missing → rustup component add
    //   healthy           → everything works
    let active_is_set = active_toolchain
        .as_ref()
        .map(|s| !s.is_empty() && !s.contains("no active toolchain"))
        .unwrap_or(false);
    let mut missing_components: Vec<&str> = Vec::new();
    if !has_rustfmt {
        missing_components.push("rustfmt");
    }
    if !has_clippy {
        missing_components.push("clippy");
    }
    let rust_state = if !xcode_clt_installed {
        "clt_missing"
    } else if !rustup_installed {
        "not_installed"
    } else if !cargo_installed || !rustc_installed || !active_is_set {
        "no_default"
    } else if !missing_components.is_empty() {
        "missing_components"
    } else {
        "healthy"
    };

    serde_json::json!({
        "wizard_completed": config.wizard_completed,
        "all_good": all_good,
        "rust_state": rust_state,
        "missing_components": missing_components,
        "xcode_clt": {
            "installed": xcode_clt_installed,
            "path": clt_path_str,
        },
        "rustup": {
            "installed": rustup_installed,
            "version": rustup_version,
        },
        "cargo": {
            "installed": cargo_installed,
            "path": cargo,
            "version": cargo_version,
        },
        "rustc": {
            "installed": rustc_installed,
            "version": rustc_version,
        },
        "active_toolchain": active_toolchain,
        "components": {
            "rustfmt": has_rustfmt,
            "clippy": has_clippy,
        },
        "clang": {
            "installed": clang_installed,
            "path": clang_path,
            "version": clang_version,
        },
        "zig": {
            "installed": zig_installed,
            "path": zig_path,
            "version": zig_version,
            "version_ok": zig_version.as_ref().map(|v| v.starts_with("0.15")).unwrap_or(false),
        },
        "swiftc": {
            "installed": swiftc_installed,
            "path": swiftc_path,
            "version": swiftc_version,
        }
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── validate_crate_name ──────────────────────────────────────────────

    #[test]
    fn crate_name_accepts_simple() {
        assert!(validate_crate_name("serde").is_ok());
        assert!(validate_crate_name("tokio").is_ok());
    }

    #[test]
    fn crate_name_accepts_hyphens_and_underscores() {
        assert!(validate_crate_name("serde-json").is_ok());
        assert!(validate_crate_name("serde_json").is_ok());
        assert!(validate_crate_name("my-crate-2").is_ok());
    }

    #[test]
    fn crate_name_rejects_empty() {
        assert!(validate_crate_name("").is_err());
    }

    #[test]
    fn crate_name_rejects_leading_digit() {
        assert!(validate_crate_name("2fast").is_err());
    }

    #[test]
    fn crate_name_rejects_leading_hyphen() {
        assert!(validate_crate_name("-bad").is_err());
    }

    #[test]
    fn crate_name_rejects_special_chars() {
        assert!(validate_crate_name("my@crate").is_err());
        assert!(validate_crate_name("my crate").is_err());
        assert!(validate_crate_name("my.crate").is_err());
    }

    // ── validate_version ─────────────────────────────────────────────────

    #[test]
    fn version_accepts_semver() {
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("0.1").is_ok());
        assert!(validate_version("2").is_ok());
    }

    #[test]
    fn version_accepts_operators() {
        assert!(validate_version("^1.0").is_ok());
        assert!(validate_version("~1.2.3").is_ok());
        assert!(validate_version(">=1.0.0").is_ok());
        assert!(validate_version("=1.0.0").is_ok());
    }

    #[test]
    fn version_accepts_wildcard() {
        assert!(validate_version("*").is_ok());
        assert!(validate_version("1.*").is_ok());
        assert!(validate_version("1.2.*").is_ok());
    }

    #[test]
    fn version_rejects_empty_after_operator() {
        assert!(validate_version("^").is_err());
        assert!(validate_version(">=").is_err());
    }

    #[test]
    fn version_rejects_non_numeric_parts() {
        assert!(validate_version("abc").is_err());
        assert!(validate_version("1.abc").is_err());
    }

    // ── TOML add/remove dependency (pure TOML logic) ─────────────────────

    #[test]
    fn add_dep_to_valid_toml() {
        let toml = "[package]\nname = \"test\"\n\n[dependencies]\n";
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        doc["dependencies"]["serde"] = toml_edit::value("1.0");
        let result = doc.to_string();
        assert!(result.contains("serde = \"1.0\""));
    }

    #[test]
    fn add_inline_table_dep() {
        let version = r#"{ version = "1", features = ["derive"] }"#;
        let tmp = format!("x = {}", version);
        let tmp_doc = tmp.parse::<toml_edit::DocumentMut>().unwrap();
        assert!(tmp_doc["x"].is_inline_table());
    }

    #[test]
    fn remove_dep_from_toml() {
        let toml = "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\ntokio = \"1\"\n";
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        let deps = doc["dependencies"].as_table_mut().unwrap();
        assert!(deps.remove("serde").is_some());
        let result = doc.to_string();
        assert!(!result.contains("serde"));
        assert!(result.contains("tokio"));
    }

    #[test]
    fn remove_nonexistent_dep_returns_none() {
        let toml = "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n";
        let mut doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        let deps = doc["dependencies"].as_table_mut().unwrap();
        assert!(deps.remove("nonexistent").is_none());
    }

    #[test]
    fn invalid_toml_rejected() {
        let bad = "this is not valid toml [[[";
        assert!(bad.parse::<toml_edit::DocumentMut>().is_err());
    }

    #[test]
    fn duplicate_dep_detected() {
        let toml = "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n";
        let doc = toml.parse::<toml_edit::DocumentMut>().unwrap();
        assert!(doc["dependencies"].get("serde").is_some());
    }
}

/// Mark the toolchain wizard as completed and persist enabled languages.
#[tauri::command]
pub fn complete_wizard(
    enabled_languages: Vec<String>,
    app: AppHandle,
) -> Result<(), String> {
    let existing = load_config(&app);
    let config = Config {
        active_project: existing.active_project,
        wizard_completed: true,
        enabled_languages,
    };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialise config: {}", e))?;
    std::fs::write(config_path(&app), json)
        .map_err(|e| format!("Failed to write config.json: {}", e))
}
