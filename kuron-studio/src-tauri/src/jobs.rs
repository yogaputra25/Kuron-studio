//! JobRegistry: dedup + konkurensi terkendali + cancel untuk batch translate (#55).
//!
//! Registry hidup per panggilan `translate_batch`:
//!
//! - `admit` mendup `page_ids` dalam satu batch (duplikat langsung lapor
//!   busy tanpa claim). Antar-batch tetap dijaga `claim_page` via store.
//! - `semaphore` membatasi POST LLM paralel (default 3, sama seperti
//!   semaphore inline lama di `commands::batch`).
//! - `cancel` / `is_cancelled` meneruskan flag `CancelState` — satu-satunya
//!   sumber kebenaran cancel (#38: keluar dari `AppState`).

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use tokio::sync::Semaphore;

use crate::translate::CancelState;

/// Batas POST LLM paralel batch.
pub const DEFAULT_BATCH_CONCURRENCY: usize = 3;

pub struct JobRegistry {
    permits: Arc<Semaphore>,
    inflight: Mutex<HashSet<String>>,
}

impl JobRegistry {
    pub fn new(concurrency: usize) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(concurrency.max(1))),
            inflight: Mutex::new(HashSet::new()),
        }
    }

    /// Daftarkan job; `false` = page ini sudah ada di batch (duplikat) —
    /// caller JANGAN prepare, langsung lapor busy. Job yang diakui (`true`)
    /// WAJIB `release` setelah selesai (sukses maupun gagal).
    pub fn admit(&self, page_id: &str) -> bool {
        self.inflight
            .lock()
            .map(|mut s| s.insert(page_id.to_string()))
            .unwrap_or(false)
    }

    /// Lepaskan job (selesai/gagal-prepare). Tidak menyentuh flag cancel —
    /// flag milik `AppState`, bukan registry.
    pub fn release(&self, page_id: &str) {
        if let Ok(mut s) = self.inflight.lock() {
            s.remove(page_id);
        }
    }

    /// Izin paralelisme untuk worker (`acquire_owned` per worker).
    pub fn semaphore(&self) -> Arc<Semaphore> {
        Arc::clone(&self.permits)
    }

    /// Teruskan permintaan batal ke `CancelState` — satu-satunya sumber
    /// kebenaran cancel (#38: keluar dari `AppState`).
    pub fn cancel(&self, cancel: &CancelState, page_id: &str) {
        cancel.request_cancel(page_id);
    }

    pub fn is_cancelled(&self, cancel: &CancelState, page_id: &str) -> bool {
        cancel.is_cancelled(page_id)
    }

    pub fn inflight_count(&self) -> usize {
        self.inflight.lock().map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for JobRegistry {
    fn default() -> Self {
        Self::new(DEFAULT_BATCH_CONCURRENCY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn admit_dedup_dan_release() {
        let reg = JobRegistry::default();
        assert!(reg.admit("p0"));
        // Duplikat dalam batch yang sama ditolak.
        assert!(!reg.admit("p0"));
        assert_eq!(reg.inflight_count(), 1);
        // Page lain tetap diakui.
        assert!(reg.admit("p1"));
        assert_eq!(reg.inflight_count(), 2);
        reg.release("p0");
        assert_eq!(reg.inflight_count(), 1);
        // Sesudah release bisa diakui lagi.
        assert!(reg.admit("p0"));
    }

    #[test]
    fn batas_paralel_default_tiga() {
        let reg = JobRegistry::default();
        let sem = reg.semaphore();
        let _p1 = sem.try_acquire().unwrap();
        let _p2 = sem.try_acquire().unwrap();
        let _p3 = sem.try_acquire().unwrap();
        assert!(sem.try_acquire().is_err(), "izin ke-4 harus ditolak");
    }

    #[test]
    fn new_nol_tetap_minimal_satu() {
        let reg = JobRegistry::new(0);
        assert!(reg.semaphore().try_acquire().is_ok());
    }

    #[test]
    fn cancel_meneruskan_flag_cancelstate() {
        let reg = JobRegistry::default();
        let cs = CancelState::default();
        assert!(reg.admit("p0"));
        assert!(!reg.is_cancelled(&cs, "p0"));
        reg.cancel(&cs, "p0");
        assert!(reg.is_cancelled(&cs, "p0"));
        // Idempotent + release tak mencabut cancel (flag milik CancelState).
        reg.cancel(&cs, "p0");
        reg.release("p0");
        assert!(reg.is_cancelled(&cs, "p0"));
    }
}
