// Bridge notifikasi sistem (batch-done).
// Satu-satunya modul yang memanggil command plugin-notification — LEWAT
// `contracts/log.call` (mentah, bukan binding JS plugin). Alasan: binding JS
// plugin (`@tauri-apps/plugin-notification`) di desktop lewat Web
// Notification API (`window.Notification`), yang di WKWebView berstatus
// denied tanpa prompt — badge selamanya kuning dan kirim tak pernah sampai.
// Command Rust mentah (`plugin:notification|*`, izin `notification:default`)
// mem-bypass itu dan tercatat di Diagnostics sebagai JSON Lines.
// Gate mobile: no-op di iOS/Android (mobile.json tanpa capability).
// Best-effort: tak pernah melempar, batch tetap selesai tanpa notifikasi.
import { isMobile } from "./device";
import { call } from "../contracts/log";

/** Status izin notifikasi sisi plugin. "unknown" = tak bisa dicek (mobile
 *  atau invoke gagal) — bukan berarti ditolak. */
export type NotificationState = "granted" | "not-granted" | "unknown";

/** Baca status izin; tak pernah melempar (gagal baca = "unknown"). */
export async function notificationStatus(): Promise<NotificationState> {
  try {
    if (isMobile) return "unknown";
    const granted = await call<boolean>("plugin:notification|is_permission_granted");
    return granted ? "granted" : "not-granted";
  } catch {
    return "unknown";
  }
}

/** Minta izin; kembalikan status sesudahnya. Tak pernah melempar. */
export async function requestNotificationPermission(): Promise<NotificationState> {
  try {
    if (isMobile) return "unknown";
    const state = await call<string>("plugin:notification|request_permission");
    return state === "granted" ? "granted" : "not-granted";
  } catch {
    return "unknown";
  }
}

/** Hasil kirim mentah: null = perintah terkirim tanpa error, string = pesan
 *  error (invoke / OS / plugin). Tak pernah melempar. Catatan: null berarti
 *  OS menerima perintah — bukan jaminan banner tampil (macOS modern bisa
 *  menelan notifikasi API lama secara diam-diam). */
export async function trySendNotification(title: string, body: string): Promise<string | null> {
  try {
    if (isMobile) return "nonaktif di mobile";
    await call("plugin:notification|notify", { options: { title, body } });
    return null;
  } catch (e) {
    return String(e);
  }
}

/** Satu notifikasi ringkasan per batch run (jangan panggil per halaman). */
export async function notifyBatchDone(title: string, body: string): Promise<void> {
  try {
    if (isMobile) return;
    const st = await notificationStatus();
    if (st !== "granted") return;
    await trySendNotification(title, body);
  } catch {
    /* best-effort: batch tetap selesai tanpa notifikasi */
  }
}
