import { describe, expect, it } from "vitest";
import { mergeUserEdits } from "./translation";
import type { BubbleTranslation } from "./types";

const mk = (index: number, translated: string, isUserEdited = false, original?: string, reading?: string): BubbleTranslation => ({
  index, x: 0, y: 0, w: 10, h: 10, original: original ?? `o${index}`, reading: reading ?? "",
  translated, aiOriginal: original ?? `o${index}`, aiReading: reading ?? "", aiTranslated: translated,
  needsWhitePatch: false, isUserEdited,
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
