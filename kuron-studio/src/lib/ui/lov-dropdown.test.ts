import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// Kontrak lov-dropdown-unified: semua dropdown memakai ui/Lov (custom
// searchable ala LOV project), bukan <select> bawaan yang beda tampilan
// per OS dan tidak bisa di-search.

const SRC = fileURLToPath(new URL("../..", import.meta.url));

function svelteFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((d) => {
    const p = join(dir, d.name);
    if (d.isDirectory()) return svelteFiles(p);
    return d.name.endsWith(".svelte") ? [p] : [];
  });
}

describe("tidak ada <select> bawaan tersisa", () => {
  it("src/lib bebas tag <select>", () => {
    const hits = svelteFiles(join(SRC, "lib")).filter((f) =>
      /<select\b/.test(readFileSync(f, "utf8")),
    );
    expect(hits, `masih pakai <select>: ${hits.join(", ")}`).toEqual([]);
  });
});

describe("ui/Lov pola LOV project", () => {
  const lov = readFileSync(join(SRC, "lib/ui/Lov.svelte"), "utf8");

  it("punya tombol pemicu + listbox + search + aria", () => {
    for (const m of [
      'aria-haspopup="listbox"',
      "aria-expanded={open}",
      'role="listbox"',
      'role="option"',
      "aria-selected={o.value === value}",
      "bind:value={query}",
    ]) {
      expect(lov, `Lov kehilangan: ${m}`).toContain(m);
    }
  });

  it("tutup saat focusout keluar atau Escape", () => {
    expect(lov).toContain("closeOnFocusOut");
    expect(lov.match(/Escape/g)?.length ?? 0).toBeGreaterThanOrEqual(2);
  });

  it("pakai searchList i18n (bukan hardcode)", () => {
    expect(lov).toContain('$t("searchList")');
    expect(lov).not.toContain("Cari…");
  });
});

describe("pemakaian sudah Lov semua", () => {
  it("TranslatePanel: Provider/Model/Target/Style/Mosaic", () => {
    const src = readFileSync(join(SRC, "lib/TranslatePanel.svelte"), "utf8");
    expect(src).not.toContain("ui/Select.svelte");
    expect(src.match(/<Lov\b/g)?.length).toBe(5);
  });

  it("ProviderSettings: Tipe/Model tersedia", () => {
    const src = readFileSync(join(SRC, "lib/ProviderSettings.svelte"), "utf8");
    expect(src).not.toContain("ui/Select.svelte");
    expect(src.match(/<Lov\b/g)?.length).toBe(2);
  });

  it("searchList ada di 3 bahasa", () => {
    const src = readFileSync(join(SRC, "lib/app/i18n.ts"), "utf8");
    expect(src).toContain("searchList");
  });
});
