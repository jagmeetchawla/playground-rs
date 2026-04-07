use std::collections::HashMap;
use tauri::menu::{
    CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::AppHandle;

use crate::languages::Lang;

/// Book source tags → submenu labels (same order as frontend BOOK_LABELS).
const BOOK_LABELS: &[(&str, &str)] = &[
    ("rust_book", "The Rust Programming Language (2021 Edition)"),
    ("knr_book", "The C Programming Language (K&&R)"),
    ("swift_book", "The Swift Programming Language"),
];

/// Builds the full macOS menu bar.  Called once on startup and again via
/// `rebuild_menu` whenever the project list or active project changes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_menu<R: tauri::Runtime>(
    handle: &impl tauri::Manager<R>,
    projects: &[String],
    active: &str,
    _playground_count: usize,
    has_active_playground: bool,
    has_active_tab: bool,
    _project_type: &str,
    is_book_project: bool,
    project_sources: &HashMap<String, String>,
    enabled_languages: &[String],
) -> tauri::Result<tauri::menu::Menu<R>> {
    let product_name = handle.config().product_name.clone().unwrap_or_else(|| "Rustic Playground".to_string());
    let app_submenu = SubmenuBuilder::new(handle, &product_name)
        .item(
            &MenuItemBuilder::with_id("check_update", "Check for Updates…")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("show_settings", "Settings…")
                .accelerator("CmdOrCtrl+,")
                .build(handle)?,
        )
        .separator()
        .item(&PredefinedMenuItem::hide(handle, None)?)
        .item(&PredefinedMenuItem::hide_others(handle, None)?)
        .item(&PredefinedMenuItem::show_all(handle, None)?)
        .separator()
        .item(&PredefinedMenuItem::quit(handle, None)?)
        .build()?;

    // Separate user projects from book projects
    let mut user_projects: Vec<&String> = Vec::new();
    let mut book_groups: HashMap<&str, Vec<&String>> = HashMap::new();
    for name in projects {
        if let Some(source) = project_sources.get(name) {
            book_groups.entry(source.as_str()).or_default().push(name);
        } else {
            user_projects.push(name);
        }
    }

    let new_project_item = MenuItemBuilder::with_id("new_project", "New Project…")
        .accelerator("CmdOrCtrl+Shift+N")
        .build(handle)?;
    let rename_project_item = MenuItemBuilder::with_id("rename_project", "Rename Project…")
        .enabled(!is_book_project)
        .build(handle)?;
    let delete_project_item = MenuItemBuilder::with_id("delete_project", "Delete Project…")
        .enabled(projects.len() > 1 && !is_book_project)
        .build(handle)?;

    let mut proj_builder = SubmenuBuilder::new(handle, "Project")
        .item(&new_project_item)
        .separator();

    // User projects as flat check items
    for name in &user_projects {
        proj_builder = proj_builder.item(
            &CheckMenuItemBuilder::with_id(format!("switch_project::{}", name), name.as_str())
                .checked(name.as_str() == active)
                .build(handle)?,
        );
    }

    // Book projects grouped into submenus
    let has_books = BOOK_LABELS
        .iter()
        .any(|(key, _)| book_groups.contains_key(key));
    if has_books {
        proj_builder = proj_builder.separator();
        for &(key, label) in BOOK_LABELS {
            if let Some(book_projects) = book_groups.get(key) {
                let mut sub = SubmenuBuilder::new(handle, label);
                for name in book_projects {
                    sub = sub.item(
                        &CheckMenuItemBuilder::with_id(
                            format!("switch_project::{}", name),
                            name.as_str(),
                        )
                        .checked(name.as_str() == active)
                        .build(handle)?,
                    );
                }
                proj_builder = proj_builder.item(&sub.build()?);
            }
        }
    }

    let duplicate_project_item =
        MenuItemBuilder::with_id("duplicate_project", "Duplicate Project").build(handle)?;

    let project_menu = proj_builder
        .separator()
        .item(&duplicate_project_item)
        .item(&rename_project_item)
        .item(&delete_project_item)
        .separator()
        .item(
            &MenuItemBuilder::with_id("export_project", "Export Project…")
                .accelerator("CmdOrCtrl+Shift+E")
                .build(handle)?,
        )
        .build()?;

    let playground_menu = SubmenuBuilder::new(handle, "Playground")
        .item(
            &MenuItemBuilder::with_id("new_playground", "New Playground")
                .accelerator("CmdOrCtrl+N")
                .enabled(!is_book_project)
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("save", "Save")
                .accelerator("CmdOrCtrl+S")
                .enabled(has_active_tab && !is_book_project)
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("revert", "Revert to Saved")
                .enabled(has_active_tab && !is_book_project)
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("close_tab", "Close Tab")
                .accelerator("CmdOrCtrl+W")
                .enabled(has_active_tab)
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("copy_code", "Copy Code to Clipboard")
                .accelerator("CmdOrCtrl+Shift+C")
                .enabled(has_active_playground)
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("menu_rename_playground", "Rename Playground…")
                .enabled(has_active_playground && !is_book_project)
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("menu_delete_playground", "Delete Playground…")
                .enabled(has_active_playground && !is_book_project)
                .build(handle)?,
        )
        .build()?;

    let run_menu = SubmenuBuilder::new(handle, "Run")
        .item(
            &MenuItemBuilder::with_id("run_playground", "Run")
                .accelerator("CmdOrCtrl+R")
                .enabled(has_active_playground)
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("stop_playground", "Stop")
                .accelerator("CmdOrCtrl+.")
                .enabled(has_active_playground)
                .build(handle)?,
        )
        .build()?;

    let edit_menu = SubmenuBuilder::new(handle, "Edit")
        .item(&PredefinedMenuItem::undo(handle, None)?)
        .item(&PredefinedMenuItem::redo(handle, None)?)
        .separator()
        .item(&PredefinedMenuItem::cut(handle, None)?)
        .item(&PredefinedMenuItem::copy(handle, None)?)
        .item(&PredefinedMenuItem::paste(handle, None)?)
        .separator()
        .select_all()
        .build()?;

    // ── Per-language menus (replaces old "Learn" menu) ──
    // Single-language editions: one top-level menu named after the language
    //   (e.g. "Rust") with toolchain + book actions inline.
    // Power Edition (multi-language): one top-level "Languages" menu containing
    //   per-language submenus.
    //
    // Each language menu contains (when applicable):
    //   • Rust Toolchain…           (Rust only — opens install/repair wizard)
    //   • rust-lang.org…            (Rust only — opens website)
    //   • rustup.rs…                (Rust only — opens website)
    //   • ─────
    //   • Load Examples…            (book languages)
    //   • Remove Examples           (book languages)
    //   • Read Online…              (book languages with URL)
    let single_language = enabled_languages.len() == 1;

    fn lang_display_name(lang: &Lang) -> &'static str {
        match lang {
            Lang::Rust => "Rust",
            Lang::Clang => "C/C++",
            Lang::Zig => "Zig",
            Lang::Swift => "Swift",
        }
    }

    /// Build a single language's submenu. Returns None if there's nothing to show.
    fn build_lang_submenu<R: tauri::Runtime>(
        handle: &impl tauri::Manager<R>,
        lang_variant: &Lang,
        project_sources: &HashMap<String, String>,
    ) -> tauri::Result<Option<tauri::menu::Submenu<R>>> {
        let mut sub = SubmenuBuilder::new(handle, lang_display_name(lang_variant));
        let mut has_items = false;

        // Toolchain item — Rust only for v0.3.4. Single entry that opens the
        // status/repair modal (the modal self-describes healthy vs needs-fix).
        if matches!(lang_variant, Lang::Rust) {
            sub = sub
                .item(
                    &MenuItemBuilder::with_id("rust_toolchain", "Rust Toolchain…")
                        .build(handle)?,
                )
                .separator()
                .item(
                    &MenuItemBuilder::with_id("open_rust_lang", "rust-lang.org…")
                        .build(handle)?,
                )
                .item(
                    &MenuItemBuilder::with_id("open_rustup", "rustup.rs…")
                        .build(handle)?,
                );
            has_items = true;
        }

        // Book items grouped under a sub-submenu named after the book
        // (matches the structure of the old "Learn" menu).
        if let Some(book) = lang_variant.book_info() {
            if has_items {
                sub = sub.separator();
            }
            let already_loaded = project_sources.values().any(|s| s == book.source_tag);
            let book_submenu = SubmenuBuilder::new(handle, book.book_name)
                .item(
                    &MenuItemBuilder::with_id(book.menu_id, "Load Examples…")
                        .enabled(!already_loaded)
                        .build(handle)?,
                )
                .item(
                    &MenuItemBuilder::with_id(book.remove_menu_id, "Remove Examples")
                        .enabled(already_loaded)
                        .build(handle)?,
                )
                .separator()
                .item(
                    &MenuItemBuilder::with_id(
                        format!("open_book_{}", book.source_tag),
                        "Read Online…",
                    )
                    .build(handle)?,
                )
                .build()?;
            sub = sub.item(&book_submenu);
            has_items = true;
        }

        if has_items {
            Ok(Some(sub.build()?))
        } else {
            Ok(None)
        }
    }

    // Collect submenus for every enabled language that has content.
    let mut lang_submenus: Vec<(&Lang, tauri::menu::Submenu<R>)> = Vec::new();
    for lang_variant in Lang::all() {
        if !enabled_languages.iter().any(|l| l == lang_variant.project_type()) {
            continue;
        }
        if let Some(sm) = build_lang_submenu(handle, lang_variant, project_sources)? {
            lang_submenus.push((lang_variant, sm));
        }
    }

    // Build the top-level language menu (single or grouped under "Languages").
    let language_menu: Option<tauri::menu::Submenu<R>> = if single_language {
        lang_submenus.into_iter().next().map(|(_, sm)| sm)
    } else if lang_submenus.is_empty() {
        None
    } else {
        let mut langs_builder = SubmenuBuilder::new(handle, "Languages");
        for (_, sm) in &lang_submenus {
            langs_builder = langs_builder.item(sm);
        }
        Some(langs_builder.build()?)
    };

    let help_menu = SubmenuBuilder::new(handle, "Help")
        .item(
            &MenuItemBuilder::with_id("show_help", "Playground Help…")
                .accelerator("CmdOrCtrl+Shift+/")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("show_about", "About Rustic Playground").build(handle)?)
        .build()?;

    let mut menu_builder = MenuBuilder::new(handle)
        .item(&app_submenu)
        .item(&project_menu)
        .item(&playground_menu)
        .item(&run_menu)
        .item(&edit_menu);
    if let Some(ref lm) = language_menu {
        menu_builder = menu_builder.item(lm);
    }
    menu_builder.item(&help_menu).build()
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn rebuild_menu(
    projects: Vec<String>,
    active: String,
    playground_count: usize,
    has_active_playground: bool,
    has_active_tab: bool,
    project_type: String,
    is_book_project: bool,
    project_sources: HashMap<String, String>,
    enabled_languages: Vec<String>,
    app: AppHandle,
) -> Result<(), String> {
    let menu = build_menu(
        &app,
        &projects,
        &active,
        playground_count,
        has_active_playground,
        has_active_tab,
        &project_type,
        is_book_project,
        &project_sources,
        &enabled_languages,
    )
    .map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    Ok(())
}
