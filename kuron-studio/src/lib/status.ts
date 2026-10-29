import type { Page, PageStatus } from "./types";

/**
 * Warna + label status. Satu sumber kebenaran: `ui/StatusBadge.svelte` yang
 * merender, file ini yang memegang datanya (biar bisa dites tanpa DOM).
 *
 * Tone memakai token semantik dari app.css, bukan palet Tailwind mentah —
 * supaya light/dark theme otomatis ikut dan kontrasnya sudah diuji WCAG AA.
 */
  const TONE: Record<PageStatus, { cls: string; label: string }> = {
    idle: { cls: "bg-raised text-ink-3", label: "idle" },
    detecting: { cls: "bg-info-soft text-info", label: "deteksi" },
    detected: { cls: "bg-cyan-soft text-cyan", label: "terdeteksi" },
    noBubbles: { cls: "bg-warn-soft text-warn", label: "tanpa bubble" },
    translating: { cls: "bg-info-soft text-info", label: "menerjemah" },
    translated: { cls: "bg-success-soft text-success", label: "diterjemah" },
    failed: { cls: "bg-danger-soft text-danger", label: "gagal" },
  };

/** Kelas badge untuk sebuah status. */
export const statusBadgeClass = (s: PageStatus): string => TONE[s].cls;

/** Label Bahasa Indonesia, untuk `title`/aria. */
export const statusLabel = (s: PageStatus): string => TONE[s].label;

/** Filename order, natural-ish via localeCompare numeric. */
export function sortPagesByName(pages: Page[]): Page[] {
  return [...pages].sort((a, b) =>
    a.path.localeCompare(b.path, undefined, { numeric: true }),
  );
}

/** Urutan status untuk sortir: Belum → Proses → Sudah → Gagal (cerminan bucket kanban). */
const STATUS_RANK: Record<PageStatus, number> = {
  idle: 0,
  detected: 0,
  noBubbles: 0,
  detecting: 1,
  translating: 1,
  translated: 2,
  failed: 3,
};

/** Sortir stabil: status (bucket kanban), tiebreak nama natural. */
export function sortPagesByStatus(pages: Page[], dir: 1 | -1 = 1): Page[] {
  return [...pages]
    .map((p, i) => ({ p, i }))
    .sort((a, b) => {
      const r = (STATUS_RANK[a.p.status] ?? 99) - (STATUS_RANK[b.p.status] ?? 99);
      if (r !== 0) return dir * r;
      const n = a.p.path.localeCompare(b.p.path, undefined, { numeric: true });
      if (n !== 0) return dir * n;
      return a.i - b.i;
    })
    .map(({ p }) => p);
}

/** Sortir stabil: jumlah bubble, tiebreak nama natural. */
export function sortPagesByBubbles(pages: Page[], dir: 1 | -1 = 1): Page[] {
  return [...pages]
    .map((p, i) => ({ p, i }))
    .sort((a, b) => {
      const r = a.p.bubbles.length - b.p.bubbles.length;
      if (r !== 0) return dir * r;
      const n = a.p.path.localeCompare(b.p.path, undefined, { numeric: true });
      if (n !== 0) return dir * n;
      return a.i - b.i;
    })
    .map(({ p }) => p);
}

/** Bubble terjemahan minimal: cukup field `translated` untuk completeness. */
export interface TrBubble {
  translated: string;
}

/** Lengkap = flag manual ATAU semua bubble ada isi (non-blank). */
export function translationComplete(bubbles: TrBubble[], manualDone?: boolean): boolean {
  if (manualDone) return true;
  return bubbles.length > 0 && bubbles.every((b) => b.translated.trim() !== "");
}

/**
 * Status setelah simpan geometri bubble (mirror `persist_bubbles` backend):
 * halaman berterjemahan tak didemosikan; tanpa terjemahan ikut jumlah bubble.
 */
export function statusAfterBubbleSave(prev: PageStatus, hasTranslation: boolean, count: number): PageStatus {
  if (hasTranslation && (prev === "translated" || prev === "translating")) return prev;
  return count > 0 ? "detected" : "noBubbles";
}

/**
 * Status setelah simpan terjemahan (mirror `persist_result` backend):
 * lengkap → translated; sebagian → status lama (transient translating
 * jatuh ke detected — backend memakai stash pra-klaim bila ada).
 */
export function statusAfterTranslationSave(prev: PageStatus, bubbles: TrBubble[], manualDone?: boolean): PageStatus {
  if (translationComplete(bubbles, manualDone)) return "translated";
  if (prev === "translating") return "detected";
  return prev;
}

/** Nama file dari path (`a/b/c.jpg` → `c.jpg`). */
export function fileName(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

/** Folder induk langsung (`a/Ch08/p037.jpg` → `Ch08`; tanpa folder → ""). */
export function parentFolder(path: string): string {
  const parts = path.split(/[/\\]/).filter((p) => p.length > 0);
  return parts.length > 1 ? parts[parts.length - 2] : "";
}
