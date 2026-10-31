import { describe, expect, it } from "vitest";
import appSrc from "../../App.svelte?raw";
import gridSrc from "./PageGrid.svelte?raw";
import listSrc from "./PageList.svelte?raw";
import queueSrc from "../previews/thumbnailQueue.svelte.ts?raw";

// grid-preview-perf: antrean + viewport + guard + progress wajib lewat jalur
// terpusat (previewQueue/previewMetrics), bukan angka tersebar. Pola-sumber
// (gaya preview-transport.test.ts): App terikat Svelte runtime + Tauri invoke,
// mount unit tak bermakna.
describe("grid preview queue wiring (grid-preview-perf)", () => {
  it("App memakai modul antrean + metrik terpusat", () => {
    expect(appSrc).toContain('from "./lib/previews/thumbnailQueue.svelte.ts"');
    expect(appSrc).toContain("createThumbnailQueue");
    expect(appSrc).toContain("queue.loadThumbs");
  });

  it("loadThumbs tak lagi Promise.allSettled semua-halaman", () => {
    const fn = appSrc.slice(appSrc.indexOf("function loadThumbs"));
    expect(fn).not.toContain("Promise.allSettled");
  });

  it("generation guard: gen naik tiap load, hasil basi dibuang", () => {
    expect(queueSrc).toContain("++loadGen");
    expect(queueSrc).toContain("myGen !== loadGen");
  });

  it("IntersectionObserver viewport + disconnect saat unmount", () => {
    expect(queueSrc).toContain("new IntersectionObserver");
    expect(queueSrc).toContain("rootMargin");
    expect(queueSrc).toContain("io.disconnect()");
    expect(gridSrc).toContain("use:observeCard");
    expect(listSrc).toContain("use:observeCard");
  });

  it("progress jujur: label Memuat n/m + retry yang gagal", () => {
    expect(appSrc).toContain("loadingPreviews");
    expect(appSrc).toContain("queue.done");
    expect(appSrc).toContain("queue.total");
    expect(appSrc).toContain("retryThumbs");
  });

});
