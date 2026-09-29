import type { BubbleTranslation } from "./types";

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
