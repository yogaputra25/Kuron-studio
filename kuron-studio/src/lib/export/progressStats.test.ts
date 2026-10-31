import { describe, expect, it } from "vitest";
import type { Page, PageStatus, Project } from "../contracts/types";
import { computeStats } from "./progressStats";

const bubble = () => ({ x: 0, y: 0, w: 1, h: 1, confidence: 1, shape: null, kind: null, tail: null });

const trBubble = (translated: string, index = 0) =>
  ({
    index,
    x: 0,
    y: 0,
    w: 1,
    h: 1,
    shape: null,
    layout: null,
    fontSize: null,
    original: "a",
    reading: "a",
    translated,
    aiOriginal: "a",
    aiReading: "a",
    aiTranslated: translated,
    needsWhitePatch: false,
    isUserEdited: false,
  }) as never;

const page = (
  id: string,
  status: PageStatus,
  bubbles: number,
  translatedBubbles = 0,
): Page =>
  ({
    id,
    path: `${id}.jpg`,
    width: 0,
    height: 0,
    status,
    bubbles: Array(bubbles).fill(null).map(bubble),
    translation: {
      pageId: id,
      targetLang: "id",
      style: "standard",
      model: "m",
      bubbles: Array(bubbles)
        .fill(null)
        .map((_, i) => trBubble(i < translatedBubbles ? "T" : "", i)),
    },
  }) as unknown as Page;

const project = (pages: Page[]): Project =>
  ({ id: "p", name: "p", folders: [{ id: "default", name: "default", pages }] }) as Project;

describe("computeStats", () => {
  it("0 halaman -> semua 0", () => {
    expect(computeStats(project([]))).toEqual({
      done: 0,
      failed: 0,
      pending: 0,
      total: 0,
      pct: 0,
      bubblePct: 0,
    });
  });

  it("semua gagal", () => {
    const s = computeStats(project([page("a", "failed", 2), page("b", "failed", 1)]));
    expect(s).toEqual({ done: 0, failed: 2, pending: 0, total: 2, pct: 0, bubblePct: 0 });
  });

  it("sebagian selesai", () => {
    const s = computeStats(
      project([
        page("a", "translated", 2, 2),
        page("b", "failed", 2, 0),
        page("c", "idle", 0),
        page("d", "detected", 4, 1),
      ]),
    );
    expect(s.done).toBe(1);
    expect(s.failed).toBe(1);
    expect(s.pending).toBe(2);
    expect(s.total).toBe(4);
    expect(s.pct).toBe(25);
    expect(s.bubblePct).toBe(Math.round((3 / 8) * 100));
  });

  it("semua selesai", () => {
    const s = computeStats(project([page("a", "translated", 2, 2), page("b", "translated", 2, 2)]));
    expect(s).toEqual({ done: 2, failed: 0, pending: 0, total: 2, pct: 100, bubblePct: 100 });
  });
});
