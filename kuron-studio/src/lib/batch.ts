import type { TranslateProgress } from "./types";

export interface BatchState {
  done: number;
  total: number;
  failed: string[];
  messages: Record<string, string>;
}

/** Agregasi event translate_progress → state bar (pure, unit-testable). */
export function reduceProgress(state: BatchState, ev: TranslateProgress): BatchState {
  const messages = { ...state.messages, [ev.pageId]: ev.message };
  const failed = ev.ok
    ? state.failed.filter((id) => id !== ev.pageId)
    : [...new Set([...state.failed, ev.pageId])];
  return { done: ev.done, total: ev.total, failed, messages };
}

export const initialBatchState = (total: number): BatchState => ({
  done: 0,
  total,
  failed: [],
  messages: {},
});
