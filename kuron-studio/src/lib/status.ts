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
