use tauri::menu::{
    CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::AppHandle;

/// Builds the full macOS menu bar.  Called once on startup and again via
/// `rebuild_menu` whenever the project list or active project changes.
pub(crate) fn build_menu<R: tauri::Runtime>(
    handle: &impl tauri::Manager<R>,
    projects: &[String],
    active: &str,
    playground_count: usize,
) -> tauri::Result<tauri::menu::Menu<R>> {
    let app_submenu = SubmenuBuilder::new(handle, "Rustic Playground")
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

    // Build the dynamic per-project check items first so they outlive the builder.
    let check_items: Vec<tauri::menu::CheckMenuItem<R>> = projects
        .iter()
        .map(|name| {
            CheckMenuItemBuilder::with_id(format!("switch_project::{}", name), name.as_str())
                .checked(name.as_str() == active)
                .build(handle)
        })
        .collect::<tauri::Result<Vec<_>>>()?;

    let new_project_item = MenuItemBuilder::with_id("new_project", "New Project…")
        .accelerator("CmdOrCtrl+Shift+N")
        .build(handle)?;
    let rename_project_item =
        MenuItemBuilder::with_id("rename_project", "Rename Project…").build(handle)?;
    let delete_project_item = MenuItemBuilder::with_id("delete_project", "Delete Project…")
        .enabled(projects.len() > 1)
        .build(handle)?;

    let mut proj_builder = SubmenuBuilder::new(handle, "Project")
        .item(&new_project_item)
        .separator();
    for item in &check_items {
        proj_builder = proj_builder.item(item);
    }
    let project_menu = proj_builder
        .separator()
        .item(&rename_project_item)
        .item(&delete_project_item)
        .build()?;

    let playground_menu = SubmenuBuilder::new(handle, "Playground")
        .item(
            &MenuItemBuilder::with_id("new_playground", "New Playground")
                .accelerator("CmdOrCtrl+N")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("save", "Save")
                .accelerator("CmdOrCtrl+S")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("close_tab", "Close Tab")
                .accelerator("CmdOrCtrl+W")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("menu_delete_playground", "Delete Playground…")
                .enabled(playground_count > 0)
                .build(handle)?,
        )
        .build()?;

    let run_menu = SubmenuBuilder::new(handle, "Run")
        .item(
            &MenuItemBuilder::with_id("run_playground", "Run")
                .accelerator("CmdOrCtrl+R")
                .build(handle)?,
        )
        .item(
            &MenuItemBuilder::with_id("stop_playground", "Stop")
                .accelerator("CmdOrCtrl+.")
                .build(handle)?,
        )
        .build()?;

    let edit_menu = SubmenuBuilder::new(handle, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .separator()
        .select_all()
        .build()?;

    let help_menu = SubmenuBuilder::new(handle, "Help")
        .item(
            &MenuItemBuilder::with_id("show_help", "Playground Help…")
                .accelerator("CmdOrCtrl+Shift+/")
                .build(handle)?,
        )
        .separator()
        .item(
            &MenuItemBuilder::with_id("seed_rust_book", "Load Rust Book Examples…")
                .build(handle)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("show_about", "About Rustic Playground").build(handle)?)
        .build()?;

    MenuBuilder::new(handle)
        .item(&app_submenu)
        .item(&project_menu)
        .item(&playground_menu)
        .item(&run_menu)
        .item(&edit_menu)
        .item(&help_menu)
        .build()
}

#[tauri::command]
pub fn rebuild_menu(
    projects: Vec<String>,
    active: String,
    playground_count: usize,
    app: AppHandle,
) -> Result<(), String> {
    let menu = build_menu(&app, &projects, &active, playground_count).map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    Ok(())
}
