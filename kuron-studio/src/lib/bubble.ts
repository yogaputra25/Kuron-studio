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
