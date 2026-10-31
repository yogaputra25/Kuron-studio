// Warna editor Konva (FE-5 canvas-redraw-split 1.1).
// Konva menggambar ke <canvas>, bukan DOM — tak bisa pakai class Tailwind,
// jadi warna dibaca dari CSS var yang sama dengan token app.css. Satu-satunya
// tempat yang menyentuh `--ks-*` untuk kanvas; dibaca per redraw supaya ganti
// tema ikut (menukar tema hanya menukar nilai var).
//
// Pengecualian sadar (bukan tema, jangan dijadikan token):
// - patch putih overlay (`#ffffff`) + teks `#111`: backing fungsional agar
//   teks terbaca + cocok dengan export PNG (paritas export-text-parity).
// - scrim gelap `rgba(0,0,0,0.78)` + teks `#fff`: konstanta kontras, bekerja
//   di tema terang maupun gelap.
export interface ThemeColors {
  accent: string;
  cyan: string;
  accentRgba: string;
  cyanRgba: string;
  /** Teks di atas aksen (chip nomor). */
  accentFg: string;
}

function hexToRgba(hex: string, a: number): string {
  const h = hex.replace("#", "");
  const full = h.length === 3 ? [...h].map((c) => c + c).join("") : h;
  const [r, g, b] = [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16));
  return Number.isNaN(r) ? `rgba(53,231,245,${a})` : `rgba(${r},${g},${b},${a})`;
}

export function readThemeColors(el: Element | null): ThemeColors {
  const cs = getComputedStyle(el ?? document.body);
  const v = (n: string, fallback: string) => cs.getPropertyValue(n).trim() || fallback;
  const accent = v("--ks-accent", "#ff6b5a");
  const cyan = v("--ks-cyan", "#35e7f5");
  return {
    accent,
    cyan,
    accentRgba: hexToRgba(accent, 0.18),
    cyanRgba: hexToRgba(cyan, 0.12),
    accentFg: v("--ks-accent-fg", "#fff"),
  };
}
