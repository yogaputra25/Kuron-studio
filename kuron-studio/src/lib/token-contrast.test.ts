import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// Kontras WCAG token, dicek otomatis.
//
// app.css mengklaim semua pasangan teks/latar itu AA. Kalau tidak diuji,
// klaim itu cuma komentar yang bisa basi diam-diam — dan pengembang berikutnya
// akan menambahkan `text-ink-3` di atas `bg-raised` tanpa sadar. Test ini
// membacanya dari CSS, jadi sumber kebenaran tetap satu file.
//
// CATATAN: Vite tidak menerima `?raw` untuk `.css` (hasilnya string kosong),
// jadi file ini baca lewat node:fs. Batasnya: `node:*` hanya boleh di
// `*.test.ts` — bukan di kode app.

const css = readFileSync(fileURLToPath(new URL("../app.css", import.meta.url)), "utf8");

type Var = Record<string, string>;

function parseTheme(head: RegExp): Var {
  // `head` cuma mencocokkan pembuka blok (`:root[data-theme=…] {`); badan
  // blok kita tempelkan di sini. Kedua block tidak punya nested brace, jadi
  // `[^}]*` cukup dan tidak perlu hitung depth.
  const flags = head.flags.replace(/[gy]/g, "");
  const m = new RegExp(`${head.source}\\s*\\{([^}]*)\\}`, flags).exec(css);
  if (!m) throw new Error(`block tema tidak ditemukan: ${head.source}`);
  const out: Var = {};
  for (const d of m[1].matchAll(/(--ks-[\w-]+):\s*(#[0-9a-fA-F]{3,8})/g)) {
    out[d[1]] = d[2];
  }
  return out;
}

function channel(c: number): number {
  const s = c / 255;
  return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string): number {
  let h = hex.replace("#", "");
  if (h.length === 3) h = [...h].map((c) => c + c).join("");
  const r = parseInt(h.slice(0, 2), 16);
  const g = parseInt(h.slice(2, 4), 16);
  const b = parseInt(h.slice(4, 6), 16);
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

// Hanya selector pembuka — `{` dan badan blok ditempel `parseTheme`.
const THEMES: [string, RegExp][] = [
  ["dark", /^:root,\s*:root\[data-theme="dark"\]/m],
  ["light", /^:root\[data-theme="light"\]/m],
];

// [foreground, background, minimum]. 4.5 = AA teks normal, 3.0 = AA teks besar
// + komponen UI (border tombol, ikon, badge).
const PAIRS: [string, string, number][] = [
  ["--ks-text", "--ks-bg", 4.5],
  ["--ks-text", "--ks-surface", 4.5],
  ["--ks-text", "--ks-surface-2", 4.5],
  ["--ks-text", "--ks-raised", 4.5],
  ["--ks-text-2", "--ks-bg", 4.5],
  ["--ks-text-2", "--ks-surface", 4.5],
  ["--ks-text-2", "--ks-surface-2", 4.5],
  ["--ks-text-2", "--ks-raised", 4.5],
  ["--ks-text-3", "--ks-bg", 3.0],
  ["--ks-text-3", "--ks-surface", 3.0],
  ["--ks-accent", "--ks-bg", 3.0],
  ["--ks-accent", "--ks-surface", 3.0],
  ["--ks-accent", "--ks-surface-2", 3.0],
  ["--ks-accent-fg", "--ks-accent", 4.5],
  ["--ks-cyan", "--ks-bg", 3.0],
  ["--ks-cyan", "--ks-surface", 3.0],
  ["--ks-success", "--ks-bg", 3.0],
  ["--ks-success", "--ks-surface", 3.0],
  ["--ks-warn", "--ks-bg", 3.0],
  ["--ks-warn", "--ks-surface", 3.0],
  ["--ks-danger", "--ks-bg", 3.0],
  ["--ks-danger", "--ks-surface", 3.0],
  ["--ks-danger", "--ks-surface-2", 3.0],
  ["--ks-info", "--ks-bg", 3.0],
  ["--ks-info", "--ks-surface", 3.0],
  ["--ks-line-strong", "--ks-surface", 1.2],

  // State "soft": badge/alert memakai warna solids dengan opacity rendah.
  // Teksnya harus tetap terbaca di atas warna solids itu, bukan atas surface.
  ["--ks-danger", "--ks-danger-soft", 4.5],
  ["--ks-success", "--ks-success-soft", 4.5],
  ["--ks-warn", "--ks-warn-soft", 4.5],
  ["--ks-info", "--ks-info-soft", 4.5],
  ["--ks-cyan", "--ks-cyan-soft", 4.5],

  // State hover tombol: `Button` variant danger jadi bg solid + accent-fg.
  ["--ks-accent-fg", "--ks-danger", 4.5],
];

describe("token contrast (WCAG)", () => {
  for (const [name, head] of THEMES) {
    describe(`theme ${name}`, () => {
      const vars = parseTheme(head);

      it("defines every token the pairs need", () => {
        const missing = new Set<string>();
        for (const [fg, bg] of PAIRS) {
          if (!vars[fg]) missing.add(fg);
          if (!vars[bg]) missing.add(bg);
        }
        expect([...missing]).toEqual([]);
      });

      for (const [fg, bg, min] of PAIRS) {
        it(`${fg} on ${bg} >= ${min}:1`, () => {
          const ratio = contrast(vars[fg], vars[bg]);
          expect(
            ratio,
            `${fg} (${vars[fg]}) di ${bg} (${vars[bg]}) = ${ratio.toFixed(2)}:1, butuh ${min}:1`,
          ).toBeGreaterThanOrEqual(min);
        });
      }
    });
  }

  it("both themes define the same token set", () => {
    const [dark, light] = THEMES.map(([, sel]) => parseTheme(sel));
    expect(Object.keys(dark).sort()).toEqual(Object.keys(light).sort());
  });
});
