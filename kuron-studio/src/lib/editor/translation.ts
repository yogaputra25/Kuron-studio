import type { BubbleBox, BubbleTranslation } from "../contracts/types";
import { needsWhitePatch } from "./bubble";
import type { TextLayout } from "./text/textLayout";

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

/** Hitung layout teks final dari teks + geometri, dalam px halaman.
 *  Disuntik dari luar supaya modul ini tetap bebas Konva/DOM dan bisa diuji
 *  di node; `CanvasEditor` menyediakan versi Konva-nya. */
export type LayoutFor = (
  text: string,
  w: number,
  h: number,
  fontSize?: number | null,
  shape?: Array<[number, number]> | null,
) => TextLayout;

/** Salin geometri canvas (x/y/w/h/shape) ke translation (match by index).
 *  Syarat sah resize: tanpa ini resize = kosmetik (export baca snapshot lama).
 *  `layout` ikut dihitung ulang di sini — teks atau ukuran bisa sudah berubah,
 *  jadi layout lama tak boleh dibiarkan. Teks + ai* + flag tak disentuh. */
export function syncGeometryToTranslation(
  boxes: BubbleBox[],
  rows: BubbleTranslation[],
  pageW: number,
  pageH: number,
  layoutFor?: LayoutFor,
): BubbleTranslation[] {
  const geo = new Map<number, BubbleBox>();
  for (let i = 0; i < boxes.length; i++) geo.set(i, boxes[i]);
  return rows.map((b, j) => {
    const g = geo.get(b.index) ?? geo.get(j);
    return g
      ? {
          ...b,
          x: g.x, y: g.y, w: g.w, h: g.h,
          shape: g.shape,
          needsWhitePatch: needsWhitePatch(g.w, g.h, pageW, pageH),
          layout: layoutFor ? layoutFor(b.translated, g.w, g.h, b.fontSize, g.shape && g.shape.length >= 3 ? g.shape.map(([px, py]) => [px - g.x, py - g.y] as [number, number]) : null) : (b.layout ?? null),
        }
      : b;
  });
}
