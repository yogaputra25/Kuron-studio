import { describe, expect, it, vi } from "vitest";
import { createSelection, orderIds } from "./selection.svelte.ts";
import type { Page } from "../contracts/types";

vi.useRealTimers();
const pg = (id: string): Page => ({ id, path: `${id}.jpg` }) as Page;

const click = (init: Partial<MouseEvent> = {}): MouseEvent =>
  ({
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    preventDefault: () => {},
    ...init,
  }) as unknown as MouseEvent;

describe("selection", () => {
  it("klik biasa tidak memilih, kembalikan false (komposisi buka editor)", () => {
    const s = createSelection();
    expect(s.click(click(), pg("a"), ["a"])).toBe(false);
    expect(s.selectedIds).toEqual([]);
    expect(s.selectMode).toBe(false);
  });

  it("ctrl toggle tambah dan hapus, anchor mengikuti", () => {
    const s = createSelection();
    expect(s.click(click({ ctrlKey: true }), pg("a"), ["a"])).toBe(true);
    expect(s.selectedIds).toEqual(["a"]);
    expect(s.selAnchor).toBe("a");
    expect(s.selectMode).toBe(true);
    s.click(click({ ctrlKey: true }), pg("a"), ["a"]);
    expect(s.selectedIds).toEqual([]);
  });

  it("shift memilih rentang tanpa duplikat", () => {
    const s = createSelection();
    s.toggleSelect("a");
    expect(s.click(click({ shiftKey: true }), pg("c"), ["a", "b", "c"])).toBe(true);
    expect(s.selectedIds).toEqual(["a", "b", "c"]);
    expect(s.selAnchor).toBe("c");
  });

  it("long-press masuk mode + pilih, klik sesudahnya dikonsumsi", async () => {
    const s = createSelection();
    s.press(pg("a"));
    expect(s.selectMode).toBe(false);
    await new Promise((r) => setTimeout(r, 600));
    expect(s.selectMode).toBe(true);
    expect(s.selectedIds).toEqual(["a"]);
    // Lepas-tahan: klik pertama dikonsumsi, bukan buka editor.
    expect(s.click(click(), pg("a"), ["a"])).toBe(true);
    expect(s.click(click(), pg("a"), ["a"])).toBe(false);
  });

  it("press diabaikan saat sudah mode pilih", async () => {
    const s = createSelection();
    s.setSelectMode(true);
    s.press(pg("a"));
    await new Promise((r) => setTimeout(r, 600));
    expect(s.selectedIds).toEqual([]);
  });

  it("reset mengosongkan semua", () => {
    const s = createSelection();
    s.toggleSelect("a");
    s.reset();
    expect(s.selectedIds).toEqual([]);
    expect(s.selAnchor).toBeNull();
    expect(s.selectMode).toBe(false);
  });

  it("dispose membatalkan timer long-press", async () => {
    const s = createSelection();
    s.press(pg("a"));
    s.dispose();
    await new Promise((r) => setTimeout(r, 600));
    expect(s.selectMode).toBe(false);
    expect(s.selectedIds).toEqual([]);
  });

  it("orderIds mengikuti urutan tampil (sort nama)", () => {
    expect(orderIds([pg("b"), pg("a")])).toEqual(["a", "b"]);
  });
});
