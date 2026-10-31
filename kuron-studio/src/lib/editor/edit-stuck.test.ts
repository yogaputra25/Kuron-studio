import { describe, expect, it, vi } from "vitest";
import { useDraftText } from "./useDraftText.svelte.ts";

describe("useDraftText behavioral unit tests", () => {
  it("buffers inputs in draft without committing immediately", () => {
    const commit = vi.fn();
    const draft = useDraftText({
      getRow: () => 0,
      getBubble: () => ({ original: "orig", reading: "read", translated: "trans" }),
      commit,
    });

    draft.input("translated", "new translation text");
    expect(commit).not.toHaveBeenCalled();
    expect(draft.dirty).toBe(true);
    expect(draft.cur?.translated).toBe("new translation text");
  });

  it("commits buffered text on explicit commit() call", () => {
    const commit = vi.fn();
    const draft = useDraftText({
      getRow: () => 0,
      getBubble: () => ({ original: "orig", reading: "read", translated: "trans" }),
      commit,
    });

    draft.input("translated", "edited text");
    draft.commit();

    expect(commit).toHaveBeenCalledWith(0, "translated", "edited text");
    expect(draft.dirty).toBe(false);
  });

  it("revert discards uncommitted draft without calling commit", () => {
    const commit = vi.fn();
    const draft = useDraftText({
      getRow: () => 0,
      getBubble: () => ({ original: "orig", reading: "read", translated: "trans" }),
      commit,
    });

    draft.input("translated", "draft to discard");
    draft.revert();

    expect(commit).not.toHaveBeenCalled();
    expect(draft.dirty).toBe(false);
    expect(draft.cur).toBeNull();
  });

  it("handles IME composition by delaying commit until composition ends", () => {
    const commit = vi.fn();
    const draft = useDraftText({
      getRow: () => 0,
      getBubble: () => ({ original: "orig", reading: "read", translated: "trans" }),
      commit,
    });

    draft.compStart();
    draft.input("translated", "kanji draft");
    draft.commit(); // attempted during composition

    expect(commit).not.toHaveBeenCalled();

    draft.compEnd();
    expect(commit).toHaveBeenCalledWith(0, "translated", "kanji draft");
  });
});
