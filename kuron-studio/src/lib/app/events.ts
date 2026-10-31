// Bridge event Tauri (FE-7 tauri-event-bridge 1.1).
// Satu-satunya tempat komponen berlangganan event: tiap helper
// mengembalikan unlisten SINKRON yang wajib disimpan pemanggil dan
// dipanggil di onDestroy/cleanup. Registrasi Tauri itu async
// (Promise<unlisten>); closure menunggu janji dulu sehingga unmount
// sebelum registrasi selesai tetap membersihkan (tak ada handler basi).
import { listen, type EventCallback } from "@tauri-apps/api/event";
import { getCurrentWebview, type DragDropEvent } from "@tauri-apps/api/webview";
import type { DetectProgress, TranslateProgress } from "../contracts/types";

/** Bungkus janji registrasi jadi unlisten sinkron anti-bocor. */
function toSyncUnlisten(pending: Promise<() => void>): () => void {
  let off: (() => void) | null = null;
  let dead = false;
  pending.then(
    (u) => { if (dead) u(); else off = u; },
    () => { /* registrasi gagal: tak ada yang perlu dilepas */ },
  );
  return () => { dead = true; off?.(); };
}

export function onDetectProgress(cb: EventCallback<DetectProgress>): () => void {
  return toSyncUnlisten(listen<DetectProgress>("detect_progress", cb));
}

export function onTranslateProgress(cb: EventCallback<TranslateProgress>): () => void {
  return toSyncUnlisten(listen<TranslateProgress>("translate_progress", cb));
}

export function onDragDrop(cb: EventCallback<DragDropEvent>): () => void {
  return toSyncUnlisten(getCurrentWebview().onDragDropEvent(cb));
}
