import { describe, expect, it } from "vitest";
import { detectPercent, initialDetectState, reduceDetectProgress } from "./detectProgress";

describe("detect progress (detect-ux-pack 1.1)", () => {
  it("counter + persen + file berjalan bertambah per event", () => {
    let s = initialDetectState(25);
    s = reduceDetectProgress(s, { pageId: "p3", done: 3, total: 25, ok: true, message: "", file: "page-003.png" });
    expect(s.done).toBe(3);
    expect(s.total).toBe(25);
    expect(s.file).toBe("page-003.png");
    expect(detectPercent(s)).toBe(12);
    expect(s.finished).toBe(false);
  });

  it("gagal terkumpul tanpa menghentikan agregasi", () => {
    let s = initialDetectState(3);
    s = reduceDetectProgress(s, { pageId: "p1", done: 1, total: 3, ok: true, message: "", file: "a.png" });
    s = reduceDetectProgress(s, { pageId: "p2", done: 2, total: 3, ok: false, message: "decode failed", file: "b.png" });
    expect(s.failed).toEqual([{ pageId: "p2", file: "b.png", message: "decode failed" }]);
    s = reduceDetectProgress(s, { pageId: "p3", done: 3, total: 3, ok: true, message: "", file: "c.png" });
    expect(s.done).toBe(3);
    expect(s.finished).toBe(true);
    expect(detectPercent(s)).toBe(100);
  });

  it("total 0 aman (tanpa bagi-nol)", () => {
    expect(detectPercent({ done: 0, total: 0 })).toBe(0);
    const s = reduceDetectProgress(initialDetectState(0), {
      pageId: "x",
      done: 0,
      total: 0,
      ok: true,
      message: "",
      file: "",
    });
    expect(s.finished).toBe(false);
  });
});
