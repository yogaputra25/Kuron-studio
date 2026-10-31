/**
 * Deteksi batch + overlay progres (FE-2): listener `detect_progress` + guard
 * generasi + visibilitas overlay. Diekstrak dari `App.svelte`.
 * Bentuk factory (D1): panggil sekali di komposisi; `cancel()` WAJIB
 * dipanggil saat unmount (D4) — komposisi menyimpannya dari `start()`.
 */
import { onDetectProgress } from "../app/events";
import { api } from "../contracts/api";
import type { Project } from "../contracts/types";
import { initialDetectState, reduceDetectProgress, type DetectState } from "./detectProgress";

export interface DetectRun {
  readonly state: DetectState | null;
  readonly hidden: boolean;
  isCurrent(gen: number): boolean;
  /**
   * Mulai run deteksi: pasang listener, jalankan batch, ambil project terbaru.
   * `done` resolve `Project` bila selesai valid, `null` bila basi/dibatalkan,
   * reject bila batch gagal. `cancel()` melepas listener + membuang hasil.
   */
  start(
    projectId: string,
    pageIds: string[],
    names: Map<string, string>,
  ): { done: Promise<Project | null>; cancel(): void; gen: number };
  /** Tutup overlay: selesai → buang state; jalan → sembunyikan saja. */
  closeOverlay(): void;
}

export function createDetectRun(): DetectRun {
  let detectState = $state<DetectState | null>(null);
  let detectHidden = $state(false);
  let detectGen = 0;

  return {
    get state() {
      return detectState;
    },
    get hidden() {
      return detectHidden;
    },
    isCurrent(gen: number) {
      return gen === detectGen;
    },
    start(projectId: string, pageIds: string[], names: Map<string, string>) {
      const myGen = ++detectGen;
      detectState = initialDetectState(pageIds.length);
      detectHidden = false;
      let unlisten: (() => void) | null = null;
      let cancelled = false;
      const cancel = () => {
        cancelled = true;
        detectGen++;
        unlisten?.();
      };
      const done: Promise<Project | null> = (async () => {
        unlisten = onDetectProgress((e) => {
          if (myGen !== detectGen || !detectState) return;
          detectState = reduceDetectProgress(detectState, {
            ...e.payload,
            file: names.get(e.payload.pageId) ?? "",
          });
        });
        // Yield sekali: cancel() yang dipanggil sinkron tepat setelah start()
        // harus menang sebelum guard dievaluasi (dulu dijamin oleh
        // `await listen(...)`; helper mendaftar tanpa yield).
        await Promise.resolve();
        if (cancelled || myGen !== detectGen) {
          unlisten();
          return null;
        }
        try {
          await api.detectBatch(pageIds);
          if (myGen !== detectGen) return null;
          return await api.getProject(projectId);
        } catch (e) {
          unlisten();
          throw e;
        }
      })();
      return { done, cancel, gen: myGen };
    },
    closeOverlay() {
      if (detectState?.finished) detectState = null;
      else detectHidden = true;
    },
  };
}
