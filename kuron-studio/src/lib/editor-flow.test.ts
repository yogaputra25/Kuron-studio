import { describe, expect, it } from "vitest";
import { chipNumbers, clampBubble, readingOrder } from "./bubble";
import type { BubbleBox } from "./types";

const b = (x: number, y: number, w = 60, h = 40): BubbleBox => ({
  x, y, w, h, confidence: 1, shape: null, kind: "rect", tail: null,
});

// M1-12: import 1 halaman → 5 bubbles di canvas → drag/resize/delete.
describe("editor flow (M1-12)", () => {
  it("5 bubbles → drag → resize → delete → 4 tersisa, chip konsisten", () => {
    let bubbles = [b(300, 20), b(100, 20), b(300, 200), b(100, 200), b(200, 350)];
    expect(bubbles).toHaveLength(5);

    // Drag bubble pertama keluar bounds → kejepit.
    bubbles[0] = clampBubble({ ...bubbles[0], x: -50, y: -10 }, 800, 1200);
    expect(bubbles[0]).toMatchObject({ x: 0, y: 0 });

    // Resize bubble kedua jadi kecil → min 1px, masih valid.
    bubbles[1] = clampBubble({ ...bubbles[1], w: 120, h: 80 }, 800, 1200);
    expect(bubbles[1]).toMatchObject({ w: 120, h: 80 });

    // Delete satu bubble.
    bubbles.splice(2, 1);
    expect(bubbles).toHaveLength(4);

    // Chip RTL tetap 1-based dan unik.
    const nums = chipNumbers(bubbles, "rtl");
    expect([...nums.values()].sort()).toEqual([1, 2, 3, 4]);

    // Urutan baca: baris atas dulu, kanan dulu (RTL).
    const ordered = readingOrder(bubbles, "rtl");
    expect(ordered[0].y).toBeLessThanOrEqual(ordered.at(-1)!.y);
  });
});
