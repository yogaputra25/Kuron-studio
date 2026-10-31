/**
 * Seleksi multi-halaman + long-press (FE-2): diekstrak dari `App.svelte`.
 * Bentuk factory (D1): panggil sekali di komposisi; baca state lewat getter.
 * Timer long-press dimiliki modul ini dan WAJIB dibersihkan via `dispose()`
 * (dipanggil `App.svelte` di `onDestroy`) — lihat D4.
 */
import type { Page } from "../contracts/types";
import { rangeSelect } from "./library";
import { sortPagesByName } from "./status";

export interface Selection {
  readonly selectedIds: string[];
  readonly selAnchor: string | null;
  readonly selectMode: boolean;
  toggleSelect(id: string): void;
  selectAll(ids: string[]): void;
  /** Tahan gambar ±0.5 dtk = masuk mode pilih + pilih kartu itu (pola mobile). */
  press(pg: Page): void;
  /**
   * Klik kartu. Return true bila event dikonsumsi (jangan buka editor),
   * false bila klik biasa tanpa modifier (komposisi membuka editor).
   * `orderedIds` = urutan tampil untuk shift-range.
   */
  click(e: MouseEvent, pg: Page, orderedIds: string[]): boolean;
  reset(): void;
  cancelHold(): void;
  /** Keluar/masuk mode pilih tanpa mengubah isi seleksi. */
  setSelectMode(v: boolean): void;
  dispose(): void;
}

const HOLD_MS = 500;

export function createSelection(): Selection {
  let selectedIds = $state<string[]>([]);
  let selAnchor = $state<string | null>(null);
  let selectMode = $state(false);
  // Internal (bukan kontrak publik): samakan konvensi App lama — Timeout vs
  // number tergantung env. `ReturnType` di sini tepat karena tak diekspor.
  let holdTimer: ReturnType<typeof setTimeout> | null = null;
  let heldSelect = false;

  function toggleSelect(id: string) {
    selectedIds = selectedIds.includes(id)
      ? selectedIds.filter((x) => x !== id)
      : [...selectedIds, id];
    selAnchor = id;
  }

  return {
    get selectedIds() {
      return selectedIds;
    },
    get selAnchor() {
      return selAnchor;
    },
    get selectMode() {
      return selectMode;
    },
    toggleSelect,
    selectAll(ids: string[]) {
      selectedIds = [...ids];
      selAnchor = null;
    },
    press(pg: Page) {
      if (selectMode) return;
      heldSelect = false;
      if (holdTimer !== null) {
        clearTimeout(holdTimer);
        holdTimer = null;
      }
      holdTimer = setTimeout(() => {
        heldSelect = true;
        selectMode = true;
        if (!selectedIds.includes(pg.id)) {
          selectedIds = [...selectedIds, pg.id];
          selAnchor = pg.id;
        }
      }, HOLD_MS);
    },
    click(e: MouseEvent, pg: Page, orderedIds: string[]): boolean {
      if (heldSelect) {
        // Lepas-tahan setelah long-press: jangan buka editor.
        heldSelect = false;
        return true;
      }
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        selectMode = true;
        toggleSelect(pg.id);
        return true;
      }
      if (e.shiftKey && selAnchor) {
        e.preventDefault();
        selectMode = true;
        const range = rangeSelect(orderedIds, selAnchor, pg.id);
        selectedIds = [...new Set([...selectedIds, ...range])];
        selAnchor = pg.id;
        return true;
      }
      return false;
    },
    reset() {
      selectedIds = [];
      selAnchor = null;
      selectMode = false;
    },
    setSelectMode(v: boolean) {
      selectMode = v;
    },
    cancelHold() {
      if (holdTimer !== null) {
        clearTimeout(holdTimer);
        holdTimer = null;
      }
    },
    dispose() {
      if (holdTimer !== null) {
        clearTimeout(holdTimer);
        holdTimer = null;
      }
    },
  };
}

/** Urutan tampil untuk shift-range (dipakai komposisi). */
export function orderIds(pages: Page[]): string[] {
  return sortPagesByName(pages).map((p) => p.id);
}
