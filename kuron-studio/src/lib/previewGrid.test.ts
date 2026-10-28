import { describe, expect, it } from "vitest";
import appSrc from "../App.svelte?raw";

// grid-preview-perf: antrean + viewport + guard + progress wajib lewat jalur
// terpusat (previewQueue/previewMetrics), bukan angka tersebar. Pola-sumber
// (gaya preview-transport.test.ts): App terikat Svelte runtime + Tauri invoke,
// mount unit tak bermakna.
describe("grid preview queue wiring (grid-preview-perf)", () => {
  it("App memakai modul antrean + metrik terpusat", () => {
    expect(appSrc).toContain('from "./lib/previewQueue"');
    expect(appSrc).toContain("pickNext");
    expect(appSrc).toContain("PREVIEW_CONCURRENCY");
    expect(appSrc).toContain('from "./lib/previewMetrics"');
    expect(appSrc).toContain("summarizePreviews");
  });

  it("loadThumbs tak lagi Promise.allSettled semua-halaman", () => {
    const fn = appSrc.slice(appSrc.indexOf("function loadThumbs"));
    expect(fn).not.toContain("Promise.allSettled");
  });

  it("generation guard: gen naik tiap load, hasil basi dibuang", () => {
    expect(appSrc).toContain("++loadGen");
    expect(appSrc).toContain("myGen !== loadGen");
  });

  it("IntersectionObserver viewport + disconnect saat unmount", () => {
    expect(appSrc).toContain("new IntersectionObserver");
    expect(appSrc).toContain("rootMargin");
    expect(appSrc).toContain("io.disconnect()");
    expect(appSrc).toContain("use:observeCard");
  });

  it("progress jujur: label Memuat n/m + retry yang gagal", () => {
    expect(appSrc).toContain("loadingPreviews");
    expect(appSrc).toContain("thumbDone");
    expect(appSrc).toContain("thumbTotal");
    expect(appSrc).toContain("retryThumbs");
  });
});
