//! Satu-satunya tipe error internal backend (`#57` BE-layer).
//!
//! Aturan:
//! - Fungsi internal (domain/store/db/repo/provider/render/…) mengembalikan
//!   `Result<T, AppError>` dan memakai `?` + `From` — tanpa
//!   `.map_err(|e| e.to_string())` berulang.
//! - Semua `#[tauri::command]` TETAP `Result<T, String>` (kontrak Tauri
//!   byte-identik, dijaga lane paralel). Konversi hanya di titik command via
//!   `?` (pakai `From<AppError> for String` di bawah) atau `.map_err(...)`.
//! - `Display` mempertahankan string error lama byte-per-byte supaya pesan di
//!   boundary tak berubah; kategorisasi varian hanya internal.
//!
//! Pemetaan varian:
//! - `Store` = kegagalan persistensi (projects.json, SQLite, keychain).
//! - `Io` = filesystem + eksekusi task latar (`spawn_blocking` join).
//! - `Import` = arsip/zip + PDF/ekstrak.
//! - `Provider` = backend AI (HTTP LLM maupun model ONNX lokal).
//! - `Parse` = JSON/validasi/input tak valid.
//! - `Busy` / `Cancelled` / `NotFound` / `NoTranslation` = status alur kerja.

use std::fmt::Display;
use std::sync::PoisonError;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Store(String),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Import(String),
    #[error("{0}")]
    Provider(String),
    #[error("{0}")]
    Parse(String),
    #[error("page busy (already translating): {0}")]
    Busy(String),
    /// Satu-satunya string pemicu path restore (bukan `fail_page`).
    /// Sinkron dengan `crate::commands::translate::CANCELLED_MSG`.
    #[error("dibatalkan oleh user")]
    Cancelled,
    #[error("{kind} not found: {id}")]
    NotFound { kind: String, id: String },
    #[error("no translation: {0}")]
    NoTranslation(String),
}

impl AppError {
    pub fn store(msg: impl Display) -> Self {
        Self::Store(msg.to_string())
    }

    pub fn io(msg: impl Display) -> Self {
        Self::Io(msg.to_string())
    }

    pub fn import(msg: impl Display) -> Self {
        Self::Import(msg.to_string())
    }

    pub fn provider(msg: impl Display) -> Self {
        Self::Provider(msg.to_string())
    }

    pub fn parse(msg: impl Display) -> Self {
        Self::Parse(msg.to_string())
    }

    pub fn busy(page_id: impl Display) -> Self {
        Self::Busy(page_id.to_string())
    }

    pub fn cancelled() -> Self {
        Self::Cancelled
    }

    pub fn no_translation(page_id: impl Display) -> Self {
        Self::NoTranslation(page_id.to_string())
    }

    fn not_found(kind: &str, id: impl Display) -> Self {
        Self::NotFound {
            kind: kind.to_string(),
            id: id.to_string(),
        }
    }

    /// Satu konstruktor per jenis — ganti 15+ `format!("... not found: ...")`
    /// yang tersebar supaya pesan konsisten dari satu titik.
    pub fn project_not_found(id: impl Display) -> Self {
        Self::not_found("project", id)
    }

    pub fn page_not_found(id: impl Display) -> Self {
        Self::not_found("page", id)
    }

    pub fn folder_not_found(id: impl Display) -> Self {
        Self::not_found("folder", id)
    }

    pub fn parent_folder_not_found(id: impl Display) -> Self {
        Self::not_found("parent folder", id)
    }

    pub fn glossary_not_found(id: impl Display) -> Self {
        Self::not_found("glossary", id)
    }

    pub fn provider_not_found(id: impl Display) -> Self {
        Self::not_found("provider", id)
    }

    pub fn path_not_found(id: impl Display) -> Self {
        Self::not_found("path", id)
    }

    pub fn corrupt_store(e: impl Display) -> Self {
        Self::Store(format!("projects.json corrupt: {e}"))
    }

    /// Mutex poison → Store (persistensi tak bisa diakses).
    pub fn lock<T>(e: PoisonError<T>) -> Self {
        Self::Store(e.to_string())
    }
}

/// Helper untuk penguncian `std::sync::Mutex` agar ergonomis:
/// `let guard = lock_or_err(&mutex)?;`
pub fn lock_or_err<T>(mutex: &std::sync::Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, AppError> {
    mutex.lock().map_err(AppError::lock)
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Store(e.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        Self::Provider(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<image::ImageError> for AppError {
    fn from(e: image::ImageError) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<lopdf::Error> for AppError {
    fn from(e: lopdf::Error) -> Self {
        Self::Import(e.to_string())
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Import(e.to_string())
    }
}

impl From<keyring::Error> for AppError {
    fn from(e: keyring::Error) -> Self {
        Self::Store(e.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        if s == "dibatalkan oleh user" {
            Self::Cancelled
        } else {
            Self::Parse(s)
        }
    }
}

/// Konversi boundary: command `Result<T, String>` memakai `?` langsung di atas
/// `Result<T, AppError>` tanpa `map_err` manual.
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_mempertahankan_string_boundary_lama() {
        assert_eq!(
            AppError::project_not_found("p1").to_string(),
            "project not found: p1"
        );
        assert_eq!(
            AppError::page_not_found("pg").to_string(),
            "page not found: pg"
        );
        assert_eq!(
            AppError::folder_not_found("f").to_string(),
            "folder not found: f"
        );
        assert_eq!(
            AppError::parent_folder_not_found("p").to_string(),
            "parent folder not found: p"
        );
        assert_eq!(
            AppError::glossary_not_found("g").to_string(),
            "glossary not found: g"
        );
        assert_eq!(
            AppError::provider_not_found("v").to_string(),
            "provider not found: v"
        );
        assert_eq!(
            AppError::path_not_found("/x").to_string(),
            "path not found: /x"
        );
        assert_eq!(
            AppError::busy("p0").to_string(),
            "page busy (already translating): p0"
        );
        assert_eq!(
            AppError::no_translation("p0").to_string(),
            "no translation: p0"
        );
        // Sinkron dengan konstanta boundary translate.rs.
        assert_eq!(
            AppError::cancelled().to_string(),
            crate::commands::translate::CANCELLED_MSG
        );
    }

    #[test]
    fn from_memetakan_ke_varian_yang_tepat() {
        let io: AppError = std::io::Error::other("disk").into();
        assert!(matches!(io, AppError::Io(_)));
        let rusqlite: AppError = rusqlite::Error::InvalidQuery.into();
        assert!(matches!(rusqlite, AppError::Store(_)));
        let json: AppError = serde_json::from_str::<serde_json::Value>("{x}")
            .unwrap_err()
            .into();
        assert!(matches!(json, AppError::Parse(_)));
        let img: AppError = image::load_from_memory(b"bukan gambar")
            .unwrap_err()
            .into();
        assert!(matches!(img, AppError::Parse(_)));
    }

    #[test]
    fn into_string_untuk_konversi_boundary() {
        let s: String = AppError::cancelled().into();
        assert_eq!(s, "dibatalkan oleh user");
        let s: String = AppError::page_not_found("p0").into();
        assert_eq!(s, "page not found: p0");
    }

    #[test]
    fn cancel_dibandingkan_via_matches_bukan_string() {
        let e = AppError::cancelled();
        assert!(matches!(e, AppError::Cancelled));
        let e = AppError::parse("boom");
        assert!(!matches!(e, AppError::Cancelled));
    }
}
