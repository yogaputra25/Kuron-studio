import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// Static source lint rules: theme tokens, color contrast, touch spacing, and mobile gating.
// Consolidated from static source-text tests (pola #61).

const SRC = fileURLToPath(new URL("../..", import.meta.url));

function svelteFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((d) => {
    const p = join(dir, d.name);
    if (d.isDirectory()) return svelteFiles(p);
    return d.name.endsWith(".svelte") ? [p] : [];
  });
}

// ---------------------------------------------------------------------------
// 1. Theme-token discipline
// ---------------------------------------------------------------------------

const ALLOWLIST: Record<string, string> = {};

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

const HEX_ALLOWLIST: Record<string, string> = {
  // Ilustrasi brand (bukan UI chrome): badan/mata/sparkle Kuro punya warna
  // identitas sendiri yang tidak berubah ikut tema; aksen pink-nya tetap
  // pakai var(--ks-accent).
  "KuroMascot.svelte": "brand illustration palette",
};

describe("no hex/rgba literals in Svelte (illustration tokens only)", () => {
  const files = svelteFiles(SRC);
  const HEX = /#[0-9a-fA-F]{3,8}\b/g;
  const RGB_FN = /\brgba?\(/g;
  for (const file of files) {
    const name = file.split("/").pop()!;
    if (HEX_ALLOWLIST[name] ?? ALLOWLIST[name]) continue;
    it(`${name} has no hex/rgba literals`, () => {
      const src = readFileSync(file, "utf8");
      const lines = src.split("\n");
      const hits: string[] = [];
      lines.forEach((line, i) => {
        for (const m of line.matchAll(HEX)) hits.push(`${name}:${i + 1}: ${m[0]}`);
        for (const m of line.matchAll(RGB_FN)) hits.push(`${name}:${i + 1}: ${m[0]}`);
      });
      expect(
        hits,
        `${name} pakai literal hex/rgba — ganti dengan token app.css (var(--ks-*))`,
      ).toEqual([]);
    });
  }
  it("no stale hex allowlist entries", () => {
    const componentNames = svelteFiles(SRC).map((f) => f.split("/").pop()!);
    const stale = Object.keys(HEX_ALLOWLIST).filter((n) => !componentNames.includes(n));
    expect(stale, "hex allowlist sudah tidak relevan — hapus").toEqual([]);
  });
});

describe("every allowlist entry is still justified", () => {
  it("no stale allowlist entries (file tidak boleh hilang / sudah rapi)", () => {
    const componentNames = svelteFiles(SRC).map((f) => f.split("/").pop()!);
    const stale = Object.keys(ALLOWLIST).filter((n) => !componentNames.includes(n));
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
      for (const m of src.matchAll(
        /\b(?:bg|text|border|ring|fill|stroke|outline|accent|divide)-(ks-[\w-]+)/g,
      )) {
        if (!defined.has(m[1])) missing.add(`${file.split("/").pop()}: ${m[1]}`);
      }
    }
    expect([...missing], "token tidak ada di app.css — utility jadi tidak ter-generate").toEqual([]);
  });

  it("every soft token has a solid counterpart (badge/alert pairs)", () => {
    const css = readFileSync(join(SRC, "app.css"), "utf8");
    for (const name of ["success", "warn", "danger", "info", "cyan", "accent"]) {
      expect(css, `--ks-${name}-soft tidak ada`).toMatch(new RegExp(`--ks-${name}-soft:`));
      expect(css, `--ks-${name} tidak ada`).toMatch(new RegExp(`--ks-${name}:`));
    }
  });

  it("reduced-motion tetap dijaga di app.css", () => {
    const css = readFileSync(join(SRC, "app.css"), "utf8");
    expect(css).toContain("prefers-reduced-motion");
  });
});

// ---------------------------------------------------------------------------
// 2. Token contrast (WCAG)
// ---------------------------------------------------------------------------

const cssContent = readFileSync(join(SRC, "app.css"), "utf8");

type Var = Record<string, string>;

function parseTheme(head: RegExp): Var {
  const flags = head.flags.replace(/[gy]/g, "");
  const m = new RegExp(`${head.source}\\s*\\{([^}]*)\\}`, flags).exec(cssContent);
  if (!m) throw new Error(`block tema tidak ditemukan: ${head.source}`);
  const out: Var = {};
  for (const d of m[1].matchAll(/(--ks-[\w-]+):\s*(#[0-9a-fA-F]{3,8})/g)) {
    out[d[1]] = d[2];
  }
  return out;
}

function channel(c: number): number {
  const v = c / 255;
  return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string): number {
  const raw = hex.replace("#", "");
  const n =
    raw.length === 3
      ? raw.split("").map((ch) => ch + ch).join("")
      : raw.slice(0, 6);
  const r = parseInt(n.slice(0, 2), 16);
  const g = parseInt(n.slice(2, 4), 16);
  const b = parseInt(n.slice(4, 6), 16);
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const THEMES: [string, RegExp][] = [
  ["dark", /^:root,\s*:root\[data-theme="dark"\]/m],
  ["light", /^:root\[data-theme="light"\]/m],
];

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
  ["--ks-danger", "--ks-danger-soft", 4.5],
  ["--ks-success", "--ks-success-soft", 4.5],
  ["--ks-warn", "--ks-warn-soft", 4.5],
  ["--ks-info", "--ks-info-soft", 4.5],
  ["--ks-cyan", "--ks-cyan-soft", 4.5],
  ["--ks-accent-fg", "--ks-danger", 4.5],
  ["--ks-illust-on-solid", "--ks-illust-solid", 4.5],
  ["--ks-illust-on-solid-muted", "--ks-illust-solid", 3.0],
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

// ---------------------------------------------------------------------------
// 3. Spacing and controls
// ---------------------------------------------------------------------------

describe("form controls are not cramped", () => {
  it("no 10px/11px text on <select>, <input>, <textarea> controls", () => {
    const bad: string[] = [];
    for (const file of svelteFiles(SRC)) {
      const name = file.split("/").pop()!;
      const src = readFileSync(file, "utf8");
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
        if (!/\bclass=/.test(tag)) continue;
        if (/\bpx-(?:0|0\.5|1|1\.5)\b/.test(tag) && !/px-2\b/.test(tag)) {
          bad.push(`${name}: ${tag.slice(0, 90)}`);
        }
      }
    }
    expect(bad, "select dengan padding < 8px kelihatan kaku").toEqual([]);
  });

  it("src/lib bebas tag <select> bawaan", () => {
    const hits = svelteFiles(join(SRC, "lib")).filter((f) =>
      /<select\b/.test(readFileSync(f, "utf8")),
    );
    expect(hits, `masih pakai <select>: ${hits.join(", ")}`).toEqual([]);
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
    expect(
      tight,
      `spacing rapat (${tight}) mengalahkan spacing lega (${loose}) — form akan terasa kaku`,
    ).toBeLessThan(loose);
  });
});

describe("sidebar forms have breathing room", () => {
  it("EditorPanel sidebar default lebih lebar dari 256px (resizable full-range)", () => {
    const panel = readFileSync(join(SRC, "lib/editor/panel.ts"), "utf8");
    const def = Number(/SIDEBAR_DEFAULT = (\d+)/.exec(panel)?.[1]);
    const min = Number(/SIDEBAR_MIN = (\d+)/.exec(panel)?.[1]);
    const max = Number(/SIDEBAR_MAX = (\d+)/.exec(panel)?.[1]);
    expect(def, "SIDEBAR_DEFAULT hilang dari panel.ts").toBeGreaterThan(256);
    expect(min, "clamp bawah full-drag = 192").toBeGreaterThanOrEqual(192);
    expect(max, "clamp atas harus di atas default").toBeGreaterThan(def);
    const src = readFileSync(join(SRC, "lib/editor/EditorPanel.svelte"), "utf8");
    expect(src, "aside tidak lagi memakai lebar tampil dari state").toContain("displayW");
  });

  it("EditorPanel sidebar uses space-y between sections", () => {
    const src = readFileSync(join(SRC, "lib/editor/EditorPanel.svelte"), "utf8");
    const aside = src.slice(src.indexOf("<aside"), src.indexOf("</aside>"));
    expect(aside).toContain("space-y-");
  });
});

// ---------------------------------------------------------------------------
// 4. Mobile gating (iOS/Android)
// ---------------------------------------------------------------------------

describe("mobile header gating (iOS/Android)", () => {
  const appSrc = readFileSync(join(SRC, "App.svelte"), "utf8");
  const headSrc = readFileSync(join(SRC, "lib/app/shell/AppHeader.svelte"), "utf8");
  // Gating export folder (png/psd) pindah ReviewGrid → ExportActions
  // saat split #50 (main d263261); baca komponen yang memegang tombol.
  const gridSrc = readFileSync(join(SRC, "lib/export/ExportActions.svelte"), "utf8");

  it("App memakai isMobile terpusat dari lib/device", () => {
    expect(headSrc).toContain('from "../../app/device"');
    expect(headSrc).toContain("isMobile");
  });

  it("tombol desktop-only disembunyikan di mobile", () => {
    for (const token of ["<UpdateButton", "onclick={onClean}"]) {
      const i = headSrc.lastIndexOf(token);
      expect(i, `${token} hilang dari AppHeader`).toBeGreaterThan(-1);
      expect(headSrc.slice(Math.max(0, i - 150), i)).toContain("!isMobile");
    }
  });

  it("Import gabungan: satu tombol + menu, opsi folder gating mobile", () => {
    expect(headSrc).not.toContain("onclick={importFolder}");
    expect(headSrc).not.toContain("onclick={importFiles}");
    expect(headSrc).toContain("showImportMenu");
    expect(headSrc).toContain('aria-haspopup="menu"');
    const i = headSrc.indexOf("onImportFolder()");
    expect(i).toBeGreaterThan(-1);
    expect(headSrc.slice(Math.max(0, i - 400), i)).toContain("!isMobile");
    expect(headSrc).toContain('{$t("importAction")}');
  });

  it("action bar scroll horizontal di layar kecil, wrap di desktop", () => {
    expect(headSrc).toContain("overflow-x-auto");
    expect(headSrc).toContain("sm:flex-none");
  });

  it("empty state tanpa sebut drag-drop di mobile", () => {
    expect(appSrc).toContain("emptyHintMobile");
  });

  it("ExportActions menolak export folder (png/psd) di mobile", () => {
    expect(gridSrc).toContain('from "../app/device"');
    expect(gridSrc).toContain("isMobile");
    expect(gridSrc).toContain("belum didukung di iOS");
  });
});
