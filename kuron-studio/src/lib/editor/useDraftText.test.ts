// Unit test mesin draft (FE-4, task 1.3): perilaku inti tanpa DOM.
// Node env cukup — hook murni logika + setTimeout, tanpa Svelte runtime.
import { afterEach, describe, expect, it, vi } from "vitest";
import { DRAFT_SAFETY_MS, useDraftText, type DraftField } from "./useDraftText.svelte";

function setup() {
  const bubbles = [
    { original: "O0", reading: "R0", translated: "T0" },
    { original: "O1", reading: "R1", translated: "T1" },
  ];
  const committed: Array<[number, DraftField, string]> = [];
  let row = 0;
  const draft = useDraftText({
    getRow: () => row,
    getBubble: (r) => bubbles[r],
    commit: (r, f, v) => { committed.push([r, f, v]); },
  });
  return { bubbles, committed, draft, setRow: (r: number) => { row = r; } };
}

afterEach(() => {
  vi.useRealTimers();
});

describe("useDraftText", () => {
  it("timer commit: jaring 10 dtk flush ketikan tanpa blur", () => {
    vi.useFakeTimers();
    const { draft, committed } = setup();
    draft.input("translated", "T0-baru");
    expect(draft.cur?.translated).toBe("T0-baru");
    expect(draft.dirty).toBe(true);
    expect(committed).toEqual([]);
    vi.advanceTimersByTime(DRAFT_SAFETY_MS);
    expect(committed).toEqual([[0, "translated", "T0-baru"]]);
    expect(draft.cur).toBeNull();
    expect(draft.dirty).toBe(false);
  });

  it("composing tak commit: cache menampung, commit di compositionend", () => {
    vi.useFakeTimers();
    const { draft, committed } = setup();
    draft.compStart();
    draft.input("translated", "T0-ime");
    expect(draft.cur?.translated).toBe("T0-ime");
    vi.advanceTimersByTime(DRAFT_SAFETY_MS);
    expect(committed).toEqual([]);
    draft.compEnd();
    expect(committed).toEqual([[0, "translated", "T0-ime"]]);
    expect(draft.cur).toBeNull();
  });

  it("Esc revert: buang cache tanpa commit, dirty padam", () => {
    vi.useFakeTimers();
    const { draft, committed } = setup();
    draft.input("original", "O0-baru");
    expect(draft.dirty).toBe(true);
    draft.revert();
    expect(draft.cur).toBeNull();
    expect(draft.dirty).toBe(false);
    vi.advanceTimersByTime(DRAFT_SAFETY_MS);
    expect(committed).toEqual([]);
  });

  it("hanya field berubah yang di-commit, sekali per batas", () => {
    vi.useFakeTimers();
    const { bubbles, draft, committed } = setup();
    draft.input("translated", "T0-baru");
    draft.input("original", bubbles[0].original);
    draft.commit();
    expect(committed).toEqual([[0, "translated", "T0-baru"]]);
  });

  it("tanpa baris/bubble: input dan commit no-op", () => {
    const { draft, committed, setRow } = setup();
    setRow(-1);
    draft.input("translated", "x");
    expect(draft.cur).toBeNull();
    draft.commit();
    expect(committed).toEqual([]);
  });
});
