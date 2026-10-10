import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import canvasSrc from "./CanvasEditor.svelte?raw";

// Kontrak change fix-pane-switch-editor, pola-sumber seperti edit-in-bubble.test.ts:
// Konva + Svelte runtime → mount unit murah tak bermakna; assert pola kode.
// (ponytail: @testing-library/svelte + jsdom + Konva asli saat butuh
// interaksi nyata; ceiling = buka editor → ganti tab → textarea tak basi.)
describe("pane-switch commit (§1)", () => {
  it("commitEditor di-export (kanal commit panel → After)", () => {
    expect(canvasSrc).toContain("export function commitEditor()");
  });

  it("afterRef ter-bind di instance After, terpisah dari editorRef Before", () => {
    expect(panelSrc).toContain("let afterRef = $state<{ commitEditor: () => void; redraw: () => void } | null>(null)");
    // Satu bind per instance: Before = editorRef, After = afterRef.
    const binds = panelSrc.match(/bind:this=\{(editorRef|afterRef)\}/g) ?? [];
    expect(binds).toContain("bind:this={editorRef}");
    expect(binds).toContain("bind:this={afterRef}");
    expect(binds.length).toBe(2);
  });

  it("setMode commit synchronous sebelum mode berubah", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function setMode"), panelSrc.indexOf("function setMode") + 700);
    expect(fn).toContain("afterRef?.commitEditor();");
    expect(fn.indexOf("afterRef?.commitEditor();")).toBeLessThan(fn.indexOf("mode = m;"));
    // Switcher lewat setMode, bukan assign langsung.
    expect(panelSrc).toContain("onclick={() => setMode(m as PaneMode)}");
  });
  it("redraw di-export + sinkronkan background (anti split-brain)", () => {
    expect(canvasSrc).toContain("export function redraw()");
    const fn = canvasSrc.slice(canvasSrc.indexOf("export function redraw()"), canvasSrc.indexOf("export function redraw()") + 600);
    expect(fn).toContain("bgImage.size({ width: imgW * s, height: imgH * s })");
  });

  it("setMode redraw pasca-layout via rAF untuk kedua pane", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function setMode"), panelSrc.indexOf("function setMode") + 700);
    expect(fn).toContain("requestAnimationFrame(");
    expect(fn).toContain("editorRef?.redraw();");
    expect(fn).toContain("afterRef?.redraw();");
    expect(fn.indexOf("requestAnimationFrame(")).toBeGreaterThan(fn.indexOf("mode = m;"));
  });
});
