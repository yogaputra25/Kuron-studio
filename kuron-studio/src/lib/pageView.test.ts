import { describe, expect, it } from "vitest";
import gridSrc from "./PageGrid.svelte?raw";
import listSrc from "./PageList.svelte?raw";
import appSrc from "../App.svelte?raw";

// page-list-view: grid diekstrak + list baru, props identik, tanpa string
// inline. Pola-sumber (gaya preview-transport.test.ts): komponen Svelte,
// mount unit tak bermakna.
describe("page grid/list split (page-list-view)", () => {
  it("Grid dan List memakai props identik + observer + seleksi bersama", () => {
    for (const src of [gridSrc, listSrc]) {
      expect(src).toContain("thumbs: Record<string, string>");
      expect(src).toContain("broken: Record<string, string>");
      expect(src).toContain("selectedIds: string[]");
      expect(src).toContain("onCardClick");
      expect(src).toContain("onToggleSelect");
      expect(src).toContain("use:observeCard");
    }
  });

  it("list tidak punya string UI hardcode (grid mewarisi teks App apa adanya)", () => {
    expect(listSrc).not.toMatch(/Preview gagal|Coba lagi|Memuat \d/);
    expect(listSrc).toContain("$t(");
  });

  it("list: header sortir + keyboard + baris aktif coral + skeleton", () => {
    expect(listSrc).toContain("setSort");
    expect(listSrc).toContain('role="listbox"');
    expect(listSrc).toContain("onkeydown");
    expect(listSrc).toContain("border-l-accent");
    expect(listSrc).toContain("animate-pulse");
    expect(listSrc).toContain("aria-selected");
  });

  it("grid: prefix folder + outline pilihan + visual lama utuh", () => {
    expect(gridSrc).toContain("parentFolder");
    expect(gridSrc).toContain("border-accent");
    expect(gridSrc).toContain("aspect-[3/4]");
    expect(gridSrc).toContain("StatusBadge");
  });

  it("App memilih salah satu view (tidak dua markup bercabang)", () => {
    expect(appSrc).toContain("PageGrid");
    expect(appSrc).toContain("PageList");
    expect(appSrc).not.toContain("use:observeCard={pg.id}");
  });
});
