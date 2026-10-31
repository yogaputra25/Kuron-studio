import { describe, expect, it } from "vitest";
import appSrc from "../../App.svelte?raw";
import cardSrc from "./shell/ProjectCard.svelte?raw";

// pintu-ekspor: satu pintu per kartu dashboard ke panel yang sama.
// Pola-sumber (gaya preview-transport.test.ts): App terikat Svelte runtime,
// mount unit tak bermakna.
describe("dashboard results entry (pintu-ekspor)", () => {
  it("kartu punya aksi Hasil & Ekspor terpisah dari tombol buka", () => {
    expect(cardSrc).toContain('aria-label="{$t(\'resultsExport\')}: {project.name}"');
    // Aksi kartu tidak menelan klik kartu (stopPropagation).
    expect(cardSrc).toContain("stopPropagation");
    expect(appSrc).toContain("<ProjectCard");
    expect(appSrc).toContain("onResults");
  });

  it("buka = aktifkan dulu bila perlu, gagal = panel tidak dibuka", () => {
    const fn = appSrc.slice(appSrc.indexOf("async function openResults"));
    expect(fn).toContain("if (busy) return");
    expect(fn).toContain("openProject(id)");
    expect(fn).toContain("await refreshProject(id)");
    expect(fn).toContain("showReview = true");
  });
  it("reuse label i18n resultsExport (kontrak kunci di i18n.test.ts)", () => {
    expect(cardSrc).toContain("$t(\"resultsExport\")");
  });
});
