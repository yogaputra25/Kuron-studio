import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// Padding/margin itu aturan, bukan estetika — ada angka yang terlalu kecil untuk
// touch target WCAG (44px) atau terlalu rapat untuk dibaca. Test ini
// mengunci batas bawahnya supaya komponen berikutnya tidak mengulang masalah
// yang sama.

const SRC = fileURLToPath(new URL("..", import.meta.url));

function svelteFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((d) => {
    const p = join(dir, d.name);
    if (d.isDirectory()) return svelteFiles(p);
    return d.name.endsWith(".svelte") ? [p] : [];
  });
}

describe("form controls are not cramped", () => {
  it("no 10px/11px text on <select>, <input>, <textarea> controls", () => {
    const bad: string[] = [];
    for (const file of svelteFiles(SRC)) {
      const name = file.split("/").pop()!;
      const src = readFileSync(file, "utf8");
      // Control + class-nya dalam satu tag. Label 11px itu Boleh (hanya
      // teks kecil); control yang 11px = teks isinya yang kebecil.
      for (const m of src.matchAll(
        /<(select|input|textarea)\b[^>]*\btext-\[(?:10|11)px\][^>]*>/g,
      )) {
        bad.push(`${name}: ${m[0].slice(0, 90)}`);
      }
    }
    expect(
      bad,
      "Control form 10/11px = teks isinya kebecil. Minimal 13px, atau 12px\n" +
        "kalau memang padat. Label boleh 11px.",
    ).toEqual([]);
  });

  it("every select has px-2 or more horizontal padding", () => {
    const bad: string[] = [];
    for (const file of svelteFiles(SRC)) {
      const name = file.split("/").pop()!;
      const src = readFileSync(file, "utf8");
      for (const m of src.matchAll(/<select\b[^>]*>/g)) {
        const tag = m[0];
        // Class select boleh datang dari konstanta (const SELECT = "…"),
        // jadi cukup cek tag yang inline punya class padding sendiri.
        if (!/\bclass=/.test(tag)) continue;
        if (/\bpx-(?:0|0\.5|1|1\.5)\b/.test(tag) && !/px-2\b/.test(tag)) {
          bad.push(`${name}: ${tag.slice(0, 90)}`);
        }
      }
    }
    expect(bad, "select dengan padding < 8px kelihatan kaku").toEqual([]);
  });
});

describe("primary interactive controls meet touch target", () => {
  it("Button has explicit heights, not padding-derived ones", () => {
    const btn = readFileSync(join(SRC, "lib/ui/Button.svelte"), "utf8");
    const sizes = btn.slice(btn.indexOf("const SIZE"));
    for (const [name, minH] of [["sm", "h-7"], ["md", "h-8"], ["lg", "h-10"]] as const) {
      expect(sizes, `size ${name} harus punya tinggi eksplisit`).toContain(minH);
    }
  });
});

describe("spacing scale stays breathable", () => {
  it("tight values (gap-1 / py-1 / space-y-1) stay a minority of spacing utils", () => {
    const all: string[] = [];
    for (const file of svelteFiles(SRC)) all.push(readFileSync(file, "utf8"));
    const src = all.join("\n");
    const classes = [...src.matchAll(/class="([^"]*)"/g)]
      .map((m) => m[1])
      .join(" ");
    const tight = (classes.match(/\b(?:gap-1|py-1|space-y-1|mt-1|mb-1)\b/g) ?? []).length;
    const loose = (classes.match(/\b(?:gap-[2-9]|p[xy]?-[4-9]|space-y-[2-9])\b/g) ?? []).length;
    // Padding rapat hanya boleh untuk detail sekunder (badge, kbd, chip),
    // bukan struktur utama. Kalau dibalik artinya UI sedang kaku lagi.
    expect(
      tight,
      `spacing rapat (${tight}) mengalahkan spacing lega (${loose}) — form akan terasa kaku`,
    ).toBeLessThan(loose);
  });
});

describe("sidebar forms have breathing room", () => {
  it("EditorPanel sidebar is wider than w-64 (256px)", () => {
    const src = readFileSync(join(SRC, "lib/EditorPanel.svelte"), "utf8");
    const aside = src.slice(src.indexOf("<aside"));
    expect(aside.slice(0, 200)).toMatch(/w-\[(\d+)rem\]/);
    const rem = Number(/w-\[(\d+)rem\]/.exec(aside)![1]);
    // 256px = 16rem. Form translate butuh lebih dari itu.
    expect(rem, `sidebar ${rem}rem masih terlalu sempit`).toBeGreaterThan(16);
  });

  it("EditorPanel sidebar uses space-y between sections", () => {
    const src = readFileSync(join(SRC, "lib/EditorPanel.svelte"), "utf8");
    const aside = src.slice(src.indexOf("<aside"));
    // Spacing antar section harus dari wrapper, bukan margin per-section.
    expect(aside.slice(0, 300)).toContain("space-y-");
  });
});
