import type { BubbleTranslation } from "./types";

/** Gabung hasil translate baru dengan edit manual user (match by index). */
export function mergeUserEdits(
  prev: BubbleTranslation[] | undefined,
  next: BubbleTranslation[],
): BubbleTranslation[] {
  if (!prev || prev.length === 0) return next;
  const edited = new Map<number, string>();
  for (const b of prev) if (b.isUserEdited) edited.set(b.index, b.translated);
  if (edited.size === 0) return next;
  return next.map((b) =>
    edited.has(b.index)
      ? { ...b, translated: edited.get(b.index)!, isUserEdited: true }
      : b,
  );
}
