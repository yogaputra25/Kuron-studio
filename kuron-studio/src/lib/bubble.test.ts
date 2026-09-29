import { describe, expect, it } from "vitest";
import { chipNumbers, clampBubble, fitFontSize, readingOrder } from "./bubble";
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

describe("fitFontSize", () => {
  // Tinggi blok = baris * lineHeight * fs. Mirror perilaku Konva.
  const h = (lines: number, lh = 1.25) => (fs: number) => lines * lh * fs;

  it("memilih ukuran maksimal kalau teks sudah muat", () => {
    expect(fitFontSize(400, h(2), { min: 7, max: 28 })).toBe(28);
  });

  it("mengecilkan sampai tinggi blok muat di kotak", () => {
    // 10 baris * 1.25 * fs <= 100  =>  fs <= 8
    const fs = fitFontSize(100, h(10), { min: 7, max: 28 });
    expect(fs).toBeLessThanOrEqual(8.5);
    expect(fs).toBeGreaterThanOrEqual(7.5);
    expect(h(10)(fs)).toBeLessThanOrEqual(100.5);
  });

  it("tidak pernah turun di bawah min walaupun tetap tidak muat", () => {
    expect(fitFontSize(10, h(50), { min: 7, max: 28 })).toBe(7);
  });

  it("bubble besar memaksa font lebih besar, bukan nilai beku", () => {
    const kecil = fitFontSize(40, h(6), { min: 7, max: 28 });
    const besar = fitFontSize(400, h(6), { min: 7, max: 28 });
    expect(besar).toBeGreaterThan(kecil);
  });

  it("step 0.5: hasil kelipatan setengah piksel dari min", () => {
    const fs = fitFontSize(100, h(10), { min: 7, max: 28, step: 0.5 });
    expect(Number.isInteger((fs - 7) / 0.5)).toBe(true);
  });

  it("toleransi: teks yang PAS muat tidak ikut mengecil", () => {
    // 8 baris * 1.25 * 10 = 100 == boxH persis -> harus tetap 10.
    const fs = fitFontSize(100, h(8), { min: 7, max: 10 });
    expect(fs).toBe(10);
  });

  it("step tidak valid tidak bikin loop tak berujung", () => {
    expect(fitFontSize(1, h(50), { min: 7, max: 28, step: 0 })).toBe(7);
  });
});
