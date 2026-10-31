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
  /**
   * Polygon bubble, KOORDINAT RELATIF bubble (frame 0..w × 0..h).
   * Bila diisi (≥3 titik): tiap baris dibatasi tali-busur polygon pada
   * pitanya dan digambar tengah-chord — teks tak keluar garis outline.
   * Absen = jalur kotak lama (byte-identik).
   */
  shape?: Array<[number, number]> | null;
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
  return wrapCore(text, () => maxW, fontSize, m);
}

/**
 * Bungkus dengan batas lebar per-baris (untuk polygon: tali-busur tiap pita).
 * Baris ke-i dibatasi `widths[min(i, akhir)]`.
 */
export function wrapLinesVar(text: string, widths: number[], fontSize: number, m: MeasureApi): string[] {
  const last = widths.length > 0 ? widths[widths.length - 1] : 1;
  return wrapCore(text, (i) => widths[i] ?? last, fontSize, m);
}

function wrapCore(text: string, widthForLine: (i: number) => number, fontSize: number, m: MeasureApi): string[] {
  const out: string[] = [];
  for (const para of text.split("\n")) {
    const words = para.split(/\s+/).filter((w) => w.length > 0);
    if (words.length === 0) {
      out.push("");
      continue;
    }
    let cur = "";
    for (const word of words) {
      const maxW = widthForLine(out.length);
      const cand = cur ? `${cur} ${word}` : word;
      if (m.width(cand, fontSize) <= maxW) {
        cur = cand;
        continue;
      }
      if (cur) {
        out.push(cur);
        cur = "";
      }
      if (m.width(word, fontSize) <= widthForLine(out.length)) {
        cur = word;
        continue;
      }
      let piece = "";
      for (const ch of word) {
        if (piece && m.width(piece + ch, fontSize) > widthForLine(out.length)) {
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

/** Fraksi lebar chord minimum terhadap chord terlebar agar pita dianggap
 *  badan bubble (bukan ekor). Ekor sempit dikeluarkan dari area teks.
 *  WAJIB sama dengan `BODY_MIN_FRAC` di `commands/export.rs`. */
export const BODY_MIN_FRAC = 0.35;
/** Langkah sampling vertikal untuk cari badan (px). Sama di dua sisi. */
const BODY_SCAN_STEP = 2;

/** Satu interval dalam polygon pada garis horizontal. */
export interface PolyChord {
  x0: number;
  x1: number;
}

/**
 * Tali-busur polygon pada garis horizontal `y` (frame relatif bubble).
 * Titik potong sisi → interval dalam (genap-ganjil) → pilih yang memuat
 * `cx`, kalau tak ada pilih yang terlebar. `null` = garis tak memotong.
 */
export function chordAt(shape: Array<[number, number]>, y: number, cx: number): PolyChord | null {
  const xs: number[] = [];
  const n = shape.length;
  for (let i = 0; i < n; i++) {
    const [x1, y1] = shape[i];
    const [x2, y2] = shape[(i + 1) % n];
    if (y1 > y !== y2 > y) {
      xs.push(x1 + ((x2 - x1) * (y - y1)) / (y2 - y1));
    }
  }
  if (xs.length < 2) return null;
  xs.sort((a, b) => a - b);
  let best: PolyChord | null = null;
  for (let i = 0; i + 1 < xs.length; i += 2) {
    const iv = { x0: xs[i], x1: xs[i + 1] };
    if (iv.x1 <= iv.x0) continue;
    if (cx >= iv.x0 && cx <= iv.x1) return iv;
    if (!best || iv.x1 - iv.x0 > best.x1 - best.x0) best = iv;
  }
  return best;
}

/** Rentang vertikal polygon (min/max y verteks). */
export function polySpanY(shape: Array<[number, number]>): { y0: number; y1: number } {
  let y0 = Infinity;
  let y1 = -Infinity;
  for (const [, y] of shape) {
    if (y < y0) y0 = y;
    if (y > y1) y1 = y;
  }
  return { y0, y1 };
}

/**
 * Rentang vertikal BADAN bubble: sampling chord tiap `BODY_SCAN_STEP` px,
 * ambil lajur menyambung di sekitar chord terlebar yang lebarnya
 * >= `BODY_MIN_FRAC` × terlebar. Ekor sempit (di bawah/di samping badan)
 * keluar dari area teks. `null` = tak ada chord sama sekali.
 * WAJIB identik dengan port Rust (`body_span`).
 */
export function bodySpan(
  shape: Array<[number, number]>,
  w: number,
  h: number,
  padding: number,
): { lo: number; hi: number } | null {
  const cx = w / 2;
  const widths: number[] = [];
  for (let y = 0; y < h; y += BODY_SCAN_STEP) {
    const c = chordAt(shape, y + BODY_SCAN_STEP / 2, cx);
    widths.push(c ? c.x1 - c.x0 : 0);
  }
  if (widths.length === 0) return null;
  let maxW = 0;
  let arg = 0;
  widths.forEach((wd, i) => {
    if (wd > maxW) {
      maxW = wd;
      arg = i;
    }
  });
  if (maxW <= 0) return null;
  const th = maxW * BODY_MIN_FRAC;
  let lo = arg;
  while (lo > 0 && widths[lo - 1] >= th) lo--;
  let hi = arg;
  while (hi + 1 < widths.length && widths[hi + 1] >= th) hi++;
  return {
    lo: Math.max(padding, lo * BODY_SCAN_STEP),
    hi: Math.min(h - padding, (hi + 1) * BODY_SCAN_STEP),
  };
}

/** Geometri gambar per-baris: pita + chord (frame relatif bubble). */
export interface LineBand {
  top: number;
  x0: number;
  x1: number;
}

/**
 * Pita tiap baris final: blok tengah dalam rentang polygon (dijepit padding),
 * tiap baris memakai chord pada tengah pitanya. `null` = tanpa shape
 * (perender pakai jalur kotak lama). WAJIB identik dengan port Rust di
 * `commands/export.rs` (`line_bands`).
 */
export function lineBands(
  shape: Array<[number, number]> | null | undefined,
  w: number,
  h: number,
  padding: number,
  lineCount: number,
  fontSize: number,
  lineHeight: number,
): LineBand[] | null {
  if (!shape || shape.length < 3 || lineCount <= 0) return null;
  const span = bodySpan(shape, w, h, padding) ?? { lo: padding, hi: h - padding };
  const spanH = Math.max(1, span.hi - span.lo);
  const linePx = fontSize * lineHeight;
  const top = span.lo + (spanH - lineCount * linePx) / 2;
  const cx = w / 2;
  const bands: LineBand[] = [];
  for (let j = 0; j < lineCount; j++) {
    const yc = top + (j + 0.5) * linePx;
    const c = chordAt(shape, yc, cx);
    bands.push(c ? { top: top + j * linePx, x0: c.x0, x1: c.x1 } : { top: top + j * linePx, x0: cx, x1: cx });
  }
  return bands;
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

  const manual = opts.fontSize ?? 0;
  // Jalur polygon: tiap baris dibatasi chord pada pitanya (D1). Tanpa shape
  // di bawah = jalur kotak lama, byte-identik.
  const shape = opts.shape && opts.shape.length >= 3 ? opts.shape : null;
  if (shape) {
    const cx = w / 2;
    // Badan bubble saja (ekor dikeluarkan dari area teks).
    const span = bodySpan(shape, w, h, padding) ?? { lo: padding, hi: h - padding };
    const spanH = Math.max(1, span.hi - span.lo);
    const heightLimit = Math.min(innerH, spanH);
    const bandW = (y: number): number => {
      const c = chordAt(shape, y, cx);
      if (!c) return 1;
      return Math.max(1, Math.min(innerW, c.x1 - c.x0 - padding * 2));
    };
    const fits = (lines: string[], font: number): boolean => {
      const linePx = font * lineHeight;
      if (lines.length * linePx > heightLimit + 1e-9) return false;
      const top = span.lo + (spanH - lines.length * linePx) / 2;
      return lines.every((ln, j) => m.width(ln, font) <= bandW(top + (j + 0.5) * linePx) + 1e-9);
    };
    const tryFont = (font: number): string[] => {
      const linePx = font * lineHeight;
      const n0 = Math.max(1, wrapLines(text, innerW, font, m).length);
      const top0 = span.lo + (spanH - n0 * linePx) / 2;
      const widths: number[] = [];
      for (let i = 0; i < n0; i++) widths.push(bandW(top0 + (i + 0.5) * linePx));
      return wrapLinesVar(text, widths, font, m);
    };
    // Ukuran manual = batas ATAS, bukan nilai mutlak: susut sampai muat
    // polygon (keputusan user: teks tak boleh keluar garis walau manual).
    const ceiling = manual > 0 ? manual : max;
    if (!(step > 0)) {
      return { fontSize: min, lineHeight, padding, lines: tryFont(min) };
    }
    let fontSize = ceiling;
    let lines = tryFont(fontSize);
    while (fontSize > min && !fits(lines, fontSize)) {
      fontSize = Math.max(min, fontSize - step);
      lines = tryFont(fontSize);
    }
    return { fontSize, lineHeight, padding, lines };
  }

  // Ukuran manual dari user: lewati auto-fit (nilai `manual` di atas).
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
