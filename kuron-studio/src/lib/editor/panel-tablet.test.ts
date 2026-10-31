// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import Divider from "../ui/Divider.svelte";
import {
  clampWidth,
  nearestSnap,
  sidebarMax,
  sheetHeightPct,
  SIDEBAR_MIN,
  SIDEBAR_MAX,
  SIDEBAR_DEFAULT,
} from "./panel";

describe("panel layout pure functions", () => {
  it("clampWidth respects MIN and MAX bounds", () => {
    expect(clampWidth(100)).toBe(SIDEBAR_MIN);
    expect(clampWidth(600)).toBe(SIDEBAR_MAX);
    expect(clampWidth(350)).toBe(350);
    expect(clampWidth(Number.NaN)).toBe(SIDEBAR_DEFAULT);
  });

  it("sidebarMax adapts dynamically to body width", () => {
    expect(sidebarMax(1000)).toBe(720); // 1000 - 280
    expect(sidebarMax()).toBe(SIDEBAR_MAX);
    expect(sidebarMax(0)).toBe(SIDEBAR_MAX);
  });

  it("nearestSnap rounds to closest predefined snap point", () => {
    expect(nearestSnap(35)).toBe(40);
    expect(nearestSnap(60)).toBe(70);
    expect(nearestSnap(95)).toBe(100);
  });

  it("sheetHeightPct returns container percentage", () => {
    expect(sheetHeightPct(40)).toBe(40);
    expect(sheetHeightPct(100)).toBe(100);
  });
});

describe("Divider component behavior", () => {
  it("renders role separator with orientation and handles keyboard events", async () => {
    const onKey = vi.fn();
    const { getByRole } = renderWithI18n(Divider, {
      props: {
        orientation: "vertical",
        onDown: vi.fn(),
        onMove: vi.fn(),
        onUp: vi.fn(),
        onKey,
      },
    });

    const divider = getByRole("separator");
    expect(divider.getAttribute("aria-orientation")).toBe("vertical");

    await fireEvent.keyDown(divider, { key: "ArrowLeft" });
    expect(onKey).toHaveBeenCalled();
  });
});
