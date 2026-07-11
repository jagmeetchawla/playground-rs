use tauri::AppHandle;

use crate::{cargo_path, config_path, load_config, save_active_toolchain, settings_path, workspace_dir, Config, Settings};

/// Minimum rustc version required for edition = "2024" (stabilized in 1.85,
/// Feb 2025). Generated Cargo.toml files pin this edition, so a toolchain
/// below MIN_RUST compiles user playgrounds into confusing edition errors.
pub const MIN_RUST: (u32, u32, u32) = (1, 85, 0);

/// Parse a rustc/cargo --version line into (major, minor, patch).
///
/// Input shapes seen in the wild:
///   "rustc 1.85.0 (4eb161250 2025-03-15)"
///   "cargo 1.85.0 (d73d2caf9 2024-12-31)"
///   "rustc 1.90.0-nightly (abcdef 2025-06-01)"
///
/// Returns None on any parse failure — callers should default `version_ok` to
/// true in that case, so a parser regression doesn't block healthy users.
fn parse_rust_version(line: &str) -> Option<(u32, u32, u32)> {
    let version_field = line.split_whitespace().nth(1)?;
    let core = version_field.split('-').next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

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

    // Enforce minimum rustc for edition 2024. Parse failures default to
    // version_ok: true so a parser regression never blocks a healthy user.
    let rust_version_ok = rustc_version
        .as_ref()
        .and_then(|v| parse_rust_version(v))
        .map(|v| v >= MIN_RUST)
        .unwrap_or(true);

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

        let output = std::process::Command::new(&path)
            .arg("--version")
            .output()
            .ok();
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
        let output = std::process::Command::new("swiftc")
            .arg("--version")
            .output()
            .ok();
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
    } else if !rust_version_ok {
        "outdated"
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
            "version_ok": rust_version_ok,
            "min_version": format!("{}.{}.{}", MIN_RUST.0, MIN_RUST.1, MIN_RUST.2),
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

// ── Multi-version toolchain support (v0.4+) ──────────────────────────────────

/// Latest Rust stable version this release of Rustic Playground knows about.
///
/// Used by the picker to surface a subtle "install newer stable?" hint when
/// none of the user's installed toolchains — across ANY channel (stable,
/// beta, nightly, or pinned semver) — is at or above this version. Users on
/// nightly, beta, or a newer pinned version are clearly not asleep at the
/// wheel; only the truly-behind get the hint.
///
/// The hint is only shown when the user opens the pill dropdown — never
/// unsolicited.
///
/// Update this constant each release of Rustic Playground to whatever the
/// latest Rust stable is at build time. Rust releases every ~6 weeks, so this
/// will go stale between our releases — that's fine. Staleness just means we
/// don't hint at users who are ahead of us, which is the correct behaviour.
pub const LATEST_KNOWN_STABLE: (u32, u32, u32) = (1, 96, 0);

/// Resolve a rustup-managed tool (rustup, cargo, rustc, rustfmt, cargo-clippy)
/// to an absolute path via the cargo bin dir, with PATH fallback.
///
/// Same logic as the `tool_path` closure inside `check_toolchain`, extracted
/// here so multiple commands can share it. check_toolchain keeps its closure
/// for the cargo_dir-once optimisation across many tools; new commands should
/// use this helper for clarity.
fn resolve_tool(name: &str) -> String {
    let cargo = cargo_path();
    let abs = std::path::Path::new(&cargo)
        .parent()
        .unwrap_or(std::path::Path::new(""))
        .join(name);
    if abs.exists() {
        abs.to_string_lossy().to_string()
    } else {
        name.to_string()
    }
}

/// Summary of one installed Rust toolchain, used by the pill dropdown.
#[derive(serde::Serialize, Debug, Clone)]
pub struct ToolchainInfo {
    /// Full rustup-registered name, e.g. "stable-aarch64-apple-darwin".
    pub name: String,
    /// Human-friendly short form: channel ("stable", "beta", "nightly") or
    /// pinned version ("1.90.0"). Derived from `name` by stripping the host triple.
    pub short_name: String,
    /// Actual rustc semantic version this toolchain resolves to, e.g. "1.96.0".
    /// For pinned toolchains (name starts with a version) we parse from name.
    /// For channels (stable/beta/nightly) we run `rustup run <name> rustc --version`.
    /// None only when the subprocess fails or the name doesn't produce a version.
    pub version: Option<String>,
    /// True when rustup's `list` marks this as "(default)". This is rustup's
    /// fallback when no rust-toolchain.toml or override is present.
    pub is_rustup_default: bool,
    /// True when this toolchain matches Config.active_toolchain — the app's
    /// session-level default used for new-project scaffolding and as fallback
    /// when a project's pinned toolchain isn't installed.
    pub is_active: bool,
}

/// Strip the host triple from a toolchain name to get the channel/version alone.
///
/// Examples:
///   "stable-aarch64-apple-darwin"        → "stable"
///   "1.85.0-x86_64-apple-darwin"         → "1.85.0"
///   "nightly-2024-01-01-x86_64-apple-darwin" → "nightly-2024-01-01"
///
/// Host triples always begin with one of a known set of architecture strings
/// (x86_64, aarch64, i686, armv7, riscv*). Anything preceding the first
/// occurrence of "-<arch>" is the short name.
fn short_name_from_full(full: &str) -> String {
    const ARCH_MARKERS: &[&str] = &["x86_64", "aarch64", "i686", "armv7", "riscv64", "riscv32"];
    for marker in ARCH_MARKERS {
        if let Some(idx) = full.find(&format!("-{}", marker)) {
            return full[..idx].to_string();
        }
    }
    // Fallback: no known arch marker → return as-is. Better than losing info.
    full.to_string()
}

/// List all Rust toolchains rustup knows about, plus which is rustup's default
/// and which matches the app's active toolchain.
///
/// Runs `rustup toolchain list`. Format is one toolchain per line, e.g.:
///     stable-aarch64-apple-darwin (default) (active)
///     nightly-aarch64-apple-darwin
///     1.85.0-aarch64-apple-darwin
///
/// The parenthesised markers can appear in any order. We look for "(default)"
/// to identify rustup's default. We deliberately ignore "(active)" — that's
/// rustup's view of "active for the current shell / directory", which isn't
/// the same concept as our app's tracked active toolchain.
#[tauri::command]
pub fn list_rust_toolchains(app: AppHandle) -> Result<Vec<ToolchainInfo>, String> {
    let rustup_bin = resolve_tool("rustup");
    let output = std::process::Command::new(&rustup_bin)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args(["toolchain", "list"])
        .output()
        .map_err(|e| format!("Failed to run rustup: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "rustup toolchain list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("rustup output was not UTF-8: {}", e))?;

    let active_name = load_config(&app).active_toolchain;

    // Special-case: "no installed toolchains" is a stderr message, but success
    // exit + empty stdout is possible on very clean systems.
    let toolchains: Vec<ToolchainInfo> = stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            // The name is the first whitespace-separated token. Everything else
            // (parenthesised markers) is metadata we inspect on the raw line.
            let name = trimmed.split_whitespace().next()?.to_string();
            let is_default = trimmed.contains("(default)");
            let short_name = short_name_from_full(&name);
            let is_active = active_name
                .as_deref()
                .map(|a| a == name || a == short_name)
                .unwrap_or(false);
            let version = resolve_toolchain_version(&name, &short_name, &rustup_bin);
            Some(ToolchainInfo {
                name,
                short_name,
                version,
                is_rustup_default: is_default,
                is_active,
            })
        })
        .collect();

    Ok(toolchains)
}

/// Find the rustc semver for a given toolchain. Two fast paths + one subprocess:
///   - If short_name is a bare "X.Y.Z" (pinned toolchain), return it directly.
///   - If it starts with "1." followed by a version (dated nightly, e.g.
///     "1.96.0-nightly"), extract the version.
///   - Otherwise (channel names: stable, beta, nightly) shell out to
///     `rustup run <name> rustc --version` and parse the result.
///
/// Returns None if the subprocess fails or parsing doesn't produce a version.
/// A None here doesn't block anything — the picker just won't show a version
/// number next to that toolchain.
fn resolve_toolchain_version(name: &str, short_name: &str, rustup_bin: &str) -> Option<String> {
    // Fast path: pinned version (short_name is exactly "X.Y.Z")
    if short_name.split('.').all(|p| p.chars().all(|c| c.is_ascii_digit())) {
        let parts: Vec<&str> = short_name.split('.').collect();
        if parts.len() == 3 {
            return Some(short_name.to_string());
        }
    }
    // Fallback: ask rustc directly through rustup
    let output = std::process::Command::new(rustup_bin)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args(["run", name, "rustc", "--version"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let (major, minor, patch) = parse_rust_version(stdout.trim())?;
    Some(format!("{}.{}.{}", major, minor, patch))
}

/// Return the version of Rust stable that this release of Rustic Playground
/// knows about, formatted as "X.Y.Z". The frontend picker compares this
/// against installed toolchains to decide whether to show an "install newer
/// stable?" hint.
#[tauri::command]
pub fn get_latest_known_stable() -> String {
    format!(
        "{}.{}.{}",
        LATEST_KNOWN_STABLE.0, LATEST_KNOWN_STABLE.1, LATEST_KNOWN_STABLE.2
    )
}

/// Set the app's session-level active toolchain. Pass None to clear (so we
/// fall back to rustup's default). Persisted to config.json.
///
/// Also written to the currently-active project's rust-toolchain.toml if
/// `apply_to_active_project` is true — this is what happens when the user
/// clicks a toolchain in the pill dropdown while inside a project.
#[tauri::command]
pub fn set_active_toolchain(
    toolchain: Option<String>,
    _apply_to_active_project: bool,
    app: AppHandle,
) -> Result<(), String> {
    // v0.4 minimum: persist the app-level active. Per-project write-through
    // is implemented in the rust-toolchain.toml helper task. For now, this
    // command only updates config; the frontend can invoke the project-write
    // command separately if needed.
    save_active_toolchain(&app, toolchain)
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

    // ── parse_rust_version ───────────────────────────────────────────────

    #[test]
    fn parse_rust_version_stable_rustc() {
        assert_eq!(
            parse_rust_version("rustc 1.85.0 (4eb161250 2025-03-15)"),
            Some((1, 85, 0))
        );
    }

    #[test]
    fn parse_rust_version_stable_cargo() {
        assert_eq!(
            parse_rust_version("cargo 1.85.0 (d73d2caf9 2024-12-31)"),
            Some((1, 85, 0))
        );
    }

    #[test]
    fn parse_rust_version_1_90_stable() {
        assert_eq!(
            parse_rust_version("rustc 1.90.0 (abcdef123 2025-09-15)"),
            Some((1, 90, 0))
        );
    }

    #[test]
    fn parse_rust_version_nightly() {
        assert_eq!(
            parse_rust_version("rustc 1.90.0-nightly (abcdef 2025-06-01)"),
            Some((1, 90, 0))
        );
    }

    #[test]
    fn parse_rust_version_beta() {
        assert_eq!(
            parse_rust_version("rustc 1.90.0-beta.1 (abcdef 2025-06-01)"),
            Some((1, 90, 0))
        );
    }

    #[test]
    fn parse_rust_version_high_patch() {
        assert_eq!(
            parse_rust_version("rustc 1.90.15 (abcdef 2026-01-01)"),
            Some((1, 90, 15))
        );
    }

    #[test]
    fn parse_rust_version_three_digit_minor() {
        // Rust's minor version has grown from 1.x — this locks in future-proofing
        assert_eq!(
            parse_rust_version("rustc 1.100.5 (abcdef 2027-01-01)"),
            Some((1, 100, 5))
        );
    }

    #[test]
    fn parse_rust_version_bare_version() {
        // Some rustup outputs don't include the commit hash
        assert_eq!(parse_rust_version("rustc 1.90.0"), Some((1, 90, 0)));
    }

    #[test]
    fn parse_rust_version_garbage_returns_none() {
        assert_eq!(parse_rust_version("this is not a version line"), None);
        assert_eq!(parse_rust_version(""), None);
        assert_eq!(parse_rust_version("rustc"), None);
        assert_eq!(parse_rust_version("rustc 1"), None);
        assert_eq!(parse_rust_version("rustc 1.90"), None); // missing patch
    }

    #[test]
    fn version_tuple_comparison_matches_min_rust() {
        // Lock in the semantic that (major, minor, patch) tuple comparison
        // does what we mean for version gating.
        assert!((1, 90, 0) >= MIN_RUST); // 1.90.0 >= 1.85.0
        assert!((1, 85, 0) >= MIN_RUST); // 1.85.0 == floor, allowed
        assert!((1, 85, 5) >= MIN_RUST); // patch above floor
        assert!(!((1, 84, 999) >= MIN_RUST)); // just below floor
        assert!(!((1, 0, 0) >= MIN_RUST)); // clearly below
    }

    // ── short_name_from_full ─────────────────────────────────────────────

    #[test]
    fn short_name_strips_apple_darwin_arm() {
        assert_eq!(short_name_from_full("stable-aarch64-apple-darwin"), "stable");
    }

    #[test]
    fn short_name_strips_apple_darwin_intel() {
        assert_eq!(short_name_from_full("stable-x86_64-apple-darwin"), "stable");
    }

    #[test]
    fn short_name_pinned_version() {
        assert_eq!(
            short_name_from_full("1.85.0-aarch64-apple-darwin"),
            "1.85.0"
        );
        assert_eq!(
            short_name_from_full("1.90.0-x86_64-apple-darwin"),
            "1.90.0"
        );
    }

    #[test]
    fn short_name_nightly_channel() {
        assert_eq!(short_name_from_full("nightly-aarch64-apple-darwin"), "nightly");
        assert_eq!(short_name_from_full("beta-x86_64-apple-darwin"), "beta");
    }

    #[test]
    fn short_name_dated_nightly() {
        // Dated nightlies keep the date in the short name
        assert_eq!(
            short_name_from_full("nightly-2026-06-01-aarch64-apple-darwin"),
            "nightly-2026-06-01"
        );
    }

    #[test]
    fn short_name_linux_hosts() {
        // Should also work on Linux even though the app is Mac-only —
        // rust-toolchain.toml files created elsewhere might be opened.
        assert_eq!(
            short_name_from_full("stable-x86_64-unknown-linux-gnu"),
            "stable"
        );
        assert_eq!(
            short_name_from_full("1.85.0-aarch64-unknown-linux-gnu"),
            "1.85.0"
        );
    }

    #[test]
    fn short_name_unrecognised_falls_back_intact() {
        // If we don't recognise the arch marker, return the input intact rather
        // than truncating something meaningful.
        assert_eq!(short_name_from_full("weird-name"), "weird-name");
        assert_eq!(short_name_from_full(""), "");
    }

    // ── read_project_toolchain / write_project_toolchain ─────────────────

    #[test]
    fn read_project_toolchain_none_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_project_toolchain(dir.path()), None);
    }

    #[test]
    fn read_project_toolchain_parses_channel() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"stable\"\n",
        )
        .unwrap();
        assert_eq!(read_project_toolchain(dir.path()), Some("stable".to_string()));
    }

    #[test]
    fn read_project_toolchain_parses_pinned_version() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"1.90.0\"\ncomponents = [\"rustfmt\"]\n",
        )
        .unwrap();
        assert_eq!(read_project_toolchain(dir.path()), Some("1.90.0".to_string()));
    }

    #[test]
    fn read_project_toolchain_none_when_no_channel_field() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("rust-toolchain.toml"),
            "[toolchain]\ncomponents = [\"rustfmt\"]\n",
        )
        .unwrap();
        assert_eq!(read_project_toolchain(dir.path()), None);
    }

    #[test]
    fn read_project_toolchain_none_when_malformed_toml() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("rust-toolchain.toml"),
            "this is not valid toml [[[",
        )
        .unwrap();
        assert_eq!(read_project_toolchain(dir.path()), None);
    }

    #[test]
    fn read_project_toolchain_none_when_empty_channel() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"\"\n",
        )
        .unwrap();
        assert_eq!(read_project_toolchain(dir.path()), None);
    }

    #[test]
    fn write_project_toolchain_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        write_project_toolchain(dir.path(), "stable").unwrap();
        let content = std::fs::read_to_string(dir.path().join("rust-toolchain.toml")).unwrap();
        assert!(content.contains("[toolchain]"));
        assert!(content.contains("channel = \"stable\""));
        // Read back through our own reader
        assert_eq!(read_project_toolchain(dir.path()), Some("stable".to_string()));
    }

    #[test]
    fn write_project_toolchain_preserves_other_fields() {
        let dir = tempfile::tempdir().unwrap();
        // Pre-existing file with components and profile
        std::fs::write(
            dir.path().join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"stable\"\ncomponents = [\"rustfmt\", \"clippy\"]\nprofile = \"minimal\"\n",
        )
        .unwrap();
        // Overwrite channel
        write_project_toolchain(dir.path(), "1.90.0").unwrap();
        let content = std::fs::read_to_string(dir.path().join("rust-toolchain.toml")).unwrap();
        assert!(content.contains("channel = \"1.90.0\""));
        assert!(content.contains("rustfmt")); // preserved
        assert!(content.contains("clippy")); // preserved
        assert!(content.contains("minimal")); // preserved
    }

    // ── wrap_with_rustup ─────────────────────────────────────────────────

    #[test]
    fn wrap_with_rustup_passthrough_when_no_toolchain() {
        let (program, args) = wrap_with_rustup("cargo", &["run".to_string()], None);
        assert_eq!(program, "cargo");
        assert_eq!(args, vec!["run"]);
    }

    #[test]
    fn wrap_with_rustup_prepends_rustup_run() {
        let cargo_args = vec!["run".to_string(), "--bin".to_string(), "hello".to_string()];
        let (program, args) = wrap_with_rustup("/path/to/cargo", &cargo_args, Some("1.90.0"));
        // Program becomes rustup (path resolved via resolve_tool)
        assert!(program.ends_with("rustup"));
        // Args are: run, <toolchain>, <original program>, <original args...>
        assert_eq!(args[0], "run");
        assert_eq!(args[1], "1.90.0");
        assert_eq!(args[2], "/path/to/cargo");
        assert_eq!(&args[3..], &["run", "--bin", "hello"]);
    }
}

/// Mark the toolchain wizard as completed and persist enabled languages.
#[tauri::command]
pub fn complete_wizard(enabled_languages: Vec<String>, app: AppHandle) -> Result<(), String> {
    let existing = load_config(&app);
    let config = Config {
        active_project: existing.active_project,
        wizard_completed: true,
        enabled_languages,
        active_toolchain: existing.active_toolchain,
    };
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialise config: {}", e))?;
    std::fs::write(config_path(&app), json)
        .map_err(|e| format!("Failed to write config.json: {}", e))
}

// ── rust-toolchain.toml helpers (v0.4+) ──────────────────────────────────────

/// Read a project's rust-toolchain.toml and return the [toolchain].channel
/// value if the file exists and is parseable. Returns None if:
///   - The file doesn't exist
///   - The file is unreadable
///   - The TOML is malformed
///   - There's no [toolchain] table or no channel field
///
/// This intentionally swallows all errors — the caller then falls back to
/// the app's active toolchain, which is the correct behaviour when a project
/// has no valid pin.
pub(crate) fn read_project_toolchain(project_path: &std::path::Path) -> Option<String> {
    let toml_path = project_path.join("rust-toolchain.toml");
    let content = std::fs::read_to_string(&toml_path).ok()?;
    let doc: toml_edit::DocumentMut = content.parse().ok()?;
    let channel = doc
        .get("toolchain")?
        .as_table_like()?
        .get("channel")?
        .as_value()?
        .as_str()?
        .to_string();
    if channel.is_empty() {
        None
    } else {
        Some(channel)
    }
}

/// Write a project's rust-toolchain.toml with `[toolchain]\nchannel = "<name>"`.
///
/// If the file already exists with additional fields (components, targets,
/// profile), those are preserved by merging into the existing document.
/// If the file doesn't exist, a minimal one is created.
pub(crate) fn write_project_toolchain(
    project_path: &std::path::Path,
    name: &str,
) -> Result<(), String> {
    let toml_path = project_path.join("rust-toolchain.toml");

    let mut doc: toml_edit::DocumentMut = if toml_path.exists() {
        std::fs::read_to_string(&toml_path)
            .map_err(|e| format!("Failed to read rust-toolchain.toml: {}", e))?
            .parse()
            .map_err(|e| format!("Malformed rust-toolchain.toml: {}", e))?
    } else {
        "[toolchain]\n".parse().unwrap()
    };

    // Ensure [toolchain] table exists
    if !doc.contains_key("toolchain") {
        doc["toolchain"] = toml_edit::Item::Table(toml_edit::Table::new());
    }
    doc["toolchain"]["channel"] = toml_edit::value(name);

    std::fs::write(&toml_path, doc.to_string())
        .map_err(|e| format!("Failed to write rust-toolchain.toml: {}", e))
}

/// Tauri command: read a project's pinned toolchain (if any).
#[tauri::command]
pub fn get_project_toolchain(project_path: String) -> Option<String> {
    read_project_toolchain(std::path::Path::new(&project_path))
}

/// Tauri command: write a project's pinned toolchain.
#[tauri::command]
pub fn set_project_toolchain(project_path: String, name: String) -> Result<(), String> {
    write_project_toolchain(std::path::Path::new(&project_path), &name)
}

/// Tauri command: remove a project's rust-toolchain.toml, unpinning it. No-op
/// if the file doesn't exist. Called from the picker's "Remove pin" action.
///
/// After removal, resolve_toolchain_for_project falls through to the app's
/// active_toolchain — same behaviour as any project without a pin.
#[tauri::command]
pub fn remove_project_toolchain(project_path: String) -> Result<(), String> {
    let toml_path = std::path::Path::new(&project_path).join("rust-toolchain.toml");
    if !toml_path.exists() {
        return Ok(()); // No-op: nothing to remove
    }
    std::fs::remove_file(&toml_path)
        .map_err(|e| format!("Failed to remove rust-toolchain.toml: {}", e))
}

/// Check whether rustup has a given toolchain installed. Uses cached
/// `list_rust_toolchains` output; falls back to false if rustup fails.
pub(crate) fn is_toolchain_installed(name: &str, app: &AppHandle) -> bool {
    list_rust_toolchains(app.clone())
        .map(|list| {
            list.iter()
                .any(|t| t.name == name || t.short_name == name)
        })
        .unwrap_or(false)
}

/// Resolve which Rust toolchain to use for a given project directory.
///
/// Resolution:
///   1. Project's rust-toolchain.toml pin, IF that toolchain is installed
///   2. None → caller falls back to bare `cargo`, which uses rustup's default
///
/// The v0.4-original design had a step 2 that fell back to
/// Config.active_toolchain (updated by every picker click). That coupling
/// meant "pick nightly in project A" leaked into "new project B silently
/// runs on nightly", which was surprising. Removing step 2 makes the model
/// intuitive: the pin is either present-and-honored, or absent-and-let-rustup-decide.
///
/// Returning None is a valid, healthy state — projects without a pin should
/// use rustup's default, same as if you ran `cargo` from the project directory
/// in a terminal.
pub(crate) fn resolve_toolchain_for_project(
    project_path: &std::path::Path,
    app: &AppHandle,
) -> Option<String> {
    if let Some(pinned) = read_project_toolchain(project_path) {
        if is_toolchain_installed(&pinned, app) {
            return Some(pinned);
        }
    }
    None
}

/// Wrap a cargo/rustc command invocation with `rustup run <toolchain>` if a
/// toolchain is provided. Returns (program, args) ready for Command::new.
///
/// Example:
///   wrap_with_rustup("/Users/x/.cargo/bin/cargo",
///                     &["run", "--bin", "hello"], Some("1.90.0"))
///   → ("/Users/x/.cargo/bin/rustup",
///      ["run", "1.90.0", "/Users/x/.cargo/bin/cargo", "run", "--bin", "hello"])
///
/// When toolchain is None, returns the original program + args unchanged.
pub(crate) fn wrap_with_rustup(
    program: &str,
    args: &[String],
    toolchain: Option<&str>,
) -> (String, Vec<String>) {
    match toolchain {
        None => (program.to_string(), args.to_vec()),
        Some(tc) => {
            let rustup = resolve_tool("rustup");
            let mut new_args = vec!["run".to_string(), tc.to_string(), program.to_string()];
            new_args.extend(args.iter().cloned());
            (rustup, new_args)
        }
    }
}
