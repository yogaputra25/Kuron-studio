import { describe, expect, it } from "vitest";
import appSrc from "../App.svelte?raw";
import gridSrc from "./ReviewGrid.svelte?raw";

// Mobile bukan target penuh — tapi header desktop dilarang bocor ke layar
// kecil: folder picker iOS tidak ada (dialog plugin throw
// FolderPickerNotImplemented), updater/process capability desktop-only.
// Test ini mengunci gating-nya supaya komponen berikutnya tidak nambah
// tombol desktop tanpa sadar.
describe("mobile header gating (iOS/Android)", () => {
  it("App memakai isMobile terpusat dari lib/device", () => {
    expect(appSrc).toContain('from "./lib/device"');
    expect(appSrc).toContain("isMobile");
  });

  it("tombol desktop-only disembunyikan di mobile", () => {
    // Token situs pemakaian di markup (bukan definisi fungsi / label i18n).
    for (const token of ["<UpdateButton", "onclick={cleanPages}"]) {
      const i = appSrc.lastIndexOf(token);
      expect(i, `${token} hilang dari App`).toBeGreaterThan(-1);
      expect(appSrc.slice(Math.max(0, i - 150), i)).toContain("!isMobile");
    }
  });

  it("Import gabungan: satu tombol + menu, opsi folder gating mobile", () => {
    // detect-ux-pack 3.1: tidak ada lagi onclick terpisah di header.
    expect(appSrc).not.toContain("onclick={importFolder}");
    expect(appSrc).not.toContain("onclick={importFiles}");
    expect(appSrc).toContain("showImportMenu");
    expect(appSrc).toContain('aria-haspopup="menu"');
    // Opsi folder hanya dirender saat bukan mobile.
    const i = appSrc.indexOf("void importFolder()");
    expect(i).toBeGreaterThan(-1);
    expect(appSrc.slice(Math.max(0, i - 400), i)).toContain("!isMobile");
    expect(appSrc).toContain('{$t("importAction")}');
  });

  it("action bar scroll horizontal di layar kecil, wrap di desktop", () => {
    expect(appSrc).toContain("overflow-x-auto");
    expect(appSrc).toContain("sm:flex-wrap");
  });

  it("empty state tanpa sebut drag-drop di mobile", () => {
    expect(appSrc).toContain("emptyHintMobile");
  });

  it("ReviewGrid menolak export folder (png/psd) di mobile", () => {
    expect(gridSrc).toContain('from "./device"');
    expect(gridSrc).toContain("isMobile");
    expect(gridSrc).toContain("belum didukung di iOS");
  });
});
