import { describe, expect, it } from "vitest";
import batchSrc from "./BatchPanel.svelte?raw";
import panelSrc from "./TranslatePanel.svelte?raw";
import reviewSrc from "../ReviewGrid.svelte?raw";

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

describe("translate model per-run (LOV sesi-saja)", () => {
  it("pilih provider → auto list models, default = simpanan", () => {
    expect(panelSrc).toContain("loadModelsFor(providerId)");
    expect(panelSrc).toContain("api.listModels(pid)");
    expect(panelSrc).toMatch(/models\.find\(\(m\) => m\.vision\)/);
  });

  it("model masuk opts + dikirim sebagai override", () => {
    expect(panelSrc).toMatch(/interface TranslateOpts \{[\s\S]*?model: string/);
    expect(panelSrc).toContain("model: model.trim()");
  });

  it("pilihan model tak menulis DB (tanpa saveProvider)", () => {
    expect(panelSrc).not.toContain("saveProvider");
  });
});

describe("translate model per-run (forwarding batch/retry)", () => {
  it("batch + retry teruskan model dari opts", () => {
    for (const [name, src] of [["BatchPanel", batchSrc], ["ReviewGrid", reviewSrc]] as const) {
      expect(src, name).toContain("...(opts.model");
    }
  });
});

describe("translate kartu regroup (hierarki)", () => {
  it("Model di bawah Provider sebelum Target, opsi lanjut di details, tombol sticky", () => {
    const modelAt = panelSrc.indexOf('label="Model"');
    const targetAt = panelSrc.indexOf('label="Target"');
    expect(modelAt, "Model select ada").toBeGreaterThan(-1);
    expect(modelAt, "Model sebelum Target").toBeLessThan(targetAt);
    expect(panelSrc).toContain("<details");
    expect(panelSrc).toContain("sticky bottom-0");
  });
});
