import { describe, expect, it } from "vitest";
import exportRs from "../../src-tauri/src/commands/export.rs?raw";
import {
  BODY_MIN_FRAC,
  LAYOUT_FONT_STEP,
  LAYOUT_LINE_HEIGHT,
  LAYOUT_MAX_FONT,
  LAYOUT_MIN_FONT,
  LAYOUT_PADDING,
} from "./textLayout";
import { OVERLAY_FONT_STACK, OVERLAY_FONT_STYLE } from "./textMeasure";

// Rumus layout hidup di dua bahasa: `textLayout.ts` (kanvas memutuskan) dan
// `export.rs` (jalur fallback saat `layout` belum tersimpan). Begitu angkanya
// menyimpang, export dan kanvas kembali berbeda — persis bug yang change
// `export-text-parity` tutup. Dibaca via `?raw` supaya tak perlu parser.
//
// Sumber kebenaran arahnya TS → Rust: kanvas yang memutuskan tampilan, Rust
// menyesuaikan.

const rustF32 = (name: string): number | null => {
  const m = new RegExp(`pub const ${name}: f32 = ([0-9.]+);`).exec(exportRs);
  return m ? Number(m[1]) : null;
};

const rustI32 = (name: string): number | null => {
  const m = new RegExp(`pub const ${name}: i32 = ([0-9]+);`).exec(exportRs);
  return m ? Number(m[1]) : null;
};

describe("layout constants contract (TS canvas == Rust fallback)", () => {
  it("konstanta Rust bisa dibaca", () => {
    expect(rustF32("LAYOUT_MIN_FONT"), "LAYOUT_MIN_FONT tak ada di export.rs").not.toBeNull();
    expect(rustF32("LAYOUT_MAX_FONT")).not.toBeNull();
    expect(rustF32("LAYOUT_FONT_STEP")).not.toBeNull();
    expect(rustF32("LAYOUT_LINE_HEIGHT")).not.toBeNull();
    expect(rustI32("LAYOUT_PADDING")).not.toBeNull();
    expect(rustF32("BODY_MIN_FRAC"), "BODY_MIN_FRAC tak ada di export.rs").not.toBeNull();
  });

  it("angkanya sama persis di kedua sisi", () => {
    expect(rustF32("LAYOUT_MIN_FONT")).toBe(LAYOUT_MIN_FONT);
    expect(rustF32("LAYOUT_MAX_FONT")).toBe(LAYOUT_MAX_FONT);
    expect(rustF32("LAYOUT_FONT_STEP")).toBe(LAYOUT_FONT_STEP);
    expect(rustF32("LAYOUT_LINE_HEIGHT")).toBe(LAYOUT_LINE_HEIGHT);
    expect(rustI32("LAYOUT_PADDING")).toBe(LAYOUT_PADDING);
    expect(rustF32("BODY_MIN_FRAC")).toBe(BODY_MIN_FRAC);
  });
});

/**
 * Font overlay harus sama dengan yang dimuat Rust. Pendekatan "pakai font
 * sistem" (grup 4): `ab_glyph` hanya bisa TTF/OTF, dan repo ini tidak
 * membundel Inter dalam bentuk itu.
 */
const CSS_FAMILY: Record<string, string> = {
  "arial.ttf": "Arial",
  "tahoma.ttf": "Tahoma",
  "msyh.ttc": "Microsoft YaHei",
  "msyhl.ttc": "Microsoft YaHei Light",
  "msgothic.ttc": "MS Gothic",
  "segoeui.ttf": "Segoe UI",
  "calibri.ttf": "Calibri",
};

describe("font overlay == font export (grup 4)", () => {
  it("stack CSS mencerminkan tiga kandidat pertama Rust (Windows)", () => {
    const block = /target_os = "windows"[\s\S]*?\]\);/.exec(exportRs)?.[0] ?? "";
    const files = [...block.matchAll(/"C:\/Windows\/Fonts\/([^"]+)"/g)].map((m) => m[1]);
    expect(files.length, "daftar kandidat Windows tak terbaca").toBeGreaterThanOrEqual(3);

    const tiga = files.slice(0, 3);
    const families = tiga.map((f) => CSS_FAMILY[f]);
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
