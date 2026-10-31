//! Session ONNX lazy + thread-safe untuk `bubble.onnx`.
//!
//! `BubbleDetector` dipegang `DetectState` (Mutex): load sekali via
//! `ensure_loaded`, infer per page dari bytes file. Berat (infer) jalan di
//! `spawn_blocking` agar async runtime Tauri tak tersumbat; `ort::Session`
//! dipakai dari satu thread blocking dalam satu waktu (Mutex).
//! `intra_threads=2` sesuai M1-4; session di-reuse antar page.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::bubble::{post_process, BubbleBox};
use crate::error::AppError;
use crate::render::detect_decode::{decode_kept, letterbox, mask_coeffs, model_mask_polygon, proto_crop};
use ort::value::Tensor;
pub struct BubbleDetector {
    session: Option<ort::session::Session>,
    model_path: Option<PathBuf>,
}

impl BubbleDetector {
    pub fn empty() -> Self {
        Self {
            session: None,
            model_path: None,
        }
    }

    fn ensure_loaded(&mut self, path: &PathBuf) -> Result<(), AppError> {
        if self.model_path.as_ref() == Some(path) && self.session.is_some() {
            return Ok(());
        }
        // ponytail: ort 2.0.0-rc.13 `init().commit()` tidak fallible (bool).
        // Ganti dengan builder stabil (`ort::init().with_name(..).commit()?`) saat upgrade ort.
        ort::init().commit();
        let session = ort::session::Session::builder()
            .map_err(|e| AppError::provider(format!("ort builder: {e}")))?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
            .map_err(|e| AppError::provider(format!("ort opt: {e}")))?
            .with_intra_threads(2)
            .map_err(|e| AppError::provider(format!("ort threads: {e}")))?
            .commit_from_file(path)
            .map_err(|e| AppError::provider(format!("ort load {}: {e}", path.display())))?;
        self.session = Some(session);
        self.model_path = Some(path.clone());
        Ok(())
    }

    /// Infer satu halaman (bytes file) → boxes px original (sudah post_process).
    pub fn detect_bytes(&mut self, model_path: &PathBuf, page_bytes: &[u8]) -> Result<Vec<BubbleBox>, AppError> {
        self.ensure_loaded(model_path)?;
        let img = image::load_from_memory(page_bytes)
            .map_err(|e| AppError::parse(format!("decode: {e}")))?
            .to_rgb8();
        let (w, h) = (img.width(), img.height());
        let (tensor_data, lb) = letterbox(&img);
        let input = ndarray::Array4::from_shape_vec(
            (1, 3, crate::render::detect_decode::MODEL_SIDE as usize, crate::render::detect_decode::MODEL_SIDE as usize),
            tensor_data,
        )
        .map_err(|e| AppError::provider(format!("tensor: {e}")))?;
        let tensor = Tensor::from_array(input).map_err(|e| AppError::provider(format!("tensor: {e}")))?;

        let session = self.session.as_mut().ok_or_else(|| AppError::provider("session belum load"))?;
        let input_name = model_input_name(session)?;
        let has_protos = session.outputs().iter().any(|o| o.name() == "output1");
        let outputs = session
            .run(ort::inputs![input_name.as_str() => tensor])
            .map_err(|e| AppError::provider(format!("infer {w}x{h}: {e}")))?;
        let arr = outputs["output0"]
            .try_extract_array::<f32>()
            .map_err(|e| AppError::provider(format!("output0: {e}")))?;
        let s: Vec<i64> = arr.shape().iter().map(|&d| d as i64).collect();
        if s != [1, 37, 21504] {
            return Err(AppError::provider(format!("output0 shape tak dikenal: {s:?}")));
        }
        let flat: Vec<f32> = arr.iter().copied().collect();
        // Prototypes mask (YOLO-seg output1): opsional — bentuk asing = abaikan diam-diam.
        let protos: Option<Vec<f32>> = has_protos
            .then(|| {
                outputs["output1"]
                    .try_extract_array::<f32>()
                    .ok()
                    .filter(|a| a.shape() == [1, 32, 256, 256])
                    .map(|a| a.iter().copied().collect())
            })
            .flatten();
        let kept = decode_kept(&flat, &lb, 0.25);
        let boxes: Vec<BubbleBox> = kept
            .into_iter()
            .map(|(mut b, _, anchor)| {
                if b.shape.is_none() {
                    if let Some(p) = &protos {
                        let coeffs = mask_coeffs(&flat, anchor);
                        if let Some(crop) = proto_crop(&b, &lb) {
                            if let Some(poly) = model_mask_polygon(p, &coeffs, crop, &lb) {
                                b.shape = Some(poly);
                            }
                        }
                    }
                }
                b
            })
            .collect();
        // ponytail: mask threshold 0.5 + RDP default; tuning setelah overlay M2 dinilai mata.
        Ok(post_process(boxes))
}
}

/// Nama input dibaca dari metadata session (aman dari rename "images").
fn model_input_name(session: &ort::session::Session) -> Result<String, AppError> {
    session
        .inputs()
        .first()
        .map(|i| i.name().to_string())
        .ok_or_else(|| AppError::provider("model tanpa input".to_string()))
}

/// State deteksi: path model + session lazy. `Mutex` agar `&self` di command
/// bisa pinjam mutabel untuk infer berurutan (batch loop serial).
pub struct DetectorState {
    inner: Mutex<BubbleDetector>,
}

impl std::fmt::Debug for DetectorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetectorState").finish_non_exhaustive()
    }
}

impl Default for DetectorState {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectorState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BubbleDetector::empty()),
        }
    }
    pub fn detect_bytes(&self, model_path: &PathBuf, page_bytes: &[u8]) -> Result<Vec<BubbleBox>, AppError> {
        let mut det = crate::error::lock_or_err(&self.inner)?;
        det.detect_bytes(model_path, page_bytes)
    }
}
