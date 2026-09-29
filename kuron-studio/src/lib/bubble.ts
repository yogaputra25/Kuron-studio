import type { BubbleBox, ReadingDirection } from "./types";

/** Urutan baca: RTL (manga) kanan→kiri, LTR (manhwa) kiri→kanan, atas→bawah. */
export function readingOrder(
  bubbles: BubbleBox[],
  dir: ReadingDirection,
): BubbleBox[] {
  return [...bubbles].sort((a, b) => {
    const rowA = a.y + a.h / 2;
    const rowB = b.y + b.h / 2;
    const rowH = Math.max(a.h, b.h, 1);
    if (Math.abs(rowA - rowB) > rowH * 0.5) return rowA - rowB;
    return dir === "rtl" ? b.x - a.x : a.x - b.x;
  });
}

/** Nomor chip 1-based sesuai urutan baca; key = index di array asli. */
export function chipNumbers(
  bubbles: BubbleBox[],
  dir: ReadingDirection,
): Map<number, number> {
  const indexed = bubbles.map((b, i) => ({ b, i }));
  indexed.sort((p, q) => {
    const rowH = Math.max(p.b.h, q.b.h, 1);
    const rowP = p.b.y + p.b.h / 2;
    const rowQ = q.b.y + q.b.h / 2;
    if (Math.abs(rowP - rowQ) > rowH * 0.5) return rowP - rowQ;
    return dir === "ltr" ? p.b.x - q.b.x : q.b.x - p.b.x;
  });
  const out = new Map<number, number>();
  indexed.forEach((p, rank) => out.set(p.i, rank + 1));
  return out;
}

/** Clamp bubble ke bounds image agar tak ada koordinat negatif/overflow. */
export function clampBubble(b: BubbleBox, w: number, h: number): BubbleBox {
  const x = Math.min(Math.max(0, Math.round(b.x)), Math.max(0, w - 1));
  const y = Math.min(Math.max(0, Math.round(b.y)), Math.max(0, h - 1));
  const bw = Math.min(Math.max(1, Math.round(b.w)), w - x);
  const bh = Math.min(Math.max(1, Math.round(b.h)), h - y);
  return { ...b, x, y, w: bw, h: bh };
}

/**
 * Ukuran font TERBESAR (≤ max) yang tinggi blok teksnya masih muat di `boxH`.
 *
 * `measure(fs)` harus mengembalikan tinggi blok teks pada ukuran `fs`.
 *
 * Kenapa fungsi ini ada: Konva tidak pernah mengecilkan font sendiri, jadi
 * kita yang mengecilkan. Dulu loop-nya memanggil `Konva.Text.height()` yang
 * mengembalikan **0** selama `height` belum di-set — itu getter Node (default
 * 0), bukan `getHeight()` yang menghitung tinggi blok. Kondisi `0 > boxH`
 * selalu false, loop tidak pernah jalan, dan ukuran font beku di nilai awal
 * untuk semua bubble: kekecilan di bubble besar, meluber di bubble kecil.
 */
export function fitFontSize(
  boxH: number,
  measure: (fontSize: number) => number,
  { min = 7, max = 28, step = 0.5, slack = 0.5 } = {},
): number {
  if (!(step > 0)) return min;
  let fs = max;
  // Toleransi `slack`: layout Konva memotong baris begitu N*lineHeight melebihi
  // tinggi kotak, jadi "muat" harus memakai <= boxH, bukan < boxH — tanpa
  // toleransi teks yang pas muat bisa terpotong akibat pembulatan float.
  while (fs > min && measure(fs) > boxH + slack) fs -= step;
  return fs;
}
