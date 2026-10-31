//! Render/gambar (BE-layer): mosaic/preview, deteksi bubble, outline,
//! paint teks halaman, dan PSD. Pindahan verbatim dari top-level
//! (`image_ops`, `detect_decode`, `detector`, `outline`, `psd`) dan
//! `commands::export` (`page`) — tanpa perubahan perilaku.

pub mod detect_decode;
pub mod detector;
pub mod image_ops;
pub mod outline;
pub mod page;
pub mod psd;
pub mod export;
pub mod sample;
