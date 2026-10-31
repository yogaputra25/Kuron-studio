import { describe, expect, it } from "vitest";
import panelSrc from "./editor/EditorPanel.svelte?raw";
import deviceSrc from "./app/device?raw";

import gridSrc from "./library/PageGrid.svelte?raw";

// panel-resize-tablet: layout resizable + mode tablet wajib lewat jalur
// terpusat (device.ts + panel.ts), bukan angka/UA-sniff tersebar.
describe("resizable panel + tablet mode wiring", () => {
  it("EditorPanel memakai device.ts terpusat", () => {
    expect(panelSrc).toContain('from "../app/device"');
    expect(panelSrc).toContain("isNarrow");
    expect(panelSrc).toContain("watchViewport");
    expect(panelSrc).toContain("loadSidebarW");
    expect(panelSrc).toContain("saveSidebarW");
  });

  it("EditorPanel memakai logika murni panel.ts (clamp + snap)", () => {
    expect(panelSrc).toContain('from "./panel"');
    expect(panelSrc).toContain("clampWidth");
    expect(panelSrc).toContain("nearestSnap");
  });

  it("divider punya role separator + keyboard handler", () => {
    expect(panelSrc).toContain('role="separator"');
    expect(panelSrc).toContain("onDividerKey");
    expect(panelSrc).toContain("setPointerCapture");
  });

  it("sidebar lama w-[19rem] fixed sudah tidak ada", () => {
    expect(panelSrc).not.toContain("w-[19rem]");
  });

  it("grid tablet = 3 kolom di md (PageGrid, pindahan App utuh)", () => {
    // page-list-view 2.3: markup grid pindah ke PageGrid — perilaku sama.
    expect(gridSrc).toContain("md:grid-cols-3");
  });

  it("device.ts tidak mengandalkan UA untuk tablet", () => {
    expect(deviceSrc).toContain("matchMedia");
    expect(deviceSrc).toContain("pointer");
    expect(deviceSrc).toContain("isTablet");
  });
});
