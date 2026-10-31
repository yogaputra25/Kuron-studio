// Agregasi event detect_progress → state overlay (detect-ux-pack D2).
// Murni, unit-testable — meniru reduceProgress (batch.ts).
import type { DetectProgress } from "../contracts/types";

export interface DetectFailure {
  pageId: string;
  file: string;
  message: string;
}

export interface DetectState {
  done: number;
  total: number;
  /** Nama file halaman yang sedang diproses ("" bila tak diketahui). */
  file: string;
  failed: DetectFailure[];
  finished: boolean;
}

export const initialDetectState = (total: number): DetectState => ({
  done: 0,
  total,
  file: "",
  failed: [],
  finished: false,
});

/** Event wire + nama file (di-resolve pemanggil dari id → path). */
export interface DetectEvent extends DetectProgress {
  file: string;
}

/** Satu event = satu halaman selesai (sukses/gagal terkumpul, batch lanjut). */
export function reduceDetectProgress(state: DetectState, ev: DetectEvent): DetectState {
  const failed = ev.ok
    ? state.failed
    : [...state.failed, { pageId: ev.pageId, file: ev.file, message: ev.message }];
  return {
    done: ev.done,
    total: ev.total,
    file: ev.file,
    failed,
    finished: ev.total > 0 && ev.done >= ev.total,
  };
}

/** Persen 0–100; total 0 → 0 (tanpa error bagi-nol). */
export function detectPercent(s: Pick<DetectState, "done" | "total">): number {
  if (s.total <= 0) return 0;
  return Math.min(100, Math.max(0, Math.round((s.done / s.total) * 100)));
}
