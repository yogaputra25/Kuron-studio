import { describe, expect, it } from "vitest";
import { mergeUserEdits } from "./translation";
import type { BubbleTranslation } from "./types";

const mk = (index: number, translated: string, isUserEdited = false): BubbleTranslation => ({
  index, x: 0, y: 0, w: 10, h: 10, original: `o${index}`, reading: "",
  translated, needsWhitePatch: false, isUserEdited,
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
});
