import { describe, expect, it } from "vitest";
import { mergeUserEdits, syncGeometryToTranslation } from "./translation";
import type { BubbleBox, BubbleTranslation } from "./types";

const mk = (index: number, translated: string, isUserEdited = false, original?: string, reading?: string): BubbleTranslation => ({
  index, x: 0, y: 0, w: 10, h: 10, original: original ?? `o${index}`, reading: reading ?? "",
  translated, aiOriginal: original ?? `o${index}`, aiReading: reading ?? "", aiTranslated: translated,
  needsWhitePatch: false, isUserEdited,
});

const box = (x: number, y: number, w: number, h: number): BubbleBox => ({
  x, y, w, h, confidence: 1, shape: null, kind: "rect", tail: null,
});

describe("mergeUserEdits", () => {
  it("keeps edited, replaces rest", () => {
    const prev = [mk(0, "user-A", true), mk(1, "old-B")];
    const next = [mk(0, "new-A"), mk(1, "new-B")];
    const out = mergeUserEdits(prev, next);
    expect(out[0].translated).toBe("user-A");
    expect(out[0].isUserEdited).toBe(true);
    expect(out[1].translated).toBe("new-B");
    expect(out[1].isUserEdited).toBe(false);
  });
  it("returns next when no prev", () => {
    const next = [mk(0, "x")];
    expect(mergeUserEdits(undefined, next)).toBe(next);
  });
  it("preserves manual original/reading alongside translated", () => {
    const prev = [{ ...mk(0, "user-A", true), original: "manual-O", reading: "manual-R" }];
    const next = [{ ...mk(0, "new-A"), original: "ai-O", reading: "ai-R" }];
    const out = mergeUserEdits(prev, next);
    expect(out[0].original).toBe("manual-O");
    expect(out[0].reading).toBe("manual-R");
    expect(out[0].translated).toBe("user-A");
    expect(out[0].isUserEdited).toBe(true);
  });
  it("unedited bubbles still update", () => {
    const prev = [mk(0, "keep", true), mk(1, "old")];
    const next = [mk(0, "ai0"), mk(1, "ai1")];
    const out = mergeUserEdits(prev, next);
    expect(out[0].translated).toBe("keep");
    expect(out[1].translated).toBe("ai1");
  });
  it("edited bubble keeps old ai* baseline, not rejected new AI", () => {
    const prev = [{ ...mk(0, "user-A", true), aiOriginal: "BASE-O", aiReading: "BASE-R", aiTranslated: "BASE" }];
    const next = [{ ...mk(0, "new-A"), aiOriginal: "new-O", aiReading: "new-R", aiTranslated: "new-A" }];
    const out = mergeUserEdits(prev, next);
    expect(out[0].translated).toBe("user-A");
    expect(out[0].aiOriginal).toBe("BASE-O");
    expect(out[0].aiReading).toBe("BASE-R");
    expect(out[0].aiTranslated).toBe("BASE");
  });
});

describe("syncGeometryToTranslation", () => {
  it("menyalin x/y/w/h/shape by index, teks tak tersentuh", () => {
    const boxes = [box(5, 6, 60, 40)];
    const rows = [{ ...mk(0, "TEKS"), x: 0, y: 0, w: 10, h: 10 }];
    const out = syncGeometryToTranslation(boxes, rows, 800, 1200);
    expect(out[0]).toMatchObject({ x: 5, y: 6, w: 60, h: 40 });
    expect(out[0].translated).toBe("TEKS");
  });

  it("hitung ulang needsWhitePatch dari geometri baru", () => {
    const boxes = [box(0, 0, 500, 500)];
    const rows = [mk(0, "T")];
    const out = syncGeometryToTranslation(boxes, rows, 800, 1200);
    expect(out[0].needsWhitePatch).toBe(true);
  });

  it("shape polygon ikut tersinkron (repair R4)", () => {
    const shape: [number, number][] = [[1, 2], [30, 2], [30, 40]];
    const boxes = [{ ...box(1, 2, 29, 38), shape, kind: "freeform" as const }];
    const rows = [mk(0, "T")];
    const out = syncGeometryToTranslation(boxes, rows, 800, 1200);
    expect(out[0].shape).toEqual(shape);
  });
});
