import { describe, expect, it } from "vitest";
import { chipNumbers, clampBubble, readingOrder } from "./bubble";
import type { BubbleBox } from "./types";

const b = (x: number, y: number, w = 50, h = 50): BubbleBox => ({
  x, y, w, h, confidence: 1, shape: null, kind: null, tail: null,
});

describe("readingOrder", () => {
  it("rtl: kanan dulu dalam satu baris", () => {
    const out = readingOrder([b(0, 0), b(200, 0)], "rtl");
    expect(out[0].x).toBe(200);
  });

  it("ltr: kiri dulu dalam satu baris", () => {
    const out = readingOrder([b(200, 0), b(0, 0)], "ltr");
    expect(out[0].x).toBe(0);
  });

  it("baris atas selalu dulu", () => {
    const out = readingOrder([b(200, 200), b(0, 0)], "rtl");
    expect(out[0].y).toBe(0);
  });
});

describe("chipNumbers", () => {
  it("1-based sesuai urutan baca", () => {
    const nums = chipNumbers([b(0, 0), b(200, 0)], "rtl");
    expect(nums.get(1)).toBe(1);
    expect(nums.get(0)).toBe(2);
  });
});

describe("clampBubble", () => {
  it("menjepit ke bounds", () => {
    const out = clampBubble(b(-10, -5, 50, 50), 200, 200);
    expect(out).toMatchObject({ x: 0, y: 0, w: 50, h: 50 });
    const out2 = clampBubble(b(180, 180, 50, 50), 200, 200);
    expect(out2.w).toBe(20);
    expect(out2.h).toBe(20);
  });
});
