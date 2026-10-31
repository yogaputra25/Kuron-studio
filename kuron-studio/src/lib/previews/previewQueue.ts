// Antrean thumbnail grid (grid-preview-perf D1-D3): konkurensi terbatas di
// pemanggil, prioritas viewport, guard generasi di App. Murni — direct-import.
export const PREVIEW_CONCURRENCY = 4;
/** Prefetch di luar kartu terlihat: ~2 baris grid (8 kolom x 2 + margin). */
export const PREFETCH_MARGIN = 24;

/**
 * Pilih id berikutnya: kartu terlihat dulu, lalu prefetch N pertama dari
 * antrean (aproksimasi 2 baris awal sebelum observer melapor). Null = tunggu
 * intersection berikutnya. `done` = sudah ada thumb / gagal (jangan antre lagi).
 */
export function pickNext(
  pending: string[],
  visible: Set<string>,
  busy: Set<string>,
  done: (id: string) => boolean,
  prefetch: number = PREFETCH_MARGIN,
): string | null {
  for (const id of pending) {
    if (visible.has(id) && !busy.has(id) && !done(id)) return id;
  }
  let free = 0;
  for (const id of pending) {
    if (busy.has(id) || done(id)) continue;
    if (free++ < prefetch) return id;
    return null;
  }
  return null;
}
