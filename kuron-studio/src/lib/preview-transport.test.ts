import { describe, expect, it } from "vitest";
import canvasSrc from "./CanvasEditor.svelte?raw";
import appSrc from "../App.svelte?raw";
import panelSrc from "./EditorPanel.svelte?raw";

// Kontrak fix-preview-hang §2 + §3, diverifikasi statis dari sumber.
// Alasan pola-sumber (bukan mount): CanvasEditor terikat Konva + DOM +
// Svelte runtime ($effect/onMount) — mount unit murah tidak bermakna;
// test palsu dengan mock Konva hanya menguji mock itu sendiri. Test ini
// assert pada pola kode yang menjamin kontrak — bila pola hilang, gagal
// sadar, bukan silent regress. (Gaya sama dengan canvas-editor.test.ts.)
//
// (ponytail: upgrade ke component-test sungguhan — @testing-library/svelte
// + jsdom + Konva asli — saat repo butuh interaksi canvas; ceiling =
// render pixel, bukan pola string.)
describe("preview transport (fix-preview-hang §2 progressive open)", () => {
  it("openEditor set selectedId langsung tanpa await-before-mount", () => {
    const fn = appSrc.slice(appSrc.indexOf("function openEditor"));
    // \r? — file bisa CRLF di Windows; indexOf("\n  }\n") mepet jadi -1.
    const end = fn.match(/\r?\n  \}\r?\n/);
    expect(end, "akhir function openEditor tidak ditemukan").not.toBeNull();
    const body = fn.slice(0, end!.index! + end![0].length);
    expect(body).toContain("selectedId = id;");
    expect(body).not.toMatch(/await api\.getImagePreview/);
    expect(body).toMatch(/void api\s*\n?\s*\.getImagePreview\(pg\.path, 1600\)/);
  });

  it("resolve full → fullUrls update; gagal → openError tapi thumb tetap", () => {
    const fn = appSrc.slice(appSrc.indexOf("function openEditor"));
    expect(fn).toContain("fullUrls[id] = url;");
    expect(fn).toContain("openError = String(e);");
  });

  it("{#key selectedPage.id} stabil — prop update tidak remount", () => {
    expect(appSrc).toContain("{#key selectedPage.id}");
  });

  it("EditorPanel tampilkan editor bila URL ada walau openError set", () => {
    expect(panelSrc).toContain("{#if openError && !fullImageUrl}");
    // Panel error penuh hanya tanpa URL sama sekali; banner tetap ada.
    // Dicek per bagian (bukan satu regex panjang) supaya retokenisasi UI
    // tidak mematikan guard ini, tapi banner tetap wajib role="alert".
    const banner = panelSrc.slice(panelSrc.indexOf("{#if openError}"));
    expect(banner.slice(0, 200)).toContain("openError}</p>");
    expect(banner.slice(0, 200)).toContain('role="alert"');
  });

  it("App teruskan thumb sebagai fallbackUrl ke EditorPanel", () => {
    expect(appSrc).toContain("fallbackUrl={thumbs[selectedPage.id] ??");
    expect(panelSrc).toContain("fallbackUrl={fallbackUrl}");
  });
});

describe("preview transport (fix-preview-hang §3 defensive load)", () => {
  it("timeout 15s konstanta bernama + timer dibersihkan saat sukses", () => {
    expect(canvasSrc).toContain("const BG_LOAD_TIMEOUT_MS = 15_000;");
    expect(canvasSrc).toContain("bgTimer = setTimeout(");
    const onload = canvasSrc.slice(canvasSrc.indexOf("img.onload = () => {"));
    expect(onload).toContain("clearBgTimer();");
  });

  it("guard token per-request: onload/onerror/timeout mana duluan menang", () => {
    expect(canvasSrc).toContain("let bgToken = 0;");
    expect(canvasSrc).toContain("const my = ++bgToken;");
    expect(canvasSrc).toContain("if (my !== bgToken || requestedUrl !== url) return;");
    // onDestroy invalidasi callback telat + clear timer.
    const destroy = canvasSrc.slice(canvasSrc.indexOf("onDestroy(() => {"));
    expect(destroy).toContain("bgToken++;");
    expect(destroy).toContain("clearBgTimer();");
  });

  it("gagal/timeout → fallbackUrl sekali sebelum menyerah", () => {
    expect(canvasSrc).toContain("fallbackUrl?: string;");
    expect(canvasSrc).toContain('fallbackUrl = ""');
    expect(canvasSrc).toContain("function loadBackground(url: string, triedFallback = false)");
    expect(canvasSrc).toContain("if (!triedFallback && fallbackUrl && fallbackUrl !== url) {");
    expect(canvasSrc).toContain("loadBackground(fallbackUrl, true);");
  });

  it("terminal failure → imgError + tombol retry tanpa remount", () => {
    expect(canvasSrc).toContain("function retryLoad()");
    expect(canvasSrc).toContain("loadBackground(requestedUrl)");
    expect(canvasSrc).toContain("Coba lagi");
    expect(canvasSrc).toContain("onclick={retryLoad}");
  });
});
