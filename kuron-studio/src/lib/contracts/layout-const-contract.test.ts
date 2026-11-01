import { describe, expect, it } from "vitest";
import {
  BODY_MIN_FRAC,
  LAYOUT_FILL,
  LAYOUT_FONT_STEP,
  LAYOUT_LINE_HEIGHT,
  LAYOUT_MAX_FONT,
  LAYOUT_MIN_FONT,
  LAYOUT_PADDING,
} from "../editor/text/textLayout";
import { OVERLAY_FONT_STACK, OVERLAY_FONT_STYLE } from "../editor/text/textMeasure";
import layout from "./layout.json";

// Kontrak lintas bahasa, sumber = `layout.json` — TANPA membaca source Rust
// (tidak ada `?raw` ke `.rs`, tidak ada path `src-tauri/`):
// - TS (`textLayout.ts`/`textMeasure.ts`) mengimpor JSON langsung,
// - Rust (`commands/export.rs`) membaca JSON yang sama via `include_str!` +
//   `OnceLock` (dipin test Rust `layout_matches_json_contract`),
// - test ini memastikan pemakaian TS tetap sinkron dengan JSON.
//
// Rumus layout hidup di dua bahasa: kanvas memutuskan tampilan, Rust
// menyesuaikan di jalur fallback.

describe("layout constants contract (TS canvas == JSON)", () => {
  it("angkanya sama dengan JSON", () => {
    expect(LAYOUT_MIN_FONT).toBe(layout.minFont);
    expect(LAYOUT_MAX_FONT).toBe(layout.maxFont);
    expect(LAYOUT_FONT_STEP).toBe(layout.fontStep);
    expect(LAYOUT_LINE_HEIGHT).toBe(layout.lineHeight);
    expect(LAYOUT_PADDING).toBe(layout.padding);
    expect(LAYOUT_FILL).toBe(layout.fillFactor);
    expect(BODY_MIN_FRAC).toBe(layout.bodyMinFrac);
  });

  it("daftar kandidat font lengkap per OS", () => {
    for (const [os, list] of [
      ["windows", layout.windowsFonts],
      ["macos", layout.macosFonts],
      ["linux", layout.linuxFonts],
    ] as const) {
      expect(list.length, `kandidat ${os} kosong`).toBeGreaterThan(0);
      for (const p of list) {
        expect(typeof p, `entri ${os} bukan path`).toBe("string");
      }
    }
  });
});

/**
 * Font overlay harus sama dengan yang dimuat Rust. Pendekatan "pakai font
 * sistem" (grup 4): `ab_glyph` hanya bisa TTF/OTF, dan repo ini tidak
 * membundel Inter dalam bentuk itu.
 */
describe("font overlay == font export (grup 4)", () => {
  it("stack CSS mencerminkan tiga kandidat pertama Windows di JSON", () => {
    const cssFamily = layout.cssFamily as Record<string, string>;
    const files = layout.windowsFonts.slice(0, 3).map((p) => p.split("/").pop() ?? p);
    expect(files.length, "kandidat Windows kurang dari 3").toBeGreaterThanOrEqual(3);

    const tiga = files.slice(0, 3);
    const families = tiga.map((f) => cssFamily[f]);
    expect(families.every(Boolean), `file tanpa padanan CSS: ${tiga.join(", ")}`).toBe(true);

    // Terdaftar, DAN urutannya sama — urutan menentukan font mana yang dipakai.
    const idx = families.map((f) => OVERLAY_FONT_STACK.indexOf(f));
    expect(idx.every((i) => i >= 0), `tak ada di stack: ${families.join(", ")}`).toBe(true);
    expect([...idx].sort((a, b) => a - b)).toEqual(idx);
  });

  it("berat huruf normal (TTF sistem = regular, bukan 600)", () => {
    expect(OVERLAY_FONT_STYLE).toBe("normal");
  });
});
