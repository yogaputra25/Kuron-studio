import { describe, expect, it } from "vitest";
import panelSrc from "./TranslatePanel.svelte?raw";

// Kontrak change reset-translation §4 cancel (opsi B), diverifikasi statis
// dari sumber (pola sama seperti reset-translation.test.ts): assert pola kode.
//
// (ponytail: upgrade ke component-test sungguhan — @testing-library/svelte +
// jsdom — saat repo butuh interaksi nyata; ceiling = klik Batal beneran.)
describe("translate-cancel (opsi B)", () => {
  it("tombol jadi Batal saat busy + panggil cancel", () => {
    expect(panelSrc).toContain('{busy ? "Batal" : "Translate"}');
    expect(panelSrc).toContain("onclick={busy ? cancel : translate}");
    expect(panelSrc).toContain("async function cancel()");
    expect(panelSrc).toContain("api.cancelTranslate");
  });

  it("cancel lepas UI seketika + info Dibatalkan", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("async function cancel()"), panelSrc.indexOf("async function cancel()") + 300);
    expect(fn).toContain("cancelled = true");
    expect(fn).toContain("busy = false");
    expect(fn).toContain('info = "Dibatalkan"');
  });

  it("response telat diabaikan: tanpa onTranslated tanpa banner", () => {
    expect(panelSrc).toContain("if (cancelled) return;");
    const tr = panelSrc.slice(panelSrc.indexOf("async function translate()"), panelSrc.indexOf("async function translate()") + 1200);
    expect(tr).toContain("cancelled = false");
    expect(tr).toContain("if (!cancelled) busy = false");
  });
});
