// Draft ketikan kartu terjemahan (FE-4, task 1.3).
// Runes-module factory pola FE-2: state reaktif + timer terkungkung di sini,
// EditorPanel hanya memasok akses baris/bubble dan commit (editTr).
//
// Invarian (diwarisi fix-edit-stuck, dikunci unit test):
// - `cur` PLAIN (bukan $state): nol render/commit per huruf; DOM textarea
//   milik browser penuh. Satu-satunya $state adalah `dirty` (1 render
//   per rangkaian-ketik untuk indikator "belum tersimpan").
// - Commit hanya di batas alami: blur/pindah/Save/compositionend/jaring
//   10 dtk — bukan tiap jeda mikro. Timer-jaring = pengaman sesi-ditinggal.
// - `revert()` = buang cache tanpa commit (semantik tombol Esc / seleksi null);
//   tidak menyentuh penanganan Esc global (tetap milik panel).
export interface DraftValue {
  row: number;
  original: string;
  reading: string;
  translated: string;
}

export type DraftField = "original" | "reading" | "translated";

/** Jaring pengaman sesi-ditinggal, bukan ritme kerja. */
export const DRAFT_SAFETY_MS = 10_000;

export interface DraftTextDeps {
  /** Baris terjemahan aktif (-1 = tak ada seleksi). */
  getRow: () => number;
  /** Isi komit terakhir baris (undefined = tak ada terjemahan). */
  getBubble: (row: number) => { original: string; reading: string; translated: string } | undefined;
  /** Tulis satu field via jalur existing (editTr). */
  commit: (row: number, field: DraftField, value: string) => void;
}

export function useDraftText(deps: DraftTextDeps) {
  let cur: DraftValue | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let composing = false;
  let dirty = $state(false);

  function arm() {
    if (!timer) timer = setTimeout(() => { timer = null; doCommit(); }, DRAFT_SAFETY_MS);
  }

  function disarm() {
    if (timer) { clearTimeout(timer); timer = null; }
  }

  /** Ketikan kartu → cache plain; antre jaring bila tak tengah komposisi. */
  function input(field: DraftField, v: string) {
    const row = deps.getRow();
    if (row < 0) return;
    if (!cur || cur.row !== row) {
      const b = deps.getBubble(row);
      if (!b) return;
      cur = { row, original: b.original, reading: b.reading, translated: b.translated };
    }
    cur[field] = v;
    if (!dirty) dirty = true;
    if (composing) return;
    arm();
  }

  /** IME mulai: jangan antre commit tengah komposisi. */
  function compStart() {
    composing = true;
    disarm();
  }

  /** IME selesai = batas kata alami → commit langsung. */
  function compEnd() {
    composing = false;
    if (cur) doCommit();
  }

  /** Commit field kotor via commit existing (sekali, bukan per huruf). */
  function doCommit() {
    if (composing) return;
    disarm();
    if (!cur) return;
    const { row, original, reading, translated } = cur;
    cur = null;
    dirty = false;
    const b = deps.getBubble(row);
    if (!b) return;
    if (b.original !== original) deps.commit(row, "original", original);
    if (b.reading !== reading) deps.commit(row, "reading", reading);
    if (b.translated !== translated) deps.commit(row, "translated", translated);
  }

  /** Buang cache tanpa commit (Esc / seleksi dikosongkan / reset). */
  function revert() {
    disarm();
    cur = null;
    dirty = false;
  }

  /** Lepas timer (unmount). Tanpa commit — commit eksplisit milik pemanggil. */
  function dispose() {
    disarm();
  }

  return {
    get cur() { return cur; },
    get dirty() { return dirty; },
    input,
    compStart,
    compEnd,
    commit: doCommit,
    revert,
    dispose,
  };
}

export type DraftText = ReturnType<typeof useDraftText>;
