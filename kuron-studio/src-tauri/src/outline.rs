//! Outline-following polygon dari bbox deteksi (bubble-polygon D1).
//!
//! Prediksi: gelembung putih ber-outline tinta. Pipeline murni, tanpa model:
//! flood fill interior (dinding = outline, opening 3x3 anti-tone) →
//! tracing kontur (Moore) → simplifikasi Douglas-Peucker + cap titik.
//! Gagal (tak ada dinding / terlalu penuh / interior gelap / terlalu kecil) → `None`, pemanggil pakai kotak.

use image::RgbImage;

/// Luminance perseptual 0..255.
fn lum(p: &[u8; 3]) -> f32 {
    (p[0] as f32 * 299.0 + p[1] as f32 * 587.0 + p[2] as f32 * 114.0) / 1000.0
}

/// Ambang outline tinta. Di bawah ini = dinding (ink, tone gelap, teks).
pub const WALL_LUM: f32 = 100.0;
/// Toleransi simplifikasi (px).
pub const RDP_EPSILON: f32 = 1.5;
/// Batas titik polygon (hemat `projects.json`).
pub const MAX_POINTS: usize = 64;
/// Bbox minimum agar refine layak (px).
pub const MIN_SIDE: i32 = 16;

/// Refine bbox → polygon outline, atau `None` bila tak layak (fallback kotak).
pub fn refine_outline(img: &RgbImage, x: i32, y: i32, w: i32, h: i32) -> Option<Vec<[i32; 2]>> {
    if w < MIN_SIDE || h < MIN_SIDE {
        return None;
    }
    let (iw, ih) = (img.width() as i32, img.height() as i32);
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + w).min(iw);
    let y1 = (y + h).min(ih);
    if x1 - x0 < MIN_SIDE || y1 - y0 < MIN_SIDE {
        return None;
    }
    let lw = (x1 - x0) as usize;
    let lh = (y1 - y0) as usize;
    // Peta dinding + opening 3x3 (tone dot dihapus, garis tebal bertahan).
    let wall = |lx: i32, ly: i32| -> bool {
        let (gx, gy) = (x0 + lx, y0 + ly);
        gx < 0 || gy < 0 || gx >= x1 || gy >= y1 || lum(&img.get_pixel(gx as u32, gy as u32).0) < WALL_LUM
    };
    // Erode: wall hanya bertahan bila 3x3 penuh.
    let mut eroded = vec![vec![false; lw]; lh];
    for ly in 0..lh as i32 {
        for lx in 0..lw as i32 {
            eroded[ly as usize][lx as usize] = (-1..=1).all(|dy| {
                (-1..=1).all(|dx| {
                    let (nx, ny) = (lx + dx, ly + dy);
                    nx < 0 || ny < 0 || nx >= lw as i32 || ny >= lh as i32 || wall(nx, ny)
                })
            });
        }
    }
    // Dilate kembali: kembalikan ketebalan garis tanpa titik-titik.
    let mut open = vec![vec![false; lw]; lh];
    for ly in 0..lh as i32 {
        for lx in 0..lw as i32 {
            let mut any = false;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (lx + dx, ly + dy);
                    if nx >= 0 && ny >= 0 && (nx as usize) < lw && (ny as usize) < lh && eroded[ny as usize][nx as usize] {
                        any = true;
                    }
                }
            }
            open[ly as usize][lx as usize] = any;
        }
    }
    // Seed: tengah crop, mundur ke tetangga terang bila jatuh di tinta/teks.
    let seed = find_seed(&open, lw, lh)?;
    let mut mask = vec![vec![false; lw]; lh];
    let mut stack = vec![seed];
    let mut blocked = false;
    while let Some((sx, sy)) = stack.pop() {
        if sx < 0 || sy < 0 || sx >= lw as i32 || sy >= lh as i32 {
            continue;
        }
        let (ux, uy) = (sx as usize, sy as usize);
        if mask[uy][ux] {
            continue;
        }
        if open[uy][ux] {
            blocked = true;
            continue;
        }
        mask[uy][ux] = true;
        stack.push((sx + 1, sy));
        stack.push((sx - 1, sy));
        stack.push((sx, sy + 1));
        stack.push((sx, sy - 1));
    }
    let filled: usize = mask.iter().flatten().filter(|&&m| m).count();
    let area = (lw * lh).max(1);
    // Butuh dinding nyata (bukan kertas polos) dan isi minimal.
    if !blocked || filled * 100 < area * 3 {
        return None;
    }
    let mut contour = trace_contour(&mask)?;
    if contour.len() > MAX_POINTS {
        contour = simplify_rdp(&contour, RDP_EPSILON * 2.0, MAX_POINTS);
    } else {
        contour = simplify_rdp(&contour, RDP_EPSILON, MAX_POINTS);
    }
    if contour.len() < 3 {
        return None;
    }
    Some(
        contour
            .into_iter()
            .map(|(lx, ly)| [x0 + lx, y0 + ly])
            .collect(),
    )
}


fn find_seed(open: &[Vec<bool>], w: usize, h: usize) -> Option<(i32, i32)> {
    let (cx, cy) = (w as i32 / 2, h as i32 / 2);
    if !open[cy as usize][cx as usize] {
        return Some((cx, cy));
    }
    let max_d = (w.min(h) / 2).max(1) as i32;
    for d in 1..=max_d {
        for dy in -d..=d {
            for dx in [-d, d] {
                let (x, y) = (cx + dx, cy + dy);
                if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && !open[y as usize][x as usize] {
                    return Some((x, y));
                }
            }
        }
        for dx in -d..=d {
            for dy in [-d, d] {
                let (x, y) = (cx + dx, cy + dy);
                if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && !open[y as usize][x as usize] {
                    return Some((x, y));
                }
            }
        }
    }
    None
}

/// Tracing kontur Moore 8-neighbor searah jarum jam. Return titik-titik batas.
pub(crate) fn trace_contour(mask: &[Vec<bool>]) -> Option<Vec<(i32, i32)>> {
    let h = mask.len();
    let w = mask.first().map(|r| r.len()).unwrap_or(0);
    if w == 0 || h == 0 {
        return None;
    }
    let fg = |x: i32, y: i32| -> bool {
        x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && mask[y as usize][x as usize]
    };
    // Start: piksel teratas-terkiri yang punya tetangga background.
    let mut start: Option<(i32, i32)> = None;
    'outer: for y in 0..h as i32 {
        for x in 0..w as i32 {
            if !fg(x, y) {
                continue;
            }
            let has_bg = [(1, 0), (0, 1), (-1, 0), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)]
                .iter()
                .any(|&(dx, dy)| !fg(x + dx, y + dy));
            if has_bg {
                start = Some((x, y));
                break 'outer;
            }
        }
    }
    let s = start?;
    // 8-arah searah jarum jam mulai dari Timur.
    const DIRS: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];
    // Backtrack awal: tetangga background pertama (urutan sama).
    let mut back = 0usize;
    let mut found = false;
    for (i, &(dx, dy)) in DIRS.iter().enumerate() {
        if !fg(s.0 + dx, s.1 + dy) {
            back = i;
            found = true;
            break;
        }
    }
    if !found {
        return None;
    }
    let mut contour = vec![s];
    let mut cur = s;
    let mut back_idx = back;
    let cap = (w * h * 4 + 64).min(200_000);
    for _ in 0..cap {
        let mut advanced = false;
        for k in 1..=8usize {
            let ni = (back_idx + k) % 8;
            let np = (cur.0 + DIRS[ni].0, cur.1 + DIRS[ni].1);
            if fg(np.0, np.1) {
                cur = np;
                back_idx = (ni + 4) % 8;
                advanced = true;
                break;
            }
        }
        if !advanced {
            return None;
        }
        if cur == s {
            break;
        }
        contour.push(cur);
    }
    if cur != s || contour.len() < 3 {
        return None;
    }
    Some(contour)
}

fn perp_dist(p: (i32, i32), a: (i32, i32), b: (i32, i32)) -> f32 {
    let (px, py) = (p.0 as f32, p.1 as f32);
    let (ax, ay) = (a.0 as f32, a.1 as f32);
    let (bx, by) = (b.0 as f32, b.1 as f32);
    let dx = bx - ax;
    let dy = by - ay;
    let len2 = dx * dx + dy * dy;
    if len2 == 0.0 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / len2;
    let (cx, cy) = if t <= 0.0 {
        (ax, ay)
    } else if t >= 1.0 {
        (bx, by)
    } else {
        (ax + t * dx, ay + t * dy)
    };
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

/// Douglas-Peucker iteratif + cap titik (subsample merata bila masih berlebih).
pub(crate) fn simplify_rdp(points: &[(i32, i32)], epsilon: f32, cap: usize) -> Vec<(i32, i32)> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0usize, points.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        if b <= a + 1 {
            continue;
        }
        let mut dmax = 0.0f32;
        let mut idx = a;
        for i in a + 1..b {
            let d = perp_dist(points[i], points[a], points[b]);
            if d > dmax {
                dmax = d;
                idx = i;
            }
        }
        if dmax > epsilon {
            keep[idx] = true;
            stack.push((a, idx));
            stack.push((idx, b));
        }
    }
    let mut out: Vec<(i32, i32)> = points
        .iter()
        .enumerate()
        .filter(|(i, _)| keep[*i])
        .map(|(_, &p)| p)
        .collect();
    if out.len() > cap.max(3) {
        let step = out.len() as f32 / cap.max(3) as f32;
        let mut slim = Vec::with_capacity(cap);
        let mut f = 0.0;
        while (f as usize) < out.len() && slim.len() < cap {
            slim.push(out[f as usize]);
            f += step;
        }
        if let Some(&last) = out.last() {
            if slim.last() != Some(&last) && slim.len() < cap {
                slim.push(last);
            }
        }
        out = slim;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    /// Elips putih ber-outline tebal di atas tone abu — tiruan bubble.
    fn bubble_fixture(w: u32, h: u32) -> RgbImage {
        let mut img = RgbImage::from_pixel(w, h, image::Rgb([180, 180, 180]));
        let (cx, cy, rx, ry) = (w as f32 / 2.0, h as f32 / 2.0, w as f32 * 0.32, h as f32 * 0.28);
        for y in 0..h {
            for x in 0..w {
                let nx = (x as f32 - cx) / rx;
                let ny = (y as f32 - cy) / ry;
                let d = nx * nx + ny * ny;
                if d <= 1.0 {
                    img.put_pixel(x, y, image::Rgb([245, 245, 245]));
                }
                // Outline tebal 3px.
                if d > 0.86 && d <= 1.0 {
                    img.put_pixel(x, y, image::Rgb([20, 20, 20]));
                }
            }
        }
        img
    }

    #[test]
    fn ellipse_gets_compact_polygon() {
        let img = bubble_fixture(200, 160);
        let poly = refine_outline(&img, 0, 0, 200, 160).expect("elips harus ketemu");
        assert!(poly.len() >= 8, "terlalu sedikit: {}", poly.len());
        assert!(poly.len() <= MAX_POINTS, "melebihi cap: {}", poly.len());
        // Semua titik di dalam bbox.
        for [x, y] in &poly {
            assert!(*x >= 0 && *x < 200 && *y >= 0 && *y < 160);
        }
    }

    #[test]
    fn dark_page_falls_back_to_none() {
        // Panel gelap penuh: tak ada interior terang yang tertutup.
        let img = RgbImage::from_pixel(120, 100, image::Rgb([40, 40, 40]));
        assert!(refine_outline(&img, 0, 0, 120, 100).is_none());
    }

    #[test]
    fn plain_white_has_no_outline_so_none() {
        // Kertas putih polos: tak ada dinding untuk ditabrak → tolak.
        let img = RgbImage::from_pixel(120, 100, image::Rgb([250, 250, 250]));
        assert!(refine_outline(&img, 0, 0, 120, 100).is_none());
    }

    #[test]
    fn tiny_box_refused() {
        let img = bubble_fixture(200, 160);
        assert!(refine_outline(&img, 10, 10, 5, 5).is_none());
    }

    #[test]
    fn rdp_collapses_straight_lines() {
        let line: Vec<(i32, i32)> = (0..=20).map(|x| (x, 0)).collect();
        let out = simplify_rdp(&line, 1.0, 64);
        assert!(out.len() <= 3, "garis lurus harus collapse: {}", out.len());
    }
}
