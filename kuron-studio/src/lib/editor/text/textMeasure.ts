import Konva from "konva";
import layout from "../../contracts/layout.json";
import type { MeasureApi } from "./textLayout";

/**
 * Pengukuran teks untuk layout (export-text-parity P1).
 *
 * Dipakai BERSAMA oleh kanvas (menggambar) dan panel (menyimpan layout).
 * Kalau keduanya mengukur dengan cara berbeda, layout yang tersimpan tidak
 * akan sama dengan yang terlihat — persis masalah yang change ini tutup.
 */

/**
 * Font overlay = font yang SAMA dengan yang dipakai export (grup 4,
 * pendekatan "pakai font sistem").
 *
 * Kenapa bukan Inter Variable lagi: `ab_glyph` di Rust hanya bisa membaca
 * TTF/OTF, sedangkan repo ini tidak membundel Inter dalam bentuk itu
 * (`@fontsource-variable/inter` hanya mengirim woff2). Daripada menambah aset
 * biner, paritas dicapai dengan memakai font sistem yang sama di dua sisi.
 *
 * Urutan string ini WAJIB mencerminkan `windowsFonts` di `layout.json`
 * (dipakai `candidate_font_paths()` Rust di Windows). Dijaga test kontrak
 * lintas berkas; kalau salah satu daftar diubah, ubah JSON-nya.
 */
export const OVERLAY_FONT_STACK: string = layout.overlayFontStack;

/**
 * Berat huruf HARUS sama dengan TTF sistem yang dimuat Rust (regular).
 * Sebelumnya 600 — itu salah satu sebab lebar teks beda.
 */
export const OVERLAY_FONT_STYLE = "normal";

/** Keluarga font overlay (tanpa fallback `sans-serif` milik browser). */
export function overlayFontFamily(): string {
  return OVERLAY_FONT_STACK;
}

// Node pengukur dilepas dari stage dan dipakai ulang. `fontSize` yang masuk
// juga px halaman, jadi hasilnya px halaman (bukan px layar).
let measureNode: Konva.Text | null = null;

export const textMeasure: MeasureApi = {
  width(line, fontSize) {
    if (!measureNode) measureNode = new Konva.Text({});
    measureNode.fontFamily(overlayFontFamily());
    measureNode.fontStyle(OVERLAY_FONT_STYLE);
    measureNode.fontSize(fontSize);
    measureNode.text(line);
    return measureNode.getTextWidth();
  },
};
