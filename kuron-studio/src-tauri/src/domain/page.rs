//! Satu halaman dalam folder: path file + dimensi + status + bubble + terjemahan.
use serde::{Deserialize, Serialize};

use crate::bubble::BubbleBox;
use crate::translation::PageTranslation;

use super::status::PageStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub id: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub status: PageStatus,
    /// Bubble hasil detect/edit canvas (original px). Kosong = belum detect.
    #[serde(default)]
    pub bubbles: Vec<BubbleBox>,
    #[serde(default)]
    pub translation: Option<PageTranslation>,
}
