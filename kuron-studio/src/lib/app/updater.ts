// Bridge updater Tauri (FE-7 tauri-event-bridge 3.1).
// Satu-satunya modul yang mengimpor plugin-updater/plugin-process.
// Dynamic import: tetap lazy seperti semula (hanya saat user cek update).
export interface UpdateInfo {
  version: string;
}

/** Cek update; null = sudah terbaru. Throw bila plugin/backend gagal. */
export async function checkForUpdate(): Promise<UpdateInfo | null> {
  const { check } = await import("@tauri-apps/plugin-updater");
  const update = await check();
  if (!update) return null;
  const version = update.version;
  await update.downloadAndInstall();
  return { version };
}

/** Restart aplikasi (setelah instal update). */
export async function relaunchApp(): Promise<void> {
  const { relaunch } = await import("@tauri-apps/plugin-process");
  await relaunch();
}
