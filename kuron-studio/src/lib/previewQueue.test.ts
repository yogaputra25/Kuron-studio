import { describe, expect, it } from "vitest";
import { PREFETCH_MARGIN, PREVIEW_CONCURRENCY, pickNext } from "./previewQueue";
import { summarizePreviews } from "./previewMetrics";

describe("preview queue (grid-preview-perf 1.2)", () => {
  it("konkurensi default 4 dan prefetch 24", () => {
    expect(PREVIEW_CONCURRENCY).toBe(4);
    expect(PREFETCH_MARGIN).toBe(24);
  });

  it("pickNext: kartu terlihat dulu (urutan antrean stabil)", () => {
    const pending = ["a", "b", "c", "d"];
    expect(pickNext(pending, new Set(["c", "a"]), new Set(), () => false)).toBe("a");
    expect(pickNext(pending, new Set(["d"]), new Set(), () => false)).toBe("d");
  });

  it("pickNext: tanpa yang terlihat, prefetch N pertama", () => {
    const pending = ["a", "b", "c"];
    expect(pickNext(pending, new Set(), new Set(), () => false, 2)).toBe("a");
    expect(pickNext(pending, new Set(["z"]), new Set(), () => false, 0)).toBeNull();
  });

  it("pickNext: sibuk/selesai dilewati, habis = null", () => {
    const pending = ["a", "b"];
    expect(pickNext(pending, new Set(["a"]), new Set(["a"]), () => false)).toBe("b");
    expect(pickNext(pending, new Set(["a", "b"]), new Set(), (id) => id !== "x")).toBeNull();
    expect(pickNext([], new Set(["a"]), new Set(), () => false)).toBeNull();
  });
});

describe("preview metrics (grid-preview-perf 1.1)", () => {
  it("summarize: avg/max/total + time-to-first", () => {
    const s = summarizePreviews(
      [
        { ms: 100, bytes: 1000 },
        { ms: 300, bytes: 2000 },
      ],
      90.04,
    );
    expect(s).toEqual({ n: 2, avgMs: 200, maxMs: 300, totalBytes: 3000, timeToFirstMs: 90 });
  });

  it("summarize kosong aman", () => {
    expect(summarizePreviews([], 0)).toEqual({ n: 0, avgMs: 0, maxMs: 0, totalBytes: 0, timeToFirstMs: 0 });
  });
});
