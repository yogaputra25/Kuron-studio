//! Seed sample project: pembuatan dummy manga page dengan bubble dan panel.
//! Dipindah dari `commands/project.rs` — tanpa perubahan perilaku.

use uuid::Uuid;

use crate::bubble::BubbleBox;
use crate::domain::{Page, PageStatus, Project};
use crate::error::AppError;
use crate::store::ProjectStore;

fn px(img: &mut image::RgbImage, x: i32, y: i32, c: image::Rgb<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
        img.put_pixel(x as u32, y as u32, c);
    }
}

fn fill_rect(img: &mut image::RgbImage, x: u32, y: u32, w: u32, h: u32, c: image::Rgb<u8>) {
    for dy in 0..h {
        for dx in 0..w {
            px(img, (x + dx) as i32, (y + dy) as i32, c);
        }
    }
}

/// Panel manga: fill + bingkai hitam tebal.
fn panel(img: &mut image::RgbImage, x: u32, y: u32, w: u32, h: u32, fill: image::Rgb<u8>) {
    let ink = image::Rgb([24, 22, 20]);
    fill_rect(img, x, y, w, h, fill);
    let t = 6;
    fill_rect(img, x, y, w, t, ink);
    fill_rect(img, x, y + h.saturating_sub(t), w, t, ink);
    fill_rect(img, x, y, t, h, ink);
    fill_rect(img, x + w.saturating_sub(t), y, t, h, ink);
}

/// Bubble ellipse: fill terang + tepi hitam; kembalikan BubbleBox-nya.
fn ellipse_bubble(
    img: &mut image::RgbImage,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
) -> BubbleBox {
    let paper = image::Rgb([255, 255, 255]);
    let ink = image::Rgb([20, 20, 20]);
    for dy in -ry..=ry {
        for dx in -rx..=rx {
            let d = (dx * dx * ry * ry) + (dy * dy * rx * rx);
            let r2 = rx * rx * ry * ry;
            if d <= r2 {
                let edge = (dx * dx * (ry - 3) * (ry - 3)) + (dy * dy * (rx - 3) * (rx - 3));
                let edge_r2 = (rx - 3) * (rx - 3) * (ry - 3) * (ry - 3);
                let col = if edge > edge_r2 { ink } else { paper };
                px(img, cx + dx, cy + dy, col);
            }
        }
    }
    BubbleBox {
        x: cx - rx,
        y: cy - ry,
        w: rx * 2,
        h: ry * 2,
        confidence: 0.95,
        shape: None,
        kind: None,
        tail: None,
    }
}

/// Satu halaman contoh (800×1152): panel + 2–3 bubble. Layout beda per indeks.
fn sample_page(i: usize) -> (image::RgbImage, Vec<BubbleBox>) {
    let mut img = image::RgbImage::new(800, 1152);
    let paper = image::Rgb([250, 249, 246]);
    let tone = image::Rgb([226, 221, 213]);
    let mid = image::Rgb([180, 175, 168]);
    fill_rect(&mut img, 0, 0, 800, 1152, paper);
    let mut bubbles = Vec::new();
    match i % 3 {
        0 => {
            panel(&mut img, 40, 50, 720, 480, tone);
            panel(&mut img, 40, 560, 345, 540, mid);
            panel(&mut img, 415, 560, 345, 540, tone);
            bubbles.push(ellipse_bubble(&mut img, 620, 180, 70, 95));
            bubbles.push(ellipse_bubble(&mut img, 200, 720, 60, 85));
            bubbles.push(ellipse_bubble(&mut img, 560, 800, 65, 90));
        }
        1 => {
            panel(&mut img, 40, 50, 720, 320, tone);
            panel(&mut img, 40, 400, 720, 320, mid);
            panel(&mut img, 40, 750, 720, 350, tone);
            bubbles.push(ellipse_bubble(&mut img, 600, 180, 75, 90));
            bubbles.push(ellipse_bubble(&mut img, 220, 530, 65, 80));
            bubbles.push(ellipse_bubble(&mut img, 580, 900, 70, 95));
        }
        _ => {
            panel(&mut img, 40, 50, 720, 680, tone);
            panel(&mut img, 40, 760, 720, 340, mid);
            bubbles.push(ellipse_bubble(&mut img, 580, 220, 80, 110));
            bubbles.push(ellipse_bubble(&mut img, 240, 900, 75, 95));
        }
    }
    (img, bubbles)
}

/// Project contoh untuk onboarding: 3 halaman PNG generik + bubble seed.
/// File tinggal di store dir (`{id}_sample`) → ikut terhapus saat project
/// dihapus, seperti folder ekstraksi CBZ.
pub fn create_sample_project_inner(store: &mut ProjectStore, name: &str) -> Result<Project, AppError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::parse("project name is empty"));
    }
    let mut project = Project::new(name);
    let dir = store.dir.join(format!("{}_sample", project.id));
    std::fs::create_dir_all(&dir)?;
    let folder = project.folders.first_mut().expect("Project::new selalu punya folder default");
    for (i, slug) in ["p001", "p002", "p003"].iter().enumerate() {
        let (img, bubbles) = sample_page(i);
        let path = dir.join(format!("{slug}.png"));
        img.save(&path)?;
        folder.pages.push(Page {
            id: Uuid::new_v4().to_string(),
            path: path.to_string_lossy().to_string(),
            width: img.width(),
            height: img.height(),
            status: PageStatus::Detected,
            bubbles,
            translation: None,
        });
    }
    store.projects.insert(project.id.clone(), project.clone());
    store.save()?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("{tag}-{n}"));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn sample_project_has_pages_with_seeded_bubbles() {
        let dir = temp_dir("kuron-studio-sample");
        let mut store = ProjectStore::load(dir.clone()).unwrap();
        let proj = create_sample_project_inner(&mut store, "Contoh").unwrap();
        let pages = &proj.folders[0].pages;
        assert_eq!(proj.folders.len(), 1);
        assert_eq!(pages.len(), 3);
        for pg in pages {
            assert!(std::path::Path::new(&pg.path).exists());
            assert_eq!(pg.status, PageStatus::Detected);
            assert!(!pg.bubbles.is_empty(), "harus ada bubble seed");
            assert!(pg.translation.is_none());
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}
