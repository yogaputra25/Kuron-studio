import { describe, expect, it } from "vitest";
import {
  CANVAS_FLOOR,
  COLLAPSE_THRESHOLD,
  SHEET_REGION_PCT,
  SIDEBAR_DEFAULT,
  SIDEBAR_MAX,
  SIDEBAR_MIN,
  SNAP_DEFAULT,
  clampWidth,
  nearestSnap,
  sheetHeightPct,
  sidebarMax,
} from "./panel";

describe("panel layout pure logic (full-range)", () => {
  it("clampWidth dinamis: MIN 192, MAX bodyW-280", () => {
    expect(clampWidth(100, 1200)).toBe(SIDEBAR_MIN);
    expect(clampWidth(1000, 1200)).toBe(1200 - CANVAS_FLOOR);
    expect(clampWidth(300.6, 1200)).toBe(301);
    expect(clampWidth(304, 1200)).toBe(SIDEBAR_DEFAULT);
    expect(sidebarMax(1200)).toBe(1200 - CANVAS_FLOOR);
    expect(sidebarMax(400)).toBe(SIDEBAR_MIN);
  });

  it("clampWidth tanpa bodyW fallback ke SIDEBAR_MAX", () => {
    expect(clampWidth(900)).toBe(SIDEBAR_MAX);
    expect(clampWidth(100)).toBe(SIDEBAR_MIN);
  });

  it("clampWidth fallback default untuk non-finite", () => {
    expect(clampWidth(NaN, 1200)).toBe(SIDEBAR_DEFAULT);
    expect(clampWidth(Infinity, 1200)).toBe(SIDEBAR_DEFAULT);
  });

  it("threshold collapse 120 di bawah MIN 192", () => {
    expect(COLLAPSE_THRESHOLD).toBeLessThan(SIDEBAR_MIN);
  });

  it("nearestSnap memilih titik terdekat dari [40, 70, 100]", () => {
    expect(nearestSnap(40)).toBe(40);
    expect(nearestSnap(56)).toBe(70);
    expect(nearestSnap(54)).toBe(40);
    expect(nearestSnap(90)).toBe(100);
    expect(nearestSnap(-10)).toBe(40);
    expect(nearestSnap(200)).toBe(100);
    expect(nearestSnap(NaN)).toBe(SNAP_DEFAULT);
  });

  it("nearestSnap seri titik-tengah dimenangkan titik bawah (deterministik)", () => {
    expect(nearestSnap(55)).toBe(40);
  });

  it("sheet full boleh menutup canvas: 100% snap = 100% kontainer", () => {
    expect(SHEET_REGION_PCT).toBe(100);
    expect(sheetHeightPct(100)).toBe(100);
    expect(sheetHeightPct(40)).toBe(40);
  });
});
