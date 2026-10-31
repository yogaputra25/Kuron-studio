//! Tipe domain murni: data project/halaman/status/tree tanpa I/O,
//! tanpa Tauri/HTTP/SQLite. Dipindah verbatim dari `commands/project.rs` (#51).

pub mod page;
pub mod project;
pub mod status;
pub mod tree;

pub use page::Page;
pub use project::{ChapterFolder, Project, DEFAULT_FOLDER_NAME};
pub use status::{PageStatus, apply_translation_status};
pub(crate) use status::normalize_page_status;
