//! Menu aplikasi desktop (pengganti menu bawaan).
//!
//! Item About bawaan macOS membuka panel standar sistem tanpa event ke
//! aplikasi; item kustom `show-about` di bawah memicu event `show-about`
//! ke frontend (AboutPage). Semua item lain = predefined: perilaku dan
//! shortcut standar OS tetap asli (Cmd+Z/X/C/V/A/W/Q, dsb).

/// Id item menu kustom + nama event frontend (sama, satu sumber).
pub const SHOW_ABOUT_ID: &str = "show-about";

/// Rakit menu aplikasi. Hanya dipanggil di desktop.
pub fn build(app: &tauri::AppHandle) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{MenuBuilder, SubmenuBuilder};
    let app_menu = SubmenuBuilder::new(app, "Kuron Studio")
        .text(SHOW_ABOUT_ID, "Tentang Kuron Studio")
        .separator()
        .services()
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        .quit()
        .build()?;
    let file_menu = SubmenuBuilder::new(app, "Berkas").close_window().build()?;
    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let window_menu = SubmenuBuilder::new(app, "Jendela")
        .minimize()
        .fullscreen()
        .separator()
        .bring_all_to_front()
        .build()?;
    MenuBuilder::new(app)
        .items(&[&app_menu, &file_menu, &edit_menu, &window_menu])
        .build()
}
