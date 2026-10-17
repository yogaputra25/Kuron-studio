/**
 * Layout teks — SATU sumber untuk kanvas dan export (export-text-parity P1).
 *
 * Semua angka di sini dalam **px halaman**, bukan px layar. Itu yang membuat
 * hasilnya tidak berubah saat lebar pane berubah, dan membuat kanvas + export
 * bisa memakai nilai yang sama persis.
 *
 * Modul ini SENGAJA tidak mengimpor Konva. Pengukuran lebar disuntik lewat
 * `MeasureApi`, sehingga algoritmanya bisa diuji di node dengan pengukur palsu
 * dan modul tetap bisa dipakai di jalur yang tak menyentuh DOM.
 */

export interface TextLayout {
  /** Ukuran huruf, px halaman. */
  fontSize: number;
  /** Jarak baris sebagai rasio terhadap `fontSize` (bukan px). */
  lineHeight: number;
  /** Padding dalam kotak, px halaman. */
  padding: number;
  /** Baris final — perender TIDAK boleh memutus ulang. */
  lines: string[];
}

/** Pengukur lebar satu baris pada ukuran tertentu. */
export interface MeasureApi {
  width(line: string, fontSize: number): number;
}

/** Jarak baris. Dipakai kanvas DAN export; jangan disalin ke tempat lain. */
export const LAYOUT_LINE_HEIGHT = 1.25;
/** Padding dalam kotak, px halaman. */
export const LAYOUT_PADDING = 4;
/**
 * Batas ukuran huruf, px HALAMAN.
 *
 * Dulu nilai ini px layar (7..28) dan ikut mengecil saat pane menyempit — itu
 * yang membuat export dan kanvas tak pernah cocok. Sebagai px halaman, arti
 * angkanya bergeser: pada halaman yang ditampilkan `s ≈ 0.4` (kasus umum untuk
 * scan 2000 px), batas lama setara `[7/0.4, 28/0.4] = [17, 70]` px halaman.
 *
 * Karena itu MAKS dipilih 72 — dekat dengan batas atas efektif yang lama di
 * zoom umum, dan tetap satu angka tetap. Efeknya: teks pendek di bubble besar
 * boleh tumbuh sampai memenuhi kotak (memang itu arti "terbesar yang muat"),
 * sesuatu yang dulu dicegah batas 28 px layar.
 *
 * MIN 10 lebih longgar dari padanan lamanya; itu disengaja — ukuran font kini
 * bisa ditetapkan manual per bubble, jadi lantai yang longgar tidak memaksa
 * user mengakali.
 *
 * Angka-angka ini yang perlu dilihat mata di halaman nyata sebelum dikunci.
 */

export const LAYOUT_MIN_FONT = 10;
export const LAYOUT_MAX_FONT = 72;
export const LAYOUT_FONT_STEP = 0.5;
export interface LayoutOpts {
  padding?: number;
  lineHeight?: number;
  min?: number;
  max?: number;
  step?: number;
  /**
   * Ukuran manual dari user. Bila diisi, auto-fit DILEWATI — ukuran itu
   * dipakai apa adanya (teks boleh luber; itu keputusan user, dan export
   * menggambar hal yang sama karena barisnya ikut dibekukan).
   */
  fontSize?: number | null;
}
/**
 * Bungkus teks jadi baris-baris yang muat `maxW`.
 *
 * Kata yang lebih lebar dari kotak dipecah per karakter — bukan dibiarkan
 * luber. Kalau dibiarkan, mengecilkan font tak akan pernah menolong (kata
 * tetap lebih lebar dari kotak pada ukuran minimum), dan export jadi luber
 * permanen. Konva berperilaku sama untuk kata panjang.
 */
export function wrapLines(text: string, maxW: number, fontSize: number, m: MeasureApi): string[] {
  const out: string[] = [];
  for (const para of text.split("\n")) {
    const words = para.split(/\s+/).filter((w) => w.length > 0);
    if (words.length === 0) {
      out.push("");
      continue;
    }
    let cur = "";
    for (const word of words) {
      const cand = cur ? `${cur} ${word}` : word;
      if (m.width(cand, fontSize) <= maxW) {
        cur = cand;
        continue;
      }
      if (cur) {
        out.push(cur);
        cur = "";
      }
      if (m.width(word, fontSize) <= maxW) {
        cur = word;
        continue;
      }
      let piece = "";
      for (const ch of word) {
        if (piece && m.width(piece + ch, fontSize) > maxW) {
          out.push(piece);
          piece = "";
        }
        piece += ch;
      }
      cur = piece;
    }
    if (cur) out.push(cur);
  }
  return out;
}

/**
 * Hitung layout final: ukuran huruf terbesar yang blok teksnya masih muat
 * tinggi kotak, beserta baris finalnya.
 *
 * Tinggi blok dihitung aritmetika (`n * fontSize * lineHeight`), bukan lewat
 * pengukuran perender: kanvas (Konva) dan export (Rust) sama-sama memajukan
 * kursor sebesar `fontSize * lineHeight` per baris. Rumusnya sengaja identik
 * di dua sisi — lihat catatan yang sama di `commands/export.rs`.
 */
export function computeTextLayout(
  text: string,
  w: number,
  h: number,
  m: MeasureApi,
  opts: LayoutOpts = {},
): TextLayout {
  const padding = opts.padding ?? LAYOUT_PADDING;
  const lineHeight = opts.lineHeight ?? LAYOUT_LINE_HEIGHT;
  const min = opts.min ?? LAYOUT_MIN_FONT;
  const max = Math.max(opts.max ?? LAYOUT_MAX_FONT, min);
  const step = opts.step ?? LAYOUT_FONT_STEP;

  const innerW = Math.max(1, w - padding * 2);
  const innerH = Math.max(1, h - padding * 2);

  // Ukuran manual dari user: lewati auto-fit. Baris tetap dipecah pada ukuran
  // itu, dan hasilnya ikut dibekukan ke `layout` — jadi export menggambar hal
  // yang sama, termasuk kalau teksnya luber (itu pilihan user, bukan bug).
  const manual = opts.fontSize ?? 0;
  if (manual > 0) {
    return { fontSize: manual, lineHeight, padding, lines: wrapLines(text, innerW, manual, m) };
  }

  if (!(step > 0)) {
    return { fontSize: min, lineHeight, padding, lines: wrapLines(text, innerW, min, m) };
  }

  let fontSize = max;
  let lines = wrapLines(text, innerW, fontSize, m);
  while (fontSize > min && lines.length * fontSize * lineHeight > innerH) {
    fontSize = Math.max(min, fontSize - step);
    lines = wrapLines(text, innerW, fontSize, m);
  }
  return { fontSize, lineHeight, padding, lines };
}
