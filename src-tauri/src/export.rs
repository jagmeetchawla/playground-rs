use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::languages::Lang;
use crate::rustic_manifest;
use crate::{content_dir, workspace_dir, ActiveProject};

/// The exact main.rs from the v0.1 CLI playground runner (commit 231314e),
/// with PLAYGROUND_CONTENT env var added for content file support.
const CLI_MAIN_RS: &str = r##"use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, exit};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "playground",
    about = "Run a Rust playground file from src/bin/",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available playgrounds
    #[command(alias = "ls")]
    List,

    /// Print version information
    #[command(alias = "v")]
    Version,

    /// Run a playground by name (default when no subcommand given)
    #[command(external_subcommand)]
    Run(Vec<String>),
}

fn list_playgrounds() -> Vec<String> {
    let dir = PathBuf::from("src/bin");
    if !dir.exists() {
        return vec![];
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.extension()?.to_str()? == "rs" {
                path.file_stem()?.to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

fn print_list() {
    let playgrounds = list_playgrounds();
    if playgrounds.is_empty() {
        println!("No playgrounds found. Add a .rs file to src/bin/.");
    } else {
        println!("Available playgrounds:\n");
        for name in &playgrounds {
            println!("  {}", name);
        }
    }
}

fn pick_playground() -> String {
    let playgrounds = list_playgrounds();

    if playgrounds.is_empty() {
        eprintln!("No playgrounds found. Add a .rs file to src/bin/.");
        exit(1);
    }

    println!("Available playgrounds:\n");
    for (i, name) in playgrounds.iter().enumerate() {
        println!("  [{}] {}", i + 1, name);
    }

    loop {
        print!("\nPick a playground: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if let Ok(n) = input.parse::<usize>() {
            if n >= 1 && n <= playgrounds.len() {
                return playgrounds[n - 1].clone();
            }
        } else if playgrounds.contains(&input.to_string()) {
            return input.to_string();
        }

        eprintln!("Invalid choice. Enter a number (1-{}) or a name.", playgrounds.len());
    }
}

fn run_playground(name: &str, args: &[String]) {
    let bin_path = PathBuf::from("src/bin").join(format!("{}.rs", name));
    if !bin_path.exists() {
        eprintln!("Error: playground `{}` not found (looked for {})", name, bin_path.display());
        exit(1);
    }

    // Set PLAYGROUND_CONTENT so playgrounds can find their content files
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let content_path = std::path::Path::new(&manifest_dir).join("content");

    let status = Command::new("cargo")
        .args(["run", "--bin", name, "--"])
        .args(args)
        .env("PLAYGROUND_CONTENT", &content_path)
        .status()
        .expect("failed to invoke cargo");

    exit(status.code().unwrap_or(0));
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let name = pick_playground();
            run_playground(&name, &[]);
        }
        Some(Commands::List) => print_list(),
        Some(Commands::Version) => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
        Some(Commands::Run(args)) => {
            run_playground(&args[0], &args[1..]);
        }
    }
}
"##;

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── CLI_MAIN_RS integrity ────────────────────────────────────────────

    #[test]
    fn cli_main_rs_contains_fn_main() {
        assert!(CLI_MAIN_RS.contains("fn main()"));
    }

    #[test]
    fn cli_main_rs_contains_clap_derive() {
        assert!(CLI_MAIN_RS.contains("use clap::{Parser, Subcommand}"));
    }

    #[test]
    fn cli_main_rs_contains_playground_content_env() {
        assert!(CLI_MAIN_RS.contains("PLAYGROUND_CONTENT"));
    }

    #[test]
    fn cli_main_rs_contains_all_subcommands() {
        assert!(CLI_MAIN_RS.contains("List"));
        assert!(CLI_MAIN_RS.contains("Version"));
        assert!(CLI_MAIN_RS.contains("Run(Vec<String>)"));
    }

    #[test]
    fn cli_main_rs_is_nontrivial() {
        // The embedded CLI runner should be substantial
        assert!(
            CLI_MAIN_RS.len() > 1000,
            "CLI_MAIN_RS too short: {} bytes",
            CLI_MAIN_RS.len()
        );
    }

    // ── Export TOML merge logic ──────────────────────────────────────────

    #[test]
    fn export_toml_adds_clap_when_missing() {
        let orig =
            "[package]\nname = \"test\"\nedition = \"2021\"\n\n[dependencies]\nserde = \"1\"\n";
        let doc = orig.parse::<toml_edit::DocumentMut>().unwrap();

        let mut deps_table = doc.get("dependencies").unwrap().clone();
        assert!(deps_table.get("clap").is_none());

        // Simulate the export logic
        let mut clap_table = toml_edit::InlineTable::new();
        clap_table.insert("version", "4".into());
        let mut features = toml_edit::Array::new();
        features.push("derive");
        clap_table.insert("features", toml_edit::Value::Array(features));
        deps_table["clap"] = toml_edit::Item::Value(toml_edit::Value::InlineTable(clap_table));

        assert!(deps_table.get("clap").is_some());
        assert!(deps_table.get("serde").is_some()); // original dep preserved
    }

    #[test]
    fn export_toml_preserves_clap_if_present() {
        let orig = "[package]\nname = \"test\"\n\n[dependencies]\nclap = \"3\"\n";
        let doc = orig.parse::<toml_edit::DocumentMut>().unwrap();
        let deps = doc.get("dependencies").unwrap();
        assert!(deps.get("clap").is_some());
        // When clap is already present, we skip adding it
        assert_eq!(deps.get("clap").unwrap().as_str().unwrap(), "3");
    }

    #[test]
    fn export_toml_creates_valid_document() {
        let project_name = "my_export";
        let mut export_toml = toml_edit::DocumentMut::new();
        export_toml["package"] = toml_edit::Item::Table(toml_edit::Table::new());
        export_toml["package"]["name"] = toml_edit::value(project_name);
        export_toml["package"]["version"] = toml_edit::value("0.1.0");
        export_toml["package"]["edition"] = toml_edit::value("2021");
        export_toml["package"]["default-run"] = toml_edit::value(project_name);
        export_toml["dependencies"] = toml_edit::Item::Table(toml_edit::Table::new());

        let output = export_toml.to_string();
        // Verify it round-trips
        let reparsed = output.parse::<toml_edit::DocumentMut>();
        assert!(reparsed.is_ok(), "Export TOML failed to reparse");
        let reparsed = reparsed.unwrap();
        assert_eq!(reparsed["package"]["name"].as_str().unwrap(), "my_export");
        assert_eq!(
            reparsed["package"]["default-run"].as_str().unwrap(),
            "my_export"
        );
    }

    // ── Filesystem export (integration test with tempdir) ────────────────

    #[test]
    fn export_creates_directory_structure() {
        let tmp = tempfile::tempdir().unwrap();
        let project_name = "test_project";
        let export_dir = tmp.path().join(project_name);

        let export_src = export_dir.join("src");
        let export_bin = export_src.join("bin");
        std::fs::create_dir_all(&export_bin).unwrap();

        // Write main.rs
        std::fs::write(export_src.join("main.rs"), CLI_MAIN_RS).unwrap();

        // Write a playground
        std::fs::write(
            export_bin.join("hello.rs"),
            "fn main() { println!(\"hello\"); }\n",
        )
        .unwrap();

        // Write Cargo.toml
        let toml = format!(
            "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
            project_name
        );
        std::fs::write(export_dir.join("Cargo.toml"), &toml).unwrap();

        // Verify structure
        assert!(export_dir.join("Cargo.toml").exists());
        assert!(export_dir.join("src/main.rs").exists());
        assert!(export_dir.join("src/bin/hello.rs").exists());

        // Verify main.rs content
        let main_content = std::fs::read_to_string(export_dir.join("src/main.rs")).unwrap();
        assert!(main_content.contains("fn main()"));
    }
}

/// Shell script CLI runner for native C/C++ projects.
/// Mirrors the Rust CLI runner: list, pick, compile & run.
const CLI_PLAYGROUND_SH: &str = r##"#!/bin/sh
# playground — CLI runner for C/C++ playground files
# Exported from Rustic Playground

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$SCRIPT_DIR/src"
BUILD_DIR="$SCRIPT_DIR/build"
CONTENT_DIR="$SCRIPT_DIR/content"

# Source build flags
if [ -f "$SCRIPT_DIR/flags.sh" ]; then
    . "$SCRIPT_DIR/flags.sh"
fi

mkdir -p "$BUILD_DIR"

list_playgrounds() {
    for f in "$SRC_DIR"/*.c "$SRC_DIR"/*.cpp; do
        [ -f "$f" ] && basename "$f"
    done | sort
}

compile_and_run() {
    src="$SRC_DIR/$1"
    name="${1%.*}"
    ext="${1##*.}"
    out="$BUILD_DIR/$name"
    shift

    if [ ! -f "$src" ]; then
        echo "Error: playground '$(basename "$src")' not found (looked for $src)" >&2
        exit 1
    fi

    if [ "$ext" = "cpp" ]; then
        compiler="clang++"
        flags="${CXXFLAGS:-}"
    else
        compiler="clang"
        flags="${CFLAGS:-}"
    fi

    echo "Compiling $(basename "$src")..."
    "$compiler" "$src" -o "$out" $flags
    echo ""

    PLAYGROUND_CONTENT="$CONTENT_DIR" "$out" "$@"
}

pick_playground() {
    count=0
    for f in "$SRC_DIR"/*.c "$SRC_DIR"/*.cpp; do
        [ -f "$f" ] || continue
        count=$((count + 1))
        eval "file_$count=\"$(basename "$f")\""
    done

    if [ "$count" -eq 0 ]; then
        echo "No playgrounds found. Add a .c or .cpp file to src/."
        exit 1
    fi

    echo "Available playgrounds:"
    echo ""
    i=1
    while [ "$i" -le "$count" ]; do
        eval "printf '  [%d] %s\\n' $i \"\$file_$i\""
        i=$((i + 1))
    done

    while true; do
        printf "\nPick a playground: "
        read -r input
        case "$input" in
            *[!0-9]*|"") ;;
            *)
                if [ "$input" -ge 1 ] 2>/dev/null && [ "$input" -le "$count" ] 2>/dev/null; then
                    eval "chosen=\"\$file_$input\""
                    compile_and_run "$chosen"
                    return
                fi
                ;;
        esac
        i=1
        while [ "$i" -le "$count" ]; do
            eval "f=\"\$file_$i\""
            fname="${f%.*}"
            if [ "$input" = "$f" ] || [ "$input" = "$fname" ]; then
                compile_and_run "$f"
                return
            fi
            i=$((i + 1))
        done
        echo "Invalid choice. Enter a number (1-$count) or a filename."
    done
}

case "${1:-}" in
    list|ls)
        list_playgrounds
        ;;
    version|v)
        echo "playground 0.1.0"
        ;;
    help|--help|-h)
        echo "Usage: ./playground.sh [command|filename]"
        echo ""
        echo "Commands:"
        echo "  (none)         Interactive picker"
        echo "  <file.c>       Compile and run a specific file"
        echo "  list, ls       List all playground files"
        echo "  version, v     Print version"
        echo "  help           Show this help"
        ;;
    "")
        pick_playground
        ;;
    *)
        # Direct file — try exact, then with extension
        if [ -f "$SRC_DIR/$1" ]; then
            compile_and_run "$@"
        elif [ -f "$SRC_DIR/$1.c" ]; then
            arg="$1.c"; shift
            compile_and_run "$arg" "$@"
        elif [ -f "$SRC_DIR/$1.cpp" ]; then
            arg="$1.cpp"; shift
            compile_and_run "$arg" "$@"
        else
            echo "Error: playground '$1' not found" >&2
            exit 1
        fi
        ;;
esac
"##;

/// Export the active project. Routes to per-language export via Lang dispatch.
#[tauri::command]
pub fn export_project(dest: String, app: AppHandle) -> Result<String, String> {
    let workspace = workspace_dir(&app);
    let ptype = rustic_manifest::detect_project_type(&workspace);
    let lang = Lang::from_str(&ptype);
    match lang {
        Lang::Rust => export_rust_project(dest, &app),
        Lang::Native => export_native_project(dest, &app),
        Lang::Zig => export_zig_project(dest, &app),
        Lang::Swift => export_swift_project(dest, &app),
    }
}

/// Export a Rust project as a standalone CLI playground (v0.1 CLI style).
fn export_rust_project(dest: String, app: &AppHandle) -> Result<String, String> {
    let workspace = workspace_dir(app);
    let project_name = app.state::<ActiveProject>().0.lock().unwrap().clone();

    let export_dir = PathBuf::from(&dest).join(&project_name);
    let export_src = export_dir.join("src");
    let export_bin = export_src.join("bin");
    std::fs::create_dir_all(&export_bin).map_err(|e| e.to_string())?;

    // Read original Cargo.toml for edition and deps
    let orig_toml =
        std::fs::read_to_string(workspace.join("Cargo.toml")).map_err(|e| e.to_string())?;
    let doc = orig_toml
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| format!("Failed to parse Cargo.toml: {}", e))?;

    let edition = doc
        .get("package")
        .and_then(|p| p.get("edition"))
        .and_then(|e| e.as_str())
        .unwrap_or("2021");

    // Build merged Cargo.toml: project deps + clap + default-run
    let mut export_toml = toml_edit::DocumentMut::new();
    export_toml["package"] = toml_edit::Item::Table(toml_edit::Table::new());
    export_toml["package"]["name"] = toml_edit::value(&project_name);
    export_toml["package"]["version"] = toml_edit::value("0.1.0");
    export_toml["package"]["edition"] = toml_edit::value(edition);
    export_toml["package"]["default-run"] = toml_edit::value(&project_name);

    // Merge deps: start with original, ensure clap is present
    let mut deps_table = if let Some(deps) = doc.get("dependencies") {
        deps.clone()
    } else {
        toml_edit::Item::Table(toml_edit::Table::new())
    };
    if deps_table.get("clap").is_none() {
        let mut clap_table = toml_edit::InlineTable::new();
        clap_table.insert("version", "4".into());
        let mut features = toml_edit::Array::new();
        features.push("derive");
        clap_table.insert("features", toml_edit::Value::Array(features));
        deps_table["clap"] = toml_edit::Item::Value(toml_edit::Value::InlineTable(clap_table));
    }
    export_toml["dependencies"] = deps_table;

    std::fs::write(export_dir.join("Cargo.toml"), export_toml.to_string())
        .map_err(|e| e.to_string())?;

    // Write the CLI main.rs (the v0.1 runner with clap)
    std::fs::write(export_src.join("main.rs"), CLI_MAIN_RS).map_err(|e| e.to_string())?;

    // Copy all playground files from src/bin/
    let bin_src = workspace.join("src").join("bin");
    if bin_src.exists() {
        for entry in std::fs::read_dir(&bin_src)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
        {
            std::fs::copy(entry.path(), export_bin.join(entry.file_name()))
                .map_err(|e| e.to_string())?;
        }
    }

    // Copy content files if they exist
    copy_content_files(app, &export_dir)?;

    // Write README
    let readme = format!(
        "# {project_name}\n\n\
        Exported from Rustic Playground.\n\n\
        ## Usage\n\n\
        ```bash\n\
        # Interactive picker\n\
        cargo run\n\n\
        # Run a specific playground\n\
        cargo run -- <name>\n\n\
        # List all playgrounds\n\
        cargo run -- list\n\
        ```\n\n\
        ## Requirements\n\n\
        - [Rust](https://rustup.rs/) (cargo, rustc)\n"
    );
    std::fs::write(export_dir.join("README.md"), readme).map_err(|e| e.to_string())?;

    Ok(export_dir.to_string_lossy().to_string())
}

/// Export a native C/C++ project as a standalone CLI playground.
/// Creates dest/<project>/ with playground.sh runner, Makefile, src/*.c|cpp,
/// and content/ files.
fn export_native_project(dest: String, app: &AppHandle) -> Result<String, String> {
    let workspace = workspace_dir(app);
    let project_name = app.state::<ActiveProject>().0.lock().unwrap().clone();

    let export_dir = PathBuf::from(&dest).join(&project_name);
    let export_src = export_dir.join("src");
    std::fs::create_dir_all(&export_src).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(export_dir.join("build")).map_err(|e| e.to_string())?;

    // Read build flags from manifest
    let manifest = rustic_manifest::ensure_manifest(&workspace)
        .unwrap_or_else(|_| rustic_manifest::new_native_manifest());
    let cflags = manifest.build.cflags.join(" ");
    let cxxflags = manifest.build.cxxflags.join(" ");

    // Write flags.sh — sourced by playground.sh
    let flags_sh = format!(
        "# Build flags exported from Rustic Playground\nexport CFLAGS=\"{cflags}\"\nexport CXXFLAGS=\"{cxxflags}\"\n"
    );
    std::fs::write(export_dir.join("flags.sh"), flags_sh).map_err(|e| e.to_string())?;

    // Write Makefile
    let makefile = generate_makefile(&workspace, &cflags, &cxxflags)?;
    std::fs::write(export_dir.join("Makefile"), makefile).map_err(|e| e.to_string())?;

    // Write the CLI runner script
    std::fs::write(export_dir.join("playground.sh"), CLI_PLAYGROUND_SH)
        .map_err(|e| e.to_string())?;
    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(export_dir.join("playground.sh"), perms)
            .map_err(|e| e.to_string())?;
    }

    // Copy all .c and .cpp source files
    for entry in std::fs::read_dir(&workspace)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "c" || ext == "cpp")
                .unwrap_or(false)
        })
    {
        std::fs::copy(entry.path(), export_src.join(entry.file_name()))
            .map_err(|e| e.to_string())?;
    }

    // Copy content files if they exist
    copy_content_files(app, &export_dir)?;

    // Write a README
    let readme = format!(
        "# {project_name}\n\n\
        Exported from Rustic Playground.\n\n\
        ## Usage\n\n\
        ```bash\n\
        # Interactive picker\n\
        ./playground.sh\n\n\
        # Run a specific file\n\
        ./playground.sh hello.c\n\n\
        # List all playgrounds\n\
        ./playground.sh list\n\n\
        # Build all with make\n\
        make\n\
        make clean\n\
        ```\n\n\
        ## Build Flags\n\n\
        C flags: `{cflags}`\n\
        C++ flags: `{cxxflags}`\n\n\
        Edit `flags.sh` to change compiler flags.\n"
    );
    std::fs::write(export_dir.join("README.md"), readme).map_err(|e| e.to_string())?;

    Ok(export_dir.to_string_lossy().to_string())
}

fn generate_makefile(
    workspace: &std::path::Path,
    cflags: &str,
    cxxflags: &str,
) -> Result<String, String> {
    let mut c_files = Vec::new();
    let mut cpp_files = Vec::new();

    for entry in std::fs::read_dir(workspace)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        match path.extension().and_then(|e| e.to_str()) {
            Some("c") => {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    c_files.push(stem.to_string());
                }
            }
            Some("cpp") => {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    cpp_files.push(stem.to_string());
                }
            }
            _ => {}
        }
    }

    c_files.sort();
    cpp_files.sort();

    let all_targets: Vec<String> = c_files
        .iter()
        .chain(cpp_files.iter())
        .map(|s| format!("build/{s}"))
        .collect();

    let mut mk = String::new();
    mk.push_str("# Makefile — exported from Rustic Playground\n\n");
    mk.push_str(&format!("CFLAGS   = {cflags}\n"));
    mk.push_str(&format!("CXXFLAGS = {cxxflags}\n"));
    mk.push_str("CONTENT  = $(CURDIR)/content\n\n");
    mk.push_str(&format!("all: {}\n\n", all_targets.join(" ")));

    for name in &c_files {
        mk.push_str(&format!(
            "build/{name}: src/{name}.c\n\t@mkdir -p build\n\tclang $(CFLAGS) $< -o $@\n\n"
        ));
    }
    for name in &cpp_files {
        mk.push_str(&format!(
            "build/{name}: src/{name}.cpp\n\t@mkdir -p build\n\tclang++ $(CXXFLAGS) $< -o $@\n\n"
        ));
    }

    mk.push_str("clean:\n\trm -rf build\n\n");
    mk.push_str(".PHONY: all clean\n");

    Ok(mk)
}

/// Shell script CLI runner for Zig playground projects.
const CLI_ZIG_SH: &str = r##"#!/bin/sh
# playground — CLI runner for Zig playground files
# Exported from Rustic Playground

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$SCRIPT_DIR/src"
CONTENT_DIR="$SCRIPT_DIR/content"

list_playgrounds() {
    for f in "$SRC_DIR"/*.zig; do
        [ -f "$f" ] && basename "$f"
    done | sort
}

run_playground() {
    src="$SRC_DIR/$1"
    shift

    if [ ! -f "$src" ]; then
        echo "Error: playground '$(basename "$src")' not found (looked for $src)" >&2
        exit 1
    fi

    echo "Running $(basename "$src")..."
    echo ""
    PLAYGROUND_CONTENT="$CONTENT_DIR" zig run "$src" "$@"
}

pick_playground() {
    count=0
    for f in "$SRC_DIR"/*.zig; do
        [ -f "$f" ] || continue
        count=$((count + 1))
        eval "file_$count=\"$(basename "$f")\""
    done

    if [ "$count" -eq 0 ]; then
        echo "No playgrounds found. Add a .zig file to src/."
        exit 1
    fi

    echo "Available playgrounds:"
    echo ""
    i=1
    while [ "$i" -le "$count" ]; do
        eval "printf '  [%d] %s\\n' $i \"\$file_$i\""
        i=$((i + 1))
    done

    while true; do
        printf "\nPick a playground: "
        read -r input
        case "$input" in
            *[!0-9]*|"") ;;
            *)
                if [ "$input" -ge 1 ] 2>/dev/null && [ "$input" -le "$count" ] 2>/dev/null; then
                    eval "chosen=\"\$file_$input\""
                    run_playground "$chosen"
                    return
                fi
                ;;
        esac
        i=1
        while [ "$i" -le "$count" ]; do
            eval "f=\"\$file_$i\""
            fname="${f%.*}"
            if [ "$input" = "$f" ] || [ "$input" = "$fname" ]; then
                run_playground "$f"
                return
            fi
            i=$((i + 1))
        done
        echo "Invalid choice. Enter a number (1-$count) or a filename."
    done
}

case "${1:-}" in
    list|ls)
        list_playgrounds
        ;;
    version|v)
        echo "playground 0.1.0"
        ;;
    help|--help|-h)
        echo "Usage: ./playground.sh [command|filename]"
        echo ""
        echo "Commands:"
        echo "  (none)         Interactive picker"
        echo "  <file.zig>     Run a specific file"
        echo "  list, ls       List all playground files"
        echo "  version, v     Print version"
        echo "  help           Show this help"
        ;;
    "")
        pick_playground
        ;;
    *)
        # Direct file — try exact, then with extension
        if [ -f "$SRC_DIR/$1" ]; then
            run_playground "$@"
        elif [ -f "$SRC_DIR/$1.zig" ]; then
            arg="$1.zig"; shift
            run_playground "$arg" "$@"
        else
            echo "Error: playground '$1' not found" >&2
            exit 1
        fi
        ;;
esac
"##;

/// Export a Zig project as a standalone CLI playground.
/// Creates dest/<project>/ with playground.sh runner, src/*.zig, and content/ files.
fn export_zig_project(dest: String, app: &AppHandle) -> Result<String, String> {
    let workspace = workspace_dir(app);
    let project_name = app.state::<ActiveProject>().0.lock().unwrap().clone();

    let export_dir = PathBuf::from(&dest).join(&project_name);
    let export_src = export_dir.join("src");
    std::fs::create_dir_all(&export_src).map_err(|e| e.to_string())?;

    // Write the CLI runner script
    std::fs::write(export_dir.join("playground.sh"), CLI_ZIG_SH).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(export_dir.join("playground.sh"), perms)
            .map_err(|e| e.to_string())?;
    }

    // Copy all .zig source files
    for entry in std::fs::read_dir(&workspace)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "zig")
                .unwrap_or(false)
        })
    {
        std::fs::copy(entry.path(), export_src.join(entry.file_name()))
            .map_err(|e| e.to_string())?;
    }

    // Copy content files if they exist
    copy_content_files(app, &export_dir)?;

    // Write a README
    let readme = format!(
        "# {project_name}\n\n\
        Exported from Rustic Playground.\n\n\
        ## Usage\n\n\
        ```bash\n\
        # Interactive picker\n\
        ./playground.sh\n\n\
        # Run a specific file\n\
        ./playground.sh hello.zig\n\n\
        # List all playgrounds\n\
        ./playground.sh list\n\
        ```\n\n\
        ## Requirements\n\n\
        - [Zig](https://ziglang.org/download/) (zig compiler)\n"
    );
    std::fs::write(export_dir.join("README.md"), readme).map_err(|e| e.to_string())?;

    Ok(export_dir.to_string_lossy().to_string())
}

/// Shell script CLI runner for Swift playground projects.
const CLI_SWIFT_SH: &str = r##"#!/bin/sh
# playground — CLI runner for Swift playground files
# Exported from Rustic Playground

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$SCRIPT_DIR/src"
BUILD_DIR="$SCRIPT_DIR/build"
CONTENT_DIR="$SCRIPT_DIR/content"

# Source build flags
if [ -f "$SCRIPT_DIR/flags.sh" ]; then
    . "$SCRIPT_DIR/flags.sh"
fi

mkdir -p "$BUILD_DIR"

list_playgrounds() {
    for f in "$SRC_DIR"/*.swift; do
        [ -f "$f" ] && basename "$f"
    done | sort
}

compile_and_run() {
    src="$SRC_DIR/$1"
    name="${1%.*}"
    out="$BUILD_DIR/$name"
    shift

    if [ ! -f "$src" ]; then
        echo "Error: playground '$(basename "$src")' not found (looked for $src)" >&2
        exit 1
    fi

    flags="${SWIFTFLAGS:-}"

    echo "Compiling $(basename "$src")..."
    swiftc "$src" -o "$out" $flags
    echo ""

    PLAYGROUND_CONTENT="$CONTENT_DIR" "$out" "$@"
}

pick_playground() {
    count=0
    for f in "$SRC_DIR"/*.swift; do
        [ -f "$f" ] || continue
        count=$((count + 1))
        eval "file_$count=\"$(basename "$f")\""
    done

    if [ "$count" -eq 0 ]; then
        echo "No playgrounds found. Add a .swift file to src/."
        exit 1
    fi

    echo "Available playgrounds:"
    echo ""
    i=1
    while [ "$i" -le "$count" ]; do
        eval "printf '  [%d] %s\\n' $i \"\$file_$i\""
        i=$((i + 1))
    done

    while true; do
        printf "\nPick a playground: "
        read -r input
        case "$input" in
            *[!0-9]*|"") ;;
            *)
                if [ "$input" -ge 1 ] 2>/dev/null && [ "$input" -le "$count" ] 2>/dev/null; then
                    eval "chosen=\"\$file_$input\""
                    compile_and_run "$chosen"
                    return
                fi
                ;;
        esac
        i=1
        while [ "$i" -le "$count" ]; do
            eval "f=\"\$file_$i\""
            fname="${f%.*}"
            if [ "$input" = "$f" ] || [ "$input" = "$fname" ]; then
                compile_and_run "$f"
                return
            fi
            i=$((i + 1))
        done
        echo "Invalid choice. Enter a number (1-$count) or a filename."
    done
}

case "${1:-}" in
    list|ls)
        list_playgrounds
        ;;
    version|v)
        echo "playground 0.1.0"
        ;;
    help|--help|-h)
        echo "Usage: ./playground.sh [command|filename]"
        echo ""
        echo "Commands:"
        echo "  (none)           Interactive picker"
        echo "  <file.swift>     Compile and run a specific file"
        echo "  list, ls         List all playground files"
        echo "  version, v       Print version"
        echo "  help             Show this help"
        ;;
    "")
        pick_playground
        ;;
    *)
        # Direct file — try exact, then with extension
        if [ -f "$SRC_DIR/$1" ]; then
            compile_and_run "$@"
        elif [ -f "$SRC_DIR/$1.swift" ]; then
            arg="$1.swift"; shift
            compile_and_run "$arg" "$@"
        else
            echo "Error: playground '$1' not found" >&2
            exit 1
        fi
        ;;
esac
"##;

/// Export a Swift project as a standalone CLI playground.
fn export_swift_project(dest: String, app: &AppHandle) -> Result<String, String> {
    let workspace = workspace_dir(app);
    let project_name = app.state::<ActiveProject>().0.lock().unwrap().clone();

    let export_dir = PathBuf::from(&dest).join(&project_name);
    let export_src = export_dir.join("src");
    std::fs::create_dir_all(&export_src).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(export_dir.join("build")).map_err(|e| e.to_string())?;

    // Read build flags from manifest
    let manifest = rustic_manifest::ensure_manifest(&workspace)
        .unwrap_or_else(|_| rustic_manifest::new_swift_manifest());
    let swiftflags = manifest.build.swiftflags.join(" ");

    // Write flags.sh
    let flags_sh = format!(
        "# Build flags exported from Rustic Playground\nexport SWIFTFLAGS=\"{swiftflags}\"\n"
    );
    std::fs::write(export_dir.join("flags.sh"), flags_sh).map_err(|e| e.to_string())?;

    // Write the CLI runner script
    std::fs::write(export_dir.join("playground.sh"), CLI_SWIFT_SH).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(export_dir.join("playground.sh"), perms)
            .map_err(|e| e.to_string())?;
    }

    // Copy all .swift source files
    for entry in std::fs::read_dir(&workspace)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "swift")
                .unwrap_or(false)
        })
    {
        std::fs::copy(entry.path(), export_src.join(entry.file_name()))
            .map_err(|e| e.to_string())?;
    }

    // Copy content files
    copy_content_files(app, &export_dir)?;

    // Write README
    let readme = format!(
        "# {project_name}\n\n\
        Exported from Rustic Playground.\n\n\
        ## Usage\n\n\
        ```bash\n\
        # Interactive picker\n\
        ./playground.sh\n\n\
        # Run a specific file\n\
        ./playground.sh hello.swift\n\n\
        # List all playgrounds\n\
        ./playground.sh list\n\n\
        # Build flags\n\
        # Edit flags.sh to change compiler flags\n\
        ```\n\n\
        ## Requirements\n\n\
        - Swift compiler (`swiftc`) — ships with Xcode Command Line Tools\n\
        - Install: `xcode-select --install`\n"
    );
    std::fs::write(export_dir.join("README.md"), readme).map_err(|e| e.to_string())?;

    Ok(export_dir.to_string_lossy().to_string())
}

fn copy_content_files(app: &AppHandle, export_dir: &std::path::Path) -> Result<(), String> {
    let content_src = content_dir(app);
    if content_src.exists() {
        let content_dest = export_dir.join("content");
        if let Ok(entries) = std::fs::read_dir(&content_src) {
            for entry in entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
            {
                std::fs::create_dir_all(&content_dest).map_err(|e| e.to_string())?;
                std::fs::copy(entry.path(), content_dest.join(entry.file_name()))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
