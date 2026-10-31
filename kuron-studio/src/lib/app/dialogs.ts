// Bridge dialog Tauri (FE-7 tauri-event-bridge 2.2).
// Satu-satunya modul yang mengimpor @tauri-apps/plugin-dialog;
// normalisasi hasil (string | string[] | null) jadi bentuk sederhana.
import { open, save } from "@tauri-apps/plugin-dialog";
import { ARCHIVE_EXTS, IMAGE_EXTS, PDF_EXTS } from "../contracts/types";

/** Pilih satu folder; null bila dibatalkan. */
export async function pickFolder(title?: string): Promise<string | null> {
  const r = title === undefined ? await open({ directory: true }) : await open({ directory: true, title });
  return typeof r === "string" ? r : null;
}

/** Pilih file gambar/arsip/PDF (multi); selalu array (kosong bila batal). */
export async function pickFiles(): Promise<string[]> {
  const r = await open({
    multiple: true,
    // iOS: salin pilihan ke sandbox app (default pun copy, ditulis eksplisit
    // supaya tidak tergantung default plugin). Folder picker tak ada di iOS.
    fileAccessMode: "copy",
    filters: [
      { name: "Images", extensions: [...IMAGE_EXTS] },
      { name: "Archives", extensions: [...ARCHIVE_EXTS] },
      { name: "PDF documents", extensions: [...PDF_EXTS] },
    ],
  });
  if (Array.isArray(r)) return r;
  return typeof r === "string" ? [r] : [];
}

/** Dialog simpan satu file; null bila dibatalkan. */
export async function saveAs(defaultPath: string, filterName: string, extensions: string[]): Promise<string | null> {
  const r = await save({ defaultPath, filters: [{ name: filterName, extensions }] });
  return typeof r === "string" ? r : null;
}
