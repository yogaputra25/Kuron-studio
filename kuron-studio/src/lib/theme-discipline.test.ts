import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// Setelah restyling, Palet Tailwind mentah (bg-zinc-800, text-emerald-300)
// mulai crept back — tiap kemunculan baru melewati light theme, karena palet
// itu tidak punya pasangan kontras di sana. Test ini menjaga semua styling
// lewat token semantik dari app.css.
//
// Pengecualian yang disengaja ada di ALLOWLIST dengan alasannya.

const SRC = fileURLToPath(new URL("..", import.meta.url));

const ALLOWLIST: Record<string, string> = {
  "CanvasEditor.svelte":
    "Latar kanvas sengaja hitam (bg-black) di kedua tema: gambar manga di atas netral gelap jauh lebih terbaca, dan area luar gambar tidak salah warna di light mode. Warna overlay bubble sudah lewat token CSS var (`paint()`), bukan hex.",
};

function svelteFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((d) => {
    const p = join(dir, d.name);
    if (d.isDirectory()) return svelteFiles(p);
    return d.name.endsWith(".svelte") ? [p] : [];
  });
}

// Palet default Tailwind v4. Token app.css sengaja memakai nama di luar
// daftar ini (ink, surface, line, accent, warn, danger, …).
const RAW_PALETTE =
  /\b(?:bg|text|border|ring|fill|stroke|outline|decoration|divide|shadow|accent|caret|placeholder)-(?:zinc|sky|emerald|rose|amber|violet|slate|red|blue|neutral|stone|gray|green|indigo|purple|pink|yellow|orange|fuchsia|teal|lime)-[0-9]{2,3}\b/g;

describe("no raw Tailwind palette in Svelte (theme-token discipline)", () => {
  const files = svelteFiles(SRC);

  it("found the component files", () => {
    expect(files.length).toBeGreaterThan(5);
  });

  for (const file of files) {
    const name = file.split("/").pop()!;
    if (ALLOWLIST[name]) continue;
    it(`${name} uses tokens only`, () => {
      const src = readFileSync(file, "utf8");
      const hits = [...src.matchAll(RAW_PALETTE)].map((m) => m[0]);
      expect(
        [...new Set(hits)],
        ALLOWLIST[name] ?? `pakai palet mentah — ganti dengan token app.css`,
      ).toEqual([]);
    });
  }
});

describe("every allowlist entry is still justified", () => {
  it("no stale allowlist entries (file tidak boleh hilang / sudah rapi)", () => {
    const files = svelteFiles(SRC).map((f) => f.split("/").pop()!);
    const stale = Object.keys(ALLOWLIST).filter((n) => !files.includes(n));
    expect(stale, "allowlist sudah tidak relevan — hapus").toEqual([]);
  });
});

describe("token names used in components actually exist", () => {
  it("no component references an undefined token", () => {
    const css = readFileSync(join(SRC, "app.css"), "utf8");
    const defined = new Set([...css.matchAll(/(--ks-[\w-]+):/g)].map((m) => m[1]));
    const missing = new Set<string>();
    for (const file of svelteFiles(SRC)) {
      const src = readFileSync(file, "utf8");
      // Hanya nama yang benar-benar expose ke utility Tailwind, yaitu yang
      // terdaftar di blok `@theme` — itu yang dipetakan ke `var(--ks-*)`.
      for (const m of src.matchAll(
        /\b(?:bg|text|border|ring|fill|stroke|outline|accent|divide)-(ks-[\w-]+)/g,
      )) {
        if (!defined.has(m[1])) missing.add(`${file.split("/").pop()}: ${m[1]}`);
      }
    }
    expect([...missing], "token tidak ada di app.css — utility jadi tidak genere").toEqual([]);
  });

  it("every soft token has a solid counterpart (badge/alert pairs)", () => {
    const css = readFileSync(join(SRC, "app.css"), "utf8");
    for (const name of ["success", "warn", "danger", "info", "cyan", "accent"]) {
      expect(css, `--ks-${name}-soft tidak ada`).toMatch(new RegExp(`--ks-${name}-soft:`));
      expect(css, `--ks-${name} tidak ada`).toMatch(new RegExp(`--ks-${name}:`));
    }
  });
});
