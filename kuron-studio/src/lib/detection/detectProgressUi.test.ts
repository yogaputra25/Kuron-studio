import { describe, expect, it } from "vitest";
import compSrc from "./DetectProgress.svelte?raw";

// detect-ux-pack 1.2: overlay non-blocking + a11y + tanpa string hardcode.
// Pola-sumber (gaya preview-transport.test.ts): komponen Svelte + runtime,
// mount unit tak bermakna.
describe("DetectProgress overlay (detect-ux-pack 1.2)", () => {
  it("punya role status + progressbar dengan aria-valuenow", () => {
    expect(compSrc).toContain('role="status"');
    expect(compSrc).toContain('role="progressbar"');
    expect(compSrc).toContain("aria-valuenow");
    expect(compSrc).toContain("aria-valuemin");
    expect(compSrc).toContain("aria-valuemax");
  });

  it("non-blocking: klik luar overlay lolos (tanpa backdrop menelan klik)", () => {
    expect(compSrc).toContain("pointer-events-none");
    expect(compSrc).toContain("pointer-events-auto");
  });

  it("semua label lewat i18n, tanpa hardcode Indonesia", () => {
    expect(compSrc).toContain('$t("detecting")');
    expect(compSrc).toContain('$t("detectSummary")');
    expect(compSrc).toContain('$t("close")');
    expect(compSrc).not.toContain("Mendeteksi");
    expect(compSrc).not.toContain("Coba lagi");
  });

  it("tutup otomatis hanya bila tanpa gagal; gagal tampilkan ringkasan", () => {
    expect(compSrc).toContain("setTimeout(onClose, 2000)");
    expect(compSrc).toContain("failedCount === 0");
    expect(compSrc).toContain("state.failed");
  });
});
