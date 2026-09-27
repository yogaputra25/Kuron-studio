import { describe, expect, it } from "vitest";
import { initialBatchState, reduceProgress } from "./batch";

const ev = (pageId: string, done: number, ok: boolean, message = "") => ({
  pageId, done, total: 3, ok, message,
});

describe("reduceProgress", () => {
  it("tracks done + failed ids", () => {
    let s = initialBatchState(3);
    s = reduceProgress(s, ev("a", 1, true, "2 bubble"));
    expect(s.done).toBe(1);
    expect(s.failed).toEqual([]);
    s = reduceProgress(s, ev("b", 2, false, "boom"));
    expect(s.failed).toEqual(["b"]);
    expect(s.messages["b"]).toBe("boom");
    // retry sukses menghapus dari failed
    s = reduceProgress(s, ev("b", 3, true, "ok"));
    expect(s.failed).toEqual([]);
    expect(s.done).toBe(3);
  });
});
