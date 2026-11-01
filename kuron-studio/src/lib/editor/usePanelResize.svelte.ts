// Resize panel + mode tablet (FE-4, task 3.1; logika panel-resize-tablet D1-D6).
// Runes-module pola FE-2: lebar sidebar px, collapse (tak dipersist),
// drag-lokal-commit agar Konva redraw sekali saat lepas.
// narrow (<768px) → kolom; tablet (+coarse) → sheet.
// Persist `kuron-sidebar-w:v1` tetap lewat app/device (FE-8).
import { isNarrow, isTablet, loadSidebarW, saveSidebarW, watchViewport } from "../app/device";
import {
  COLLAPSE_THRESHOLD,
  SIDEBAR_DEFAULT,
  SIDEBAR_MIN,
  SNAP_DEFAULT,
  clampWidth,
  nearestSnap,
  sheetHeightPct,
  sidebarMax,
} from "./panel";

export interface PanelResizeDeps {
  /** Kontainer body (di-bind panel setelah mount). */
  getBody: () => HTMLDivElement | null;
  /** Redraw kedua stage setelah layout settle. */
  onLayoutSettled: () => void;
}

export function usePanelResize(deps: PanelResizeDeps) {
  let sidebarW = $state(SIDEBAR_DEFAULT);
  let collapsed = $state(false);
  let dragW = $state<number | null>(null);
  let dragH = $state<number | null>(null);
  let narrow = $state(false);
  let tablet = $state(false);
  let snap = $state(SNAP_DEFAULT);
  let unwatch: (() => void) | null = null;

  const columnMode = $derived(narrow);
  /** Lebar tampil = drag saat geser, commit setelah lepas. */
  const displayW = $derived(dragW ?? sidebarW);
  /** Tinggi panel bawah (% kontainer) saat kolom: drag atau snap terkunci. */
  const panelHPct = $derived(sheetHeightPct(dragH ?? snap));

  /** Lebar kontainer body (px) untuk clamp dinamis MAX = bodyW - lantai canvas. */
  function bodyWidth(): number {
    const r = deps.getBody()?.getBoundingClientRect();
    if (r && r.width > 0) return r.width;
    if (typeof window !== "undefined" && window.innerWidth > 0) return window.innerWidth;
    return 0;
  }

  function syncViewport() {
    narrow = isNarrow();
    tablet = isTablet();
  }

  /** Init sekali (ganti onMount lama): baca persist + dengar viewport. */
  function mount(viewportW: number) {
    sidebarW = loadSidebarW(SIDEBAR_MIN, sidebarMax(viewportW || undefined), SIDEBAR_DEFAULT);
    syncViewport();
    unwatch?.();
    unwatch = watchViewport(syncViewport);
  }

  function dispose() {
    unwatch?.();
    unwatch = null;
  }

  function down(e: PointerEvent) {
    const body = deps.getBody();
    if (collapsed || !body) return;
    // Drag sash tak boleh menyeleksi teks (rasa Ctrl+A): kunci selection
    // selama drag, lepas lagi di up (termasuk pointercancel).
    e.preventDefault();
    if (typeof document !== "undefined") document.body.style.userSelect = "none";
    (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
    const r = body.getBoundingClientRect();
    if (columnMode) {
      dragH = ((r.bottom - e.clientY) / r.height) * 100;
    } else {
      dragW = r.right - e.clientX;
    }
  }

  function move(e: PointerEvent) {
    const body = deps.getBody();
    if (!body) return;
    if (dragW !== null && !columnMode) {
      const r = body.getBoundingClientRect();
      dragW = r.right - e.clientX;
    } else if (dragH !== null && columnMode) {
      const r = body.getBoundingClientRect();
      dragH = Math.min(100, Math.max(0, ((r.bottom - e.clientY) / r.height) * 100));
    }
  }

  function up() {
    if (typeof document !== "undefined") document.body.style.userSelect = "";
    if (dragW !== null) {
      if (dragW < COLLAPSE_THRESHOLD) {
        collapsed = true;
      } else {
        sidebarW = clampWidth(dragW, bodyWidth() || undefined);
        saveSidebarW(sidebarW);
      }
      dragW = null;
      deps.onLayoutSettled();
    }
    if (dragH !== null) {
      snap = nearestSnap(dragH);
      dragH = null;
      deps.onLayoutSettled();
    }
  }

  /** Keyboard divider: row = ←/→ 8px langsung commit (mentok MIN/MAX, tak collapse); kolom = ↑/↓ pindah snap. */
  function key(e: KeyboardEvent) {
    if (collapsed) return;
    if (!columnMode && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
      e.preventDefault();
      sidebarW = clampWidth(sidebarW + (e.key === "ArrowLeft" ? -8 : 8), bodyWidth() || undefined);
      saveSidebarW(sidebarW);
      deps.onLayoutSettled();
    } else if (columnMode && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      e.preventDefault();
      const order = [40, 70, 100];
      const i = order.indexOf(snap);
      const ni = Math.min(order.length - 1, Math.max(0, i + (e.key === "ArrowUp" ? 1 : -1)));
      snap = order[ni];
      deps.onLayoutSettled();
    }
  }

  function setSnap(v: number) {
    snap = v;
  }

  function setCollapsed(v: boolean) {
    collapsed = v;
  }

  function toggleCollapsed() {
    collapsed = !collapsed;
  }

  return {
    get sidebarW() { return sidebarW; },
    get collapsed() { return collapsed; },
    get narrow() { return narrow; },
    get tablet() { return tablet; },
    get snap() { return snap; },
    get columnMode() { return columnMode; },
    get displayW() { return displayW; },
    get panelHPct() { return panelHPct; },
    mount,
    dispose,
    syncViewport,
    down,
    move,
    up,
    key,
    setSnap,
    setCollapsed,
    toggleCollapsed,
  };
}

export type PanelResize = ReturnType<typeof usePanelResize>;
