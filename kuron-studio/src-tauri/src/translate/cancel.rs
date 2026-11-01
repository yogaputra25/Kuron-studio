//! CancelState: flag batal + status pra-klaim per halaman (translate layer).
//! Dipindah dari `AppState` (#38): satu-satunya sumber kebenaran cancel,
//! di-manage terpisah (`app.manage(CancelState::default())`) agar command
//! adapter tipis dan pipeline unit-testable tanpa Tauri runtime.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::domain::PageStatus;

/// Flag `cancel` dibaca `backoff_translate` tiap iterasi + saat sleep via
/// `select!`; `prev_status` menyimpan status sebelum `claim_page`
/// (Translating menimpa) untuk path cancel-restore via helper,
/// bukan `fail_page`.
#[derive(Default)]
pub struct CancelState {
    cancel: Mutex<HashSet<String>>,
    prev_status: Mutex<HashMap<String, PageStatus>>,
}

impl CancelState {
    pub fn stash_prev(&self, page_id: &str, st: PageStatus) {
        if let Ok(mut m) = self.prev_status.lock() {
            m.insert(page_id.to_string(), st);
        }
    }

    pub fn take_prev(&self, page_id: &str) -> Option<PageStatus> {
        self.prev_status.lock().ok()?.remove(page_id)
    }

    pub fn clear_cancel(&self, page_id: &str) {
        if let Ok(mut c) = self.cancel.lock() {
            c.remove(page_id);
        }
    }

    pub fn request_cancel(&self, page_id: &str) {
        if let Ok(mut c) = self.cancel.lock() {
            c.insert(page_id.to_string());
        }
    }

    pub fn is_cancelled(&self, page_id: &str) -> bool {
        self.cancel.lock().map(|c| c.contains(page_id)).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_roundtrip() {
        let cs = CancelState::default();
        assert!(!cs.is_cancelled("p"));
        cs.request_cancel("p");
        assert!(cs.is_cancelled("p"));
        cs.clear_cancel("p");
        assert!(!cs.is_cancelled("p"));
    }

    #[test]
    fn prev_roundtrip() {
        let cs = CancelState::default();
        assert!(cs.take_prev("p").is_none());
        cs.stash_prev("p", PageStatus::Detected);
        assert_eq!(cs.take_prev("p"), Some(PageStatus::Detected));
        assert!(cs.take_prev("p").is_none());
    }
}
