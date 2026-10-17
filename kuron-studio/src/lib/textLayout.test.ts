import { describe, expect, it } from "vitest";
import {
  LAYOUT_LINE_HEIGHT,
  LAYOUT_MAX_FONT,
  LAYOUT_MIN_FONT,
  LAYOUT_PADDING,
  computeTextLayout,
  wrapLines,
  type MeasureApi,
} from "./textLayout";

// Pengukur palsu: lebar = jumlah karakter * fontSize * 0.5.
// Sengaja deterministik supaya perilaku algoritmanya bisa diuji tanpa Konva.
const mono: MeasureApi = { width: (line, fontSize) => line.length * fontSize * 0.5 };

describe("wrapLines", () => {
  it("menggabung kata selama masih muat", () => {
    // 19 karakter * 5px (@fs10) = 95 <= 100.
    expect(wrapLines("aaaa bbbb cccc dddd", 100, 10, mono)).toEqual(["aaaa bbbb cccc dddd"]);
  });

  it("memecah ke baris baru saat melebihi lebar", () => {
    const lines = wrapLines("aaaa bbbb cccc dddd eeee", 100, 10, mono);
    expect(lines.length).toBeGreaterThan(1);
    for (const l of lines) expect(mono.width(l, 10)).toBeLessThanOrEqual(100);
  });

  it("memecah kata yang lebih lebar dari kotak (tidak dibiarkan luber)", () => {
    // 10 karakter * 5px = 50 > 30, tanpa spasi → harus dipecah per karakter.
    const lines = wrapLines("abcdefghij", 30, 10, mono);
    expect(lines).toEqual(["abcdef", "ghij"]);
    for (const l of lines) expect(mono.width(l, 10)).toBeLessThanOrEqual(30);
  });

  it("baris kosong dipertahankan sebagai baris kosong", () => {
    expect(wrapLines("a\n\nb", 1000, 10, mono)).toEqual(["a", "", "b"]);
  });
});

describe("computeTextLayout", () => {
  it("memilih ukuran terbesar yang bloknya masih muat tinggi kotak", () => {
    // innerH = 20 (padding 0). Satu baris butuh fontSize * 1.25 <= 20 → fs <= 16.
    const out = computeTextLayout("aaaa bbbb", 100, 20, mono, { padding: 0 });
    expect(out.fontSize).toBe(16);
    expect(out.lines).toEqual(["aaaa bbbb"]);
  });

  it("berhenti di batas minimum untuk kotak yang sangat kecil", () => {
    const out = computeTextLayout("teks panjang sekali", 40, 12, mono, { padding: 0 });
    expect(out.fontSize).toBe(LAYOUT_MIN_FONT);
  });

  it("padding mempersempit lebar efektif dan memendekkan blok", () => {
    // Lebar dalam = 100 - 8 = 92 → 18 karakter (@fs10 = 90px) muat, 19 tidak.
    const out = computeTextLayout("a".repeat(19), 100, 400, mono, { max: 10, min: 10 });
    expect(out.padding).toBe(LAYOUT_PADDING);
    expect(out.lines.length).toBe(2);
  });

  it("hasilnya murni fungsi dari teks + ukuran kotak (bukan dari layar)", () => {
    const a = computeTextLayout("halo dunia", 200, 80, mono);
    const b = computeTextLayout("halo dunia", 200, 80, mono);
    expect(b).toEqual(a);
    expect(a.lineHeight).toBe(LAYOUT_LINE_HEIGHT);
    expect(a.fontSize).toBeGreaterThanOrEqual(LAYOUT_MIN_FONT);
  });
});

describe("ukuran font manual (export-text-parity)", () => {
  it("dipakai apa adanya, auto-fit dilewati", () => {
    // Kotak besar: auto-fit akan memilih MAKS, tapi ukuran manual harus menang.
    expect(computeTextLayout("halo", 400, 400, mono).fontSize).toBe(LAYOUT_MAX_FONT);
    expect(computeTextLayout("halo", 400, 400, mono, { fontSize: 20 }).fontSize).toBe(20);
  });

  it("baris tetap dipecah pada ukuran manual", () => {
    const lay = computeTextLayout("aaaa bbbb cccc", 60, 400, mono, { fontSize: 20 });
    expect(lay.fontSize).toBe(20);
    expect(lay.lines.length).toBeGreaterThan(1);
    const inner = 60 - LAYOUT_PADDING * 2;
    for (const l of lay.lines) expect(mono.width(l, 20)).toBeLessThanOrEqual(inner);
  });

  it("null atau 0 kembali ke auto-fit", () => {
    expect(computeTextLayout("halo", 400, 400, mono, { fontSize: null }).fontSize).toBe(LAYOUT_MAX_FONT);
    expect(computeTextLayout("halo", 400, 400, mono, { fontSize: 0 }).fontSize).toBe(LAYOUT_MAX_FONT);
  });
});
