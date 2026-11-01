import { describe, expect, it } from "vitest";
import {
  LAYOUT_FILL,
  LAYOUT_LINE_HEIGHT,
  LAYOUT_MAX_FONT,
  LAYOUT_MIN_FONT,
  LAYOUT_PADDING,
  bodySpan,
  chordAt,
  computeTextLayout,
  lineBands,
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
    // innerH = 20 (padding 0, fill 1). Satu baris butuh fontSize * 1.25 <= 20 → fs <= 16.
    const out = computeTextLayout("aaaa bbbb", 100, 20, mono, { padding: 0, fillFactor: 1 });
    expect(out.fontSize).toBe(16);
    expect(out.lines).toEqual(["aaaa bbbb"]);
  });

  it("berhenti di batas minimum untuk kotak yang sangat kecil", () => {
    const out = computeTextLayout("teks panjang sekali", 40, 12, mono, { padding: 0 });
    expect(out.fontSize).toBe(LAYOUT_MIN_FONT);
  });

  it("padding mempersempit lebar efektif dan memendekkan blok", () => {
    // Lebar dalam = (100 - 2*padding) × fill → 19 a (@fs10 = 95px) tak muat sebaris.
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

describe("faktor isi (fillFactor)", () => {
  it("default menyisakan margin: blok <= fill × tinggi kotak-dalam", () => {
    const text = "kata ".repeat(20).trim();
    const lay = computeTextLayout(text, 200, 200, mono);
    const inner = 200 - LAYOUT_PADDING * 2;
    expect(lay.lines.length * lay.fontSize * LAYOUT_LINE_HEIGHT)
      .toBeLessThanOrEqual(inner * LAYOUT_FILL + 1e-9);
  });

  it("faktor lebih kecil = font lebih kecil atau sama", () => {
    const text = "kata ".repeat(20).trim();
    const penuh = computeTextLayout(text, 200, 200, mono, { fillFactor: 1 });
    const hemat = computeTextLayout(text, 200, 200, mono, { fillFactor: 0.5 });
    expect(hemat.fontSize).toBeLessThanOrEqual(penuh.fontSize);
  });

  it("manual user tidak kena faktor isi", () => {
    const a = computeTextLayout("aaaa bbbb cccc", 60, 400, mono, { fontSize: 20 });
    const b = computeTextLayout("aaaa bbbb cccc", 60, 400, mono, { fontSize: 20, fillFactor: 0.5 });
    expect(b.lines).toEqual(a.lines);
  });
});

describe("jalur polygon (text-fit-polygon)", () => {
  // Elips kasar dalam kotak 200x200: sempit di ujung, penuh di tengah.
  const ellipse: Array<[number, number]> = [
    [100, 0], [150, 14], [180, 50], [190, 100],
    [180, 150], [150, 186], [100, 200], [50, 186],
    [20, 150], [10, 100], [20, 50], [50, 14],
  ];

  it("chord sempit di ujung, lebar di tengah", () => {
    const top = chordAt(ellipse, 5, 100);
    const mid = chordAt(ellipse, 100, 100);
    expect(top).not.toBeNull();
    expect(mid).not.toBeNull();
    expect(mid!.x1 - mid!.x0).toBeGreaterThan(top!.x1 - top!.x0);
    // Tengah elips ≈ penuh.
    expect(mid!.x1 - mid!.x0).toBeGreaterThan(150);
  });

  it("tanpa potongan = null", () => {
    expect(chordAt(ellipse, -10, 100)).toBeNull();
    expect(chordAt(ellipse, 300, 100)).toBeNull();
  });

  it("shape kotak = tanpa shape (jalur lama byte-identik)", () => {
    const rect: Array<[number, number]> = [[0, 0], [200, 0], [200, 100], [0, 100]];
    const a = computeTextLayout("halo dunia yang panjang", 200, 100, mono);
    const b = computeTextLayout("halo dunia yang panjang", 200, 100, mono, { shape: rect });
    expect(b).toEqual(a);
  });

  it("tiap baris muat chord pitanya", () => {
    const text = "kata ".repeat(30).trim();
    const lay = computeTextLayout(text, 200, 200, mono, { shape: ellipse, padding: 0 });
    const bands = lineBands(ellipse, 200, 200, 0, lay.lines.length, lay.fontSize, lay.lineHeight);
    expect(bands).not.toBeNull();
    expect(bands!.length).toBe(lay.lines.length);
    lay.lines.forEach((ln, j) => {
      expect(mono.width(ln, lay.fontSize)).toBeLessThanOrEqual(bands![j].x1 - bands![j].x0 + 1e-9);
    });
  });

  it("elips memaksa font lebih kecil daripada kotak", () => {
    const text = "kata ".repeat(40).trim();
    const box = computeTextLayout(text, 200, 200, mono, { padding: 0 });
    const poly = computeTextLayout(text, 200, 200, mono, { shape: ellipse, padding: 0 });
    expect(poly.fontSize).toBeLessThanOrEqual(box.fontSize);
  });

  it("manual adalah batas atas: susut sampai muat polygon", () => {
    const text = "kata ".repeat(40).trim();
    const lay = computeTextLayout(text, 200, 200, mono, { shape: ellipse, padding: 0, fontSize: 72 });
    expect(lay.fontSize).toBeLessThan(72);
    const bands = lineBands(ellipse, 200, 200, 0, lay.lines.length, lay.fontSize, lay.lineHeight);
    lay.lines.forEach((ln, j) => {
      expect(mono.width(ln, lay.fontSize)).toBeLessThanOrEqual(bands![j].x1 - bands![j].x0 + 1e-9);
    });
  });

  it("manual yang sudah muat tidak diubah", () => {
    const lay = computeTextLayout("hi", 200, 200, mono, { shape: ellipse, padding: 0, fontSize: 20 });
    expect(lay.fontSize).toBe(20);
    expect(lay.lines).toEqual(["hi"]);
  });

  it("ekor dikeluarkan dari area teks", () => {
    // Badan 200x150 + ekor segitiga sempit ke bawah sampai y=300.
    const tailed: Array<[number, number]> = [
      [0, 0], [200, 0], [200, 150], [110, 150], [100, 300], [90, 150], [0, 150],
    ];
    const span = bodySpan(tailed, 200, 300, 0);
    expect(span).not.toBeNull();
    expect(span!.hi).toBeLessThanOrEqual(152);
    const text = "kata ".repeat(30).trim();
    const lay = computeTextLayout(text, 200, 300, mono, { shape: tailed, padding: 0 });
    // Font tidak terjepit selebar ekor (~20px): tetap belasan ke atas.
    expect(lay.fontSize).toBeGreaterThan(10);
    const bands = lineBands(tailed, 200, 300, 0, lay.lines.length, lay.fontSize, lay.lineHeight);
    for (const b of bands!) {
      expect(b.top + lay.fontSize * lay.lineHeight).toBeLessThanOrEqual(152);
    }
  });
});
