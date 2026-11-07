// Judul jendela native (window-title) + sentuhan perhatian dock.
// Satu-satunya modul yang mengimpor @tauri-apps/api/window; format:
// "<NamaProject> — Kuron Studio", fallback "Kuron Studio" saat tak ada
// project. Tanpa penanda unsaved: codebase belum punya konsep
// dirty/unsaved (tak ada state semacam itu di src/).
import { getCurrentWindow, UserAttentionType } from "@tauri-apps/api/window";

export const APP_TITLE = "Kuron Studio";

/** Judul murni (unit-testable): nama kosong/blank → fallback. */
export function buildWindowTitle(projectName: string | null | undefined): string {
  const n = (projectName ?? "").trim();
  return n ? `${n} — Kuron Studio` : APP_TITLE;
}

/** Terapkan judul ke window native; gagal diam (browser/test tanpa
 *  runtime Tauri, atau capability belum mengizinkan). */
export async function syncWindowTitle(projectName: string | null | undefined): Promise<void> {
  try {
    await getCurrentWindow().setTitle(buildWindowTitle(projectName));
  } catch {
    /* bukan runtime Tauri — abaikan */
  }
}

/** Minta perhatian OS saat batch selesai (pentalan dock/taskbar) — HANYA
 *  bila jendela sedang tidak fokus (user pergi ke app lain). Jalur ini
 *  100% andal di dev maupun rilis: tak butuh bundle id, bundle permission,
 *  maupun API notifikasi modern. Tak pernah melempar. */
export async function nudgeWindowForBatchDone(): Promise<void> {
  try {
    const w = getCurrentWindow();
    if (await w.isFocused()) return;
    await w.requestUserAttention(UserAttentionType.Critical);
  } catch {
    /* bukan runtime Tauri — abaikan */
  }
}
