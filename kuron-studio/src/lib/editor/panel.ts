// Logika murni layout panel editor (tanpa DOM/Konva) — bisa diuji direct-import.
// Satuan: sidebar px, sheet % tinggi kontainer editor.
export const SIDEBAR_MIN = 192;
export const SIDEBAR_MAX = 512;
/** Default = 19rem @ root 16px: nilai lama `w-[19rem]`, bukan angka baru. */
export const SIDEBAR_DEFAULT = 304;
/** Lantai canvas: sidebar MAX = bodyW - nilai ini (agar canvas tak jepit habis). */
export const CANVAS_FLOOR = 280;
/** Drag dilepas di bawah ini = collapse (tak persist). */
export const COLLAPSE_THRESHOLD = 120;

/** Snap sheet = % TINGGI KONTAINER (revisi full-drag: 100% boleh tutup canvas). */
export const SNAP_POINTS = [40, 70, 100] as const;
export const SNAP_DEFAULT = 40;
/** Wilayah sheet = penuh kontainer (dulu 60% → canvas selalu ≥40%). */
export const SHEET_REGION_PCT = 100;

/** Batas atas dinamis dari lebar kontainer body. Tanpa bodyW → fallback SIDEBAR_MAX. */
export function sidebarMax(bodyW?: number): number {
  if (!Number.isFinite(bodyW as number) || (bodyW as number) <= 0) return SIDEBAR_MAX;
  return Math.max(SIDEBAR_MIN, Math.round((bodyW as number) - CANVAS_FLOOR));
}

/** Clamp lebar sidebar ke [MIN, MAX dinamis] + bulatkan (px). */
export function clampWidth(w: number, bodyW?: number): number {
  if (!Number.isFinite(w)) return SIDEBAR_DEFAULT;
  return Math.min(sidebarMax(bodyW), Math.max(SIDEBAR_MIN, Math.round(w)));
}

/** Snap terdekat dari daftar SNAP_POINTS. Di luar rentang → ujung terdekat. */
export function nearestSnap(pct: number): number {
  if (!Number.isFinite(pct)) return SNAP_DEFAULT;
  let best: number = SNAP_POINTS[0];
  for (const s of SNAP_POINTS) {
    if (Math.abs(s - pct) < Math.abs(best - pct)) best = s;
  }
  return best;
}

/** Tinggi sheet (% kontainer) untuk snap tertentu. */
export function sheetHeightPct(snap: number): number {
  return (snap / 100) * SHEET_REGION_PCT;
}

/** Ambang keluar dari full-sheet via drag bawah (persen kontainer). */
export const SHEET_EXIT_THRESHOLD = 85;
