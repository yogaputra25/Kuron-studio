/**
 * Antrean thumbnail grid (FE-2): 11 state + pump + retry + action observer.
 * Diekstrak dari `App.svelte`. Guard generasi dipertahankan: ganti halaman
 * membuang hasil basi. Bentuk factory (D1): panggil sekali di komposisi.
 */
import { api } from "../contracts/api";
import type { Page } from "../contracts/types";
import { sortPagesByName } from "../library/status";
import { PREFETCH_MARGIN, PREVIEW_CONCURRENCY, pickNext } from "./previewQueue";
import { summarizePreviews, type PreviewSample } from "./previewMetrics";


export interface ThumbnailQueue {
  readonly thumbs: Record<string, string>;
  readonly broken: Record<string, string>;
  readonly total: number;
  readonly done: number;
  readonly failed: number;
  loadThumbs(pages: Page[]): void;
  retryThumbs(pages: Page[]): void;
  /** Buang id yang tak ada di `live` (dipakai cleanPages/pindah folder). */
  pruneThumbs(live: Set<string>): void;
  /** Reset total (dipakai hapus project). */
  resetThumbs(): void;
  observeCard(node: HTMLElement, id: string): { destroy(): void };
}

export function createThumbnailQueue(): ThumbnailQueue {
  let thumbs = $state<Record<string, string>>({});
  let broken = $state<Record<string, string>>({});
  let loadGen = 0;
  let thumbPending: string[] = [];
  let thumbInflight = new Set<string>();
  let thumbActive = 0;
  let thumbByPath = new Map<string, string>();
  let visibleIds = new Set<string>();
  let thumbTotal = $state(0);
  let thumbDone = $state(0);
  let thumbFailed = $state(0);
  let thumbT0 = 0;
  let thumbFirstAt = 0;
  let thumbSamples: PreviewSample[] = [];

  function nextThumbId(): string | null {
    const done = (id: string) => !!thumbs[id] || !!broken[id];
    return pickNext(thumbPending, visibleIds, thumbInflight, done, PREFETCH_MARGIN);
  }

  /** Pompa worker sampai batas konkurensi; hasil basi (gen beda) dibuang. */
  function pumpThumbs(myGen: number) {
    while (thumbActive < PREVIEW_CONCURRENCY && myGen === loadGen) {
      const id = nextThumbId();
      if (!id) return; // tunggu intersection berikutnya
      thumbInflight.add(id);
      thumbActive++;
      const path = thumbByPath.get(id) ?? "";
      const t = performance.now();
      void api
        .getImagePreview(path)
        .then((url) => {
          if (myGen !== loadGen) return;
          thumbSamples.push({ ms: performance.now() - t, bytes: url.length });
          thumbs = { ...thumbs, [id]: url };
          if (!thumbFirstAt) thumbFirstAt = performance.now();
          thumbDone++;
        })
        .catch((e) => {
          if (myGen !== loadGen) return;
          broken = { ...broken, [id]: String(e) };
          thumbFailed++;
          thumbDone++;
        })
        .finally(() => {
          thumbInflight.delete(id);
          thumbActive--;
          if (myGen !== loadGen) return;
          if (thumbTotal > 0 && thumbDone >= thumbTotal) {
            console.info(`[preview] ${JSON.stringify(summarizePreviews(thumbSamples, thumbFirstAt - thumbT0))}`);
          }
          pumpThumbs(myGen);
        });
    }
  }

  return {
    get thumbs() {
      return thumbs;
    },
    get broken() {
      return broken;
    },
    get total() {
      return thumbTotal;
    },
    get done() {
      return thumbDone;
    },
    get failed() {
      return thumbFailed;
    },
    loadThumbs(pages: Page[]) {
      const myGen = ++loadGen;
      visibleIds = new Set<string>();
      thumbInflight = new Set<string>();
      thumbActive = 0;
      thumbByPath = new Map(pages.map((pg) => [pg.id, pg.path]));
      // Skip yang sudah broken: retry file sidecar tiap ganti project mahal.
      thumbPending = sortPagesByName(pages)
        .filter((pg) => !thumbs[pg.id] && !broken[pg.id])
        .map((pg) => pg.id);
      thumbTotal = thumbPending.length;
      thumbDone = 0;
      thumbFailed = 0;
      thumbT0 = performance.now();
      thumbFirstAt = 0;
      thumbSamples = [];
      pumpThumbs(myGen);
    },
    retryThumbs(pages: Page[]) {
      const live = new Set(pages.map((pg) => pg.id));
      for (const id of Object.keys(broken)) if (live.has(id)) delete broken[id];
      broken = { ...broken };
      this.loadThumbs(pages);
    },
    pruneThumbs(live: Set<string>) {
      for (const id of Object.keys(thumbs)) if (!live.has(id)) delete thumbs[id];
      for (const id of Object.keys(broken)) if (!live.has(id)) delete broken[id];
      thumbs = { ...thumbs };
      broken = { ...broken };
    },
    resetThumbs() {
      thumbs = {};
      broken = {};
      thumbTotal = 0;
      thumbDone = 0;
      thumbFailed = 0;
    },
    observeCard(node: HTMLElement, id: string) {
      const io = new IntersectionObserver(
        (es) => {
          for (const e of es) {
            if (e.isIntersecting) {
              visibleIds.add(id);
              pumpThumbs(loadGen);
            }
          }
        },
        { rootMargin: "400px 0px" },
      );
      io.observe(node);
      return {
        destroy() {
          io.disconnect();
        },
      };
    },
  };
}
