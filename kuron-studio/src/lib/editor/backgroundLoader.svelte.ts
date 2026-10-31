// Background loader kanvas (FE-5 canvas-redraw-split 1.2).
// Pindah verbatim dari CanvasEditor: tiap request bg harus terminasi ≤15s
// (siap/gagal+retry). Token per-request — onload/onerror/timeout mana duluan
// menang, sisanya basi. Stage/layer/Konva tetap milik panel (via deps) agar
// modul ini murni mesin state + callback, tanpa stage global.
import Konva from "konva";

export interface BackgroundDeps {
  getStage: () => Konva.Stage | null;
  getOverlay: () => Konva.Layer | null;
  getBg: () => Konva.Image | null;
  setBg: (img: Konva.Image) => void;
  scale: () => number;
  imgW: () => number;
  imgH: () => number;
  getFallbackUrl: () => string | undefined;
  redraw: () => void;
}

export const BG_LOAD_TIMEOUT_MS = 15_000;

export function useBackgroundLoader(deps: BackgroundDeps) {
  let bgToken = 0;
  let bgTimer: ReturnType<typeof setTimeout> | null = null;
  let requestedUrl = $state("");
  let imgReady = $state(false);
  let imgError = $state("");
  let pendingUrl: string | null = null;

  function clearTimer() {
    if (bgTimer) {
      clearTimeout(bgTimer);
      bgTimer = null;
    }
  }

  // Retry manual tanpa remount — stage + overlay bubble selamat.
  function retryLoad() {
    if (requestedUrl) loadBackground(requestedUrl);
  }

  function loadBackground(url: string, triedFallback = false) {
    // Panggil setelah stage ada; kalau stage belum dibuat (mount awal), antri.
    if (!deps.getStage() || !deps.getOverlay()) {
      pendingUrl = url;
      return;
    }
    clearTimer();
    requestedUrl = url;
    imgReady = false;
    imgError = "";
    const my = ++bgToken;
    const img = new Image();
    const fail = (msg: string) => {
      if (my !== bgToken || requestedUrl !== url) return;
      // Fallback otomatis sekali ke thumb sebelum menyerah.
      const fb = deps.getFallbackUrl();
      if (!triedFallback && fb && fb !== url) {
        loadBackground(fb, true);
        return;
      }
      clearTimer();
      bgToken++; // invalidasi onload telat — yang menang tetap gagal
      imgError = msg;
    };
    bgTimer = setTimeout(() => {
      fail("Gambar timed out (15 dtk) — preview lambat, coba lagi.");
    }, BG_LOAD_TIMEOUT_MS);
    img.onload = () => {
      // Abaikan load basi (user sudah pindah halaman / retry baru menang) dan
      // unmount (komponen dihancurkan saat load berjalan — stage sudah null).
      if (my !== bgToken || requestedUrl !== url) return;
      const stage = deps.getStage();
      const overlayLayer = deps.getOverlay();
      if (!stage || !overlayLayer) return;
      clearTimer();
      const s = deps.scale();
      const imgW = deps.imgW();
      const imgH = deps.imgH();
      const bgImage = deps.getBg();
      if (bgImage) {
        bgImage.image(img);
        bgImage.size({ width: imgW * s, height: imgH * s });
      } else {
        const bg = new Konva.Layer();
        const bgImg = new Konva.Image({ image: img, width: imgW * s, height: imgH * s, listening: false });
        deps.setBg(bgImg);
        bg.add(bgImg);
        stage.add(bg);
        bg.moveToBottom();
        // Overlay harus tetap di atas background.
        overlayLayer.moveToTop();
      }
      stage.width(imgW * s);
      stage.height(imgH * s);
      imgReady = true;
      overlayLayer.batchDraw();
      deps.redraw();
    };
    img.onerror = () => {
      if (my !== bgToken || requestedUrl !== url) return;
      if (!deps.getStage()) return;
      fail("Gambar gagal dimuat (preview backend gagal).");
    };
    img.src = url;
  }

  /** Effect imageUrl: lewati bila URL sama (guard requestedUrl). */
  function load(url: string) {
    if (!url || requestedUrl === url) return;
    loadBackground(url);
  }

  /** onMount: URL awal langsung, atau konsumsi antrean effect-dini. */
  function consumePending(initialUrl: string | undefined) {
    if (initialUrl) {
      pendingUrl = null;
      loadBackground(initialUrl);
    } else if (pendingUrl) {
      const u = pendingUrl;
      pendingUrl = null;
      loadBackground(u);
    }
  }

  /** onDestroy: invalidasi onload/onerror/timeout telat. */
  function invalidate() {
    bgToken++;
    clearTimer();
  }

  return {
    get requestedUrl() { return requestedUrl; },
    get imgReady() { return imgReady; },
    get imgError() { return imgError; },
    loadBackground,
    retryLoad,
    load,
    consumePending,
    invalidate,
  };
}

export type BackgroundLoader = ReturnType<typeof useBackgroundLoader>;
