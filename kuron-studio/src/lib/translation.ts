import type { BubbleBox, BubbleTranslation } from "./types";
import { needsWhitePatch } from "./bubble";

/** Gabung hasil translate baru dengan edit manual user (match by index). */
export function mergeUserEdits(
  prev: BubbleTranslation[] | undefined,
  next: BubbleTranslation[],
): BubbleTranslation[] {
  if (!prev || prev.length === 0) return next;
  const edited = new Map<number, BubbleTranslation>();
  for (const b of prev) if (b.isUserEdited) edited.set(b.index, b);
  if (edited.size === 0) return next;
  return next.map((b) => {
    const old = edited.get(b.index);
    return old
      ? { ...b, original: old.original, reading: old.reading, translated: old.translated, aiOriginal: old.aiOriginal, aiReading: old.aiReading, aiTranslated: old.aiTranslated, isUserEdited: true }
      : b;
  });
}

/** Salin geometri canvas (x/y/w/h/shape) ke translation (match by index).
 *  Syarat sah resize: tanpa ini resize = kosmetik (export baca snapshot lama).
 *  Teks + ai* + flag tak disentuh — hanya geometri + needsWhitePatch
 *  (port 1:1 heuristik Rust, bukan logika baru). */
export function syncGeometryToTranslation(
  boxes: BubbleBox[],
  rows: BubbleTranslation[],
  pageW: number,
  pageH: number,
): BubbleTranslation[] {
  const geo = new Map<number, BubbleBox>();
  for (let i = 0; i < boxes.length; i++) geo.set(i, boxes[i]);
  return rows.map((b, j) => {
    const g = geo.get(b.index) ?? geo.get(j);
    return g
      ? { ...b, x: g.x, y: g.y, w: g.w, h: g.h, shape: g.shape, needsWhitePatch: needsWhitePatch(g.w, g.h, pageW, pageH) }
      : b;
  });
}
