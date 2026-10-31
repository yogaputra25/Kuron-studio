//! Modul snapshot halaman untuk translate: PageSnapshot dan load_snapshot.
//! Dipindah dari `commands/translate.rs` — tanpa perubahan perilaku.

use crate::bubble::BubbleBox;
use crate::error::AppError;
use crate::translation::PageTranslation;
use crate::AppState;

/// Owned snapshot halaman; diambil sekali agar worker batch tak pinjam State.
#[derive(Debug, Clone)]
pub struct PageSnapshot {
    pub page_id: String,
    pub path: String,
    pub bubbles: Vec<BubbleBox>,
    pub page_w: i32,
    pub page_h: i32,
    pub prev: Option<PageTranslation>,
}

pub fn load_snapshot(
    state: &AppState,
    page_id: &str,
) -> Result<PageSnapshot, AppError> {
    let store = crate::error::lock_or_err(&state.store)?;
    let pg = store.find_page(page_id).ok_or_else(|| crate::error::AppError::page_not_found(page_id))?;
    Ok(PageSnapshot {
        page_id: page_id.to_string(),
        path: pg.path,
        bubbles: pg.bubbles,
        page_w: pg.width as i32,
        page_h: pg.height as i32,
        prev: pg.translation,
    })
}
