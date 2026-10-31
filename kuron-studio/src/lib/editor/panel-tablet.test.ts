import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import deviceSrc from "../app/device?raw";
import hookSrc from "./usePanelResize.svelte?raw";
import dividerSrc from "../ui/Divider.svelte?raw";

import gridSrc from "../library/PageGrid.svelte?raw";

// panel-resize-tablet: layout resizable + mode tablet wajib lewat jalur
// terpusat (device.ts + panel.ts), bukan angka/UA-sniff tersebar.
// Sejak FE-4 (3.1): jalur terpusat tinggal di usePanelResize, bukan panel.
describe("resizable panel + tablet mode wiring", () => {
  it("EditorPanel memakai device.ts terpusat", () => {
    expect(hookSrc).toContain('from "../app/device"');
    expect(hookSrc).toContain("isNarrow");
    expect(hookSrc).toContain("watchViewport");
    expect(hookSrc).toContain("loadSidebarW");
    expect(hookSrc).toContain("saveSidebarW");
    expect(panelSrc).toContain("usePanelResize");
  });

  it("EditorPanel memakai logika murni panel.ts (clamp + snap)", () => {
    expect(hookSrc).toContain('from "./panel"');
    expect(hookSrc).toContain("clampWidth");
    expect(hookSrc).toContain("nearestSnap");
  });

  it("divider punya role separator + keyboard handler", () => {
    expect(dividerSrc).toContain('role="separator"');
    expect(hookSrc).toContain("function key(e: KeyboardEvent)");
    expect(hookSrc).toContain("setPointerCapture");
    expect(panelSrc).toContain("Divider");
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
