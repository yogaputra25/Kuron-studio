import { describe, expect, it } from "vitest";
import appSrc from "../../App.svelte?raw";
import snackSrc from "./Snackbar.svelte?raw";
import bannerSrc from "../app/shell/AlertBanners.svelte?raw";

// Hasil sukses (mis. "41 halaman dipindah ke X") tampil sebagai snackbar
// melayang yang auto-hilang — bukan strip yang menggeser layout.
// Strip error dipertahankan: error butuh persisten + tombol Detail.
describe("success snackbar", () => {
  it("info dirender sebagai Snackbar keyed, bukan strip", () => {
    expect(bannerSrc).toContain("<Snackbar message={info}");
    expect(bannerSrc).toContain("{#key info}");
    expect(bannerSrc).not.toContain("border-success-soft");
    expect(appSrc).toContain("<AlertBanners");
  });

  it("error tetap strip persisten", () => {
    expect(bannerSrc).toContain("border-warn-soft");
    expect(bannerSrc).toContain('role="alert"');
  });

  it("snackbar: pill melayang, animasi fly, auto-tutup, tanpa spinner", () => {
    expect(snackSrc).toContain('role="status"');
    expect(snackSrc).toContain("transition:fly");
    expect(snackSrc).toContain("setTimeout(onClose");
    expect(snackSrc).toContain("z-[100]");
    expect(snackSrc).not.toContain("animate-spin");
  });
});
