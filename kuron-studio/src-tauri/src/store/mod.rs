//! Persistensi `projects.json`: atomic write + backup + recovery.
//! Dipindah dari `commands/project.rs` (#52). Bentuk JSON tidak berubah;
//! yang berubah hanya durabilitas tulis dan pemulihan saat load.
mod atomic;
mod project_store;
pub mod folder_ops;

pub use project_store::ProjectStore;
