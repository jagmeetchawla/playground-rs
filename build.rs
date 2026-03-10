use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&manifest_dir).join("src");

    let mut names: Vec<String> = fs::read_dir(&src_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|f| f.ends_with("_playground.rs"))
        .map(|f| f.replace("_playground.rs", ""))
        .collect();

    names.sort();

    let list = names.join(", ");
    let generated =
        format!("// @generated — do not edit, updated by build.rs\nplaygrounds!({list});\n");

    let out = src_dir.join("_playgrounds.rs");
    fs::write(&out, generated).unwrap();

    // Re-run if any playground file is added or removed
    println!("cargo:rerun-if-changed=src/");
}
