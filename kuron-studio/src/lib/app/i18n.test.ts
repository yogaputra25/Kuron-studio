// Kontrak key i18n (FE-9 i18n-keys 2.1): dipakai ⊆ terdefinisi, 3 bahasa
// lengkap, mati = warning. Pindai source mentah (tanpa mount): literal
// $t('k'), template $t(`a${..}b`) -> pola, dan literal di cabang ternary.
import { describe, expect, it } from "vitest";
import i18nRaw from "./i18n.ts?raw";
import appRaw from "../../App.svelte?raw";
import type { Lang } from "./i18n";

const modules = import.meta.glob<string>("../**/*.{svelte,ts}", {
  query: "?raw",
  import: "default",
  eager: true,
});

const LANGS: Lang[] = ["en", "id", "zh"];

// ---- Pemindai ----

/** Ambil isi tiap pemanggilan $t(...) dengan kurung berimbang. */
function tCallArgs(src: string): string[] {
  const out: string[] = [];
  const re = /\$t\s*\(/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(src)) !== null) {
    let depth = 1;
    let i = m.index + m[0].length;
    let inStr: string | null = null;
    let tplDepth = 0;
    for (; i < src.length && depth > 0; i++) {
      const c = src[i];
      if (inStr) {
        if (c === "\\") i++;
        else if (c === inStr && tplDepth === 0) inStr = null;
        else if (inStr === "`" && c === "$" && src[i + 1] === "{") tplDepth++;
        else if (inStr === "`" && c === "}" && tplDepth > 0) tplDepth--;
      } else if (c === '"' || c === "'" || c === "`") {
        inStr = c;
      } else if (c === "(") {
        depth++;
      } else if (c === ")") {
        depth--;
      }
    }
    out.push(src.slice(m.index + m[0].length, i - 1));
  }
  return out;
}

/** Literal 'k' di dalam argumen $t(...). */
const LIT = /['"]([A-Za-z0-9_]+)['"]/g;
/** Akhir argumen berupa operator perbandingan -> literal = nilai uji. */
const CMP_BEFORE = /([=!]=)\s*$/;

/** Kumpulkan key terpakai dari satu source. */
function collectUsed(src: string, used: Set<string>, patterns: RegExp[]) {
  for (const arg of tCallArgs(src)) {
    let lit: RegExpExecArray | null;
    LIT.lastIndex = 0;
    let foundInArg = false;
    while ((lit = LIT.exec(arg)) !== null) {
      // Literal pembanding (`key === "x"`) = nilai uji, bukan key.
      if (CMP_BEFORE.test(arg.slice(0, lit.index))) continue;
      used.add(lit[1]);
      foundInArg = true;
    }
    if (foundInArg) continue;
    // Template penuh: $t(`statis${dinamis}statis`) -> pola regex.
    const tm = /^`([\s\S]*)`$/.exec(arg.trim());
    if (tm) {
      const parts = tm[1].split(/\$\{[^}]*\}/).map((p) =>
        p.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"),
      );
      patterns.push(new RegExp(`^${parts.join(".*")}$`));
    }
  }
}

// ---- Definisi: parse blok STR dari source (tanpa import runtime) ----

const strRegion = (() => {
  const start = i18nRaw.indexOf("const STR");
  const end = i18nRaw.indexOf("\n};", start);
  return i18nRaw.slice(start, end);
})();

const defined = new Map<string, Record<string, string>>();
{
  const keyRe = /^  ([A-Za-z0-9_]+): \{/gm;
  const starts: Array<{ name: string; idx: number }> = [];
  let k: RegExpExecArray | null;
  while ((k = keyRe.exec(strRegion)) !== null) starts.push({ name: k[1], idx: k.index });
  for (let j = 0; j < starts.length; j++) {
    const block = strRegion.slice(starts[j].idx, j + 1 < starts.length ? starts[j + 1].idx : undefined);
    const vals: Record<string, string> = {};
    for (const lang of LANGS) {
      const vm = block.match(new RegExp(`${lang}:\\s*"([^"]*)"`, "m"));
      if (vm) vals[lang] = vm[1];
    }
    defined.set(starts[j].name, vals);
  }
}

// ---- Kumpulkan pemakaian ----

const used = new Set<string>();
const patterns: RegExp[] = [];
for (const [path, src] of Object.entries(modules)) {
  if (path.endsWith(".test.ts") || path.endsWith(".test.svelte")) continue;
  if (path.endsWith("/i18n.ts")) continue;
  collectUsed(src, used, patterns);
}
collectUsed(appRaw, used, patterns);
// Template cocok ke definisi -> anggap terpakai (kasus onboardStep*, dinamis).
for (const name of defined.keys()) {
  if (patterns.some((p) => p.test(name))) used.add(name);
}

describe("i18n keys: dipakai ⊆ terdefinisi", () => {
  it("tidak ada key dipakai yang tak terdefinisi", () => {
    const missing = [...used].filter((k) => !defined.has(k)).sort();
    expect(missing, missing.length ? `key hilang: ${missing.join(", ")}` : "ok").toEqual([]);
  });

  it("tiap key terdefinisi lengkap di 3 bahasa + tak ada string kosong", () => {
    const bad: string[] = [];
    for (const [name, vals] of defined) {
      for (const l of LANGS) {
        if (!vals[l]) bad.push(`${name}.${l}`);
      }
    }
    expect(bad, bad.length ? `key tak lengkap: ${bad.join(", ")}` : "ok").toEqual([]);
  });

  it("key mati hanya warning (bukan fail)", () => {
    const dead = [...defined.keys()].filter((k) => !used.has(k));
    if (dead.length > 0) console.warn(`[i18n] key mati: ${dead.sort().join(", ")}`);
  });
});
