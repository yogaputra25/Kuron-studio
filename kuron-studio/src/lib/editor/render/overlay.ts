// Overlay teks terjemahan (FE-5 canvas-redraw-split 2.2).
// Pindah verbatim dari redraw(): bg ngikut bentuk bubble + teks per baris
// di chord (cermin draw_layout Rust) atau blok tengah. Putih/hitam = konstanta
// fungsional (lihat themeColors.ts), bukan token tema.
import Konva from "konva";
import type { BubbleBox, BubbleTranslation } from "../../contracts/types";
import { computeTextLayout, lineBands } from "../text/textLayout";
import { OVERLAY_FONT_STYLE, overlayFontFamily, textMeasure } from "../text/textMeasure";

export interface OverlayOpts {
  /** Group bubble — overlay ditempel di sini (append-only). */
  group: Konva.Group;
  bubble: BubbleBox;
  showTranslation: boolean;
  /** Terjemahan untuk bubble ini (undefined = tak ada). */
  translation: BubbleTranslation | undefined;
  /** True bila editor in-bubble terbuka untuk bubble ini. */
  editing: boolean;
  scale: number;
  shapeRel: Array<[number, number]> | null;
}

export function appendOverlay(o: OverlayOpts): void {
  const { group, bubble: b, translation: tr, scale: s, shapeRel } = o;
  if (!o.showTranslation) return;
  if (!tr?.translated || o.editing) return;
  // Layout teks = SATU sumber (export-text-parity P1). Dihitung dalam
  // px HALAMAN lalu diskalakan untuk tampilan: nilainya tak berubah
  // saat lebar pane berubah, dan export memakai angka yang sama.
  const lay = computeTextLayout(tr.translated, b.w, b.h, textMeasure, { fontSize: tr.fontSize, shape: shapeRel });
  const pad = lay.padding * s;
  const boxW = Math.max(12, (b.w - lay.padding * 2) * s);
  const boxH = Math.max(12, (b.h - lay.padding * 2) * s);

  // Background mengikuti bentuk bubble (bukan kotak inset): polygon → isi
  // polygon, ellipse → isi elips, rect → isi penuh. Paritas dengan export
  // (`paint_patch` di render/page.rs) yang mengisi shape. Ukuran font tetap
  // dari `computeTextLayout` (auto-fit terbesar yang muat).
  const bgFill = tr.needsWhitePatch ? "#ffffff" : "rgba(0,0,0,0.78)";
  if (shapeRel && shapeRel.length >= 3) {
    group.add(
      new Konva.Line({
        points: shapeRel.flatMap(([px, py]) => [px * s, py * s]),
        closed: true,
        fill: bgFill,
        listening: false,
      }),
    );
  } else if (b.kind === "ellipse") {
    group.add(
      new Konva.Ellipse({
        x: (b.w * s) / 2,
        y: (b.h * s) / 2,
        radiusX: (b.w * s) / 2,
        radiusY: (b.h * s) / 2,
        fill: bgFill,
        listening: false,
      }),
    );
  } else {
    group.add(
      new Konva.Rect({
        x: 0,
        y: 0,
        width: b.w * s,
        height: b.h * s,
        fill: bgFill,
        cornerRadius: 6,
        listening: false,
      }),
    );
  }
  const txtFill = tr.needsWhitePatch ? "#111" : "#fff";
  // text-fit-polygon: bila shape ada, tiap baris digambar sendiri
  // tengah pada chord-nya (cermin `draw_layout` Rust). Tanpa shape =
  // satu blok tengah kotak seperti sebelumnya.
  const bands = lineBands(shapeRel, b.w, b.h, lay.padding, lay.lines.length, lay.fontSize, lay.lineHeight);
  if (bands) {
    lay.lines.forEach((ln, j) => {
      const band = bands[j];
      const t = new Konva.Text({
        x: band.x0 * s,
        y: band.top * s,
        width: Math.max(1, (band.x1 - band.x0) * s),
        text: ln,
        fontSize: lay.fontSize * s,
        lineHeight: 1,
        wrap: "none",
        fontFamily: overlayFontFamily(),
        fontStyle: OVERLAY_FONT_STYLE,
        align: "center",
        fill: txtFill,
        listening: false,
      });
      group.add(t);
      t.moveToTop();
    });
  } else {
  const txt = new Konva.Text({
    x: pad,
    y: pad,
    width: boxW,
    // Baris sudah diputuskan `computeTextLayout`; di sini gambar apa
    // adanya. `wrap: "none"` wajib — kalau Konva memutus ulang,
    // kanvas dan export bisa berbeda lagi.
    text: lay.lines.join("\n"),
    fontSize: lay.fontSize * s,
    lineHeight: lay.lineHeight,
    wrap: "none",
    fontFamily: overlayFontFamily(),
    fontStyle: OVERLAY_FONT_STYLE,
    align: "center",
    fill: "#fff",
    listening: false,
  });

  // Kunci tinggi kotak supaya `verticalAlign: middle` bekerja: Konva
  // menghitung ruang bebas dari `getHeight()`, dan selama height belum
  // di-set nilainya = tinggi teks (ruang bebas 0, teks nempel di atas).
  txt.height(boxH);
  txt.verticalAlign("middle");

  if (tr.needsWhitePatch) txt.fill("#111");
  txt.moveToTop();
  group.add(txt);
  }
}
