import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";

// Kontrak change reset-translation (opsi A), diverifikasi statis dari sumber
// (pola sama seperti manual-text-split.test.ts): assert pada pola kode.
//
// (ponytail: upgrade ke component-test sungguhan — @testing-library/svelte +
// jsdom — saat repo butuh interaksi nyata; ceiling = klik ↺ beneran.)
describe("reset-translation (opsi A)", () => {
  it("resetBubble salin ai*→current + flag false", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function resetBubble"), panelSrc.indexOf("function resetBubble") + 400);
    expect(fn).toContain("original: b.aiOriginal");
    expect(fn).toContain("reading: b.aiReading");
    expect(fn).toContain("translated: b.aiTranslated");
    expect(fn).toContain("isUserEdited: false");
  });

  it("tombol ↺ hanya tampil bila edited + baseline non-kosong", () => {
    expect(panelSrc).toContain("{#if b.isUserEdited && b.aiTranslated}");
    expect(panelSrc).toContain("onclick={() => resetBubble(i)}");
  });

  it("reset kirim langsung, bukan via debounce (§4 D7)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function resetBubble"), panelSrc.indexOf("function resetBubble") + 800);
    expect(fn).toContain("void saveTranslationEdits();");
    expect(fn).not.toContain("queueSaveTranslationEdits()");
    expect(fn).not.toContain("translate_page");
    expect(fn).not.toContain("retryBubble");
  });

  it("Isi-manual kosong ikut bawa ai* kosong (zaman baseline)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("async function createEmptyTranslation"), panelSrc.indexOf("async function createEmptyTranslation") + 1200);
    expect(fn).toContain('aiOriginal: "", aiReading: "", aiTranslated: ""');
  });
});
