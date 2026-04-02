use tauri::AppHandle;

use crate::{content_dir, is_text_file, safe_content_path, validate_filename, ContentFile};

#[tauri::command]
pub fn list_content_files(app: AppHandle) -> Result<Vec<ContentFile>, String> {
    let dir = content_dir(&app);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut files: Vec<ContentFile> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| {
            let filename = e.file_name().to_string_lossy().to_string();
            let size_bytes = e.metadata().map(|m| m.len()).unwrap_or(0);
            let is_text = is_text_file(&filename);
            ContentFile {
                filename,
                size_bytes,
                is_text,
            }
        })
        .collect();
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(files)
}

#[tauri::command]
pub fn create_content_file(filename: String, app: AppHandle) -> Result<(), String> {
    let path = safe_content_path(&filename, &app)?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    if path.exists() {
        return Err(format!("'{}' already exists", filename));
    }
    std::fs::write(&path, "").map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_content_file(filename: String, app: AppHandle) -> Result<String, String> {
    let path = safe_content_path(&filename, &app)?;
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_content_file(filename: String, content: String, app: AppHandle) -> Result<(), String> {
    let path = safe_content_path(&filename, &app)?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_content_file(filename: String, app: AppHandle) -> Result<(), String> {
    let path = safe_content_path(&filename, &app)?;
    std::fs::remove_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_content_file(
    old_filename: String,
    new_filename: String,
    app: AppHandle,
) -> Result<(), String> {
    let old_path = safe_content_path(&old_filename, &app)?;
    let new_path = safe_content_path(&new_filename, &app)?;
    if new_path.exists() {
        return Err(format!("'{}' already exists", new_filename));
    }
    std::fs::rename(&old_path, &new_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_content_file(src_path: String, app: AppHandle) -> Result<String, String> {
    let src = std::path::Path::new(&src_path);
    let filename = src
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| "Invalid source path".to_string())?
        .to_string();
    validate_filename(&filename)?;

    let dir = content_dir(&app);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Resolve name collision with _1, _2, … suffix
    let mut final_name = filename.clone();
    let mut counter = 1u32;
    while dir.join(&final_name).exists() {
        let stem = std::path::Path::new(&filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&filename);
        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        final_name = format!("{}_{}{}", stem, counter, ext);
        counter += 1;
    }

    let dst = dir.join(&final_name);
    std::fs::copy(src, &dst).map_err(|e| e.to_string())?;
    Ok(final_name)
}

#[tauri::command]
pub fn reveal_in_finder(path: String) -> Result<(), String> {
    std::process::Command::new("open")
        .args(["-R", &path])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_content_file_path(filename: String, app: AppHandle) -> Result<String, String> {
    let path = safe_content_path(&filename, &app)?;
    Ok(path.to_string_lossy().to_string())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    /// Test the name collision resolution logic used in import_content_file.
    /// This is isolated from the Tauri command to test the pure algorithm.
    fn resolve_collision(filename: &str, existing: &[&str]) -> String {
        let mut final_name = filename.to_string();
        let mut counter = 1u32;
        while existing.contains(&final_name.as_str())
            || (counter > 1 && existing.contains(&final_name.as_str()))
        {
            let stem = std::path::Path::new(filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(filename);
            let ext = std::path::Path::new(filename)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e))
                .unwrap_or_default();
            final_name = format!("{}_{}{}", stem, counter, ext);
            counter += 1;
        }
        final_name
    }

    #[test]
    fn no_collision_keeps_original_name() {
        assert_eq!(resolve_collision("data.csv", &[]), "data.csv");
    }

    #[test]
    fn collision_adds_suffix() {
        assert_eq!(resolve_collision("data.csv", &["data.csv"]), "data_1.csv");
    }

    #[test]
    fn multiple_collisions_increment() {
        assert_eq!(
            resolve_collision("data.csv", &["data.csv", "data_1.csv"]),
            "data_2.csv"
        );
    }

    #[test]
    fn collision_with_no_extension() {
        assert_eq!(resolve_collision("README", &["README"]), "README_1");
    }

    #[test]
    fn collision_with_dotfile() {
        assert_eq!(
            resolve_collision(".gitignore", &[".gitignore"]),
            ".gitignore_1"
        );
    }
}
