// Penjaga sinkron prop->lokal (FE-4, task 2.2).
// Runes-module pola FE-2; isi murni agar keputusan sync bisa diuji tanpa mount.
//
// Kepemilikan `bubbles` (sumber tulis tunggal):
// - PEMILIK = EditorPanel (`let bubbles` + `dirty`). Satu-satunya penulis.
// - CanvasEditor hanya terima SALINAN via props (`bubbles={bubbles}`) dan
//   mengembalikan niat via `onChange` — tak pernah tulis balik langsung.
// - `translation.bubbles` = salinan geometri hasil `syncGeometryToTranslation`
//   saat save; komit teks via `editTr`. Jangan samakan kedua array ini.
// Guard memakai revisi struktural (`rev`), bukan referensi: echo onSaved
// membawa konten identik dengan objek baru — referensi selalu beda, jadi
// perbandingan harus struktural agar save identik tak me-reset dirty.
export function rev(v: unknown): string {
  const s = JSON.stringify(v);
  return typeof s === "string" ? s : "";
}

export function structurallyEqual(a: unknown, b: unknown): boolean {
  return rev(a) === rev(b);
}

export interface BubblesSyncDecision {
  samePage: boolean;
  localBusy: boolean;
  incoming: unknown;
  current: unknown;
}

/** Cabang bubbles: resync bila ganti halaman, atau tenang + beda struktur. */
export function shouldApplyBubbles(d: BubblesSyncDecision): boolean {
  return !d.samePage || (!d.localBusy && !structurallyEqual(d.incoming, d.current));
}

export interface TranslationSyncDecision {
  busyTyping: boolean;
  /** rev(translation prop); "" = tak ada terjemahan masuk. */
  incomingRev: string;
  current: unknown;
}

/** Cabang translation: jangan timpa lokal yang lebih baru dengan prop basi. */
export function shouldApplyTranslation(d: TranslationSyncDecision): boolean {
  return !d.busyTyping && d.incomingRev !== "" && rev(d.current) !== d.incomingRev;
}
