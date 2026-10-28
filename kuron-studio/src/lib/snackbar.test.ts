import { describe, expect, it } from "vitest";
import appSrc from "../App.svelte?raw";
import snackSrc from "./ui/Snackbar.svelte?raw";

// Hasil sukses (mis. "41 halaman dipindah ke X") tampil sebagai snackbar
// melayang yang auto-hilang — bukan strip yang menggeser layout.
// Strip error dipertahankan: error butuh persisten + tombol Detail.
describe("success snackbar", () => {
  it("info dirender sebagai Snackbar keyed, bukan strip", () => {
    expect(appSrc).toContain("<Snackbar message={info}");
    expect(appSrc).toContain("{#key info}");
    expect(appSrc).not.toContain("border-success-soft");
  });

  it("error tetap strip persisten", () => {
    expect(appSrc).toContain("border-warn-soft");
    expect(appSrc).toContain('role="alert"');
  });

  it("snackbar: pill melayang, animasi fly, auto-tutup, tanpa spinner", () => {
    expect(snackSrc).toContain('role="status"');
    expect(snackSrc).toContain("transition:fly");
    expect(snackSrc).toContain("setTimeout(onClose");
    expect(snackSrc).toContain("z-[100]");
    expect(snackSrc).not.toContain("animate-spin");
  });
});
