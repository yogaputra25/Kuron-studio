// ponytail: deteksi mobile tanpa dep baru — UA cukup untuk menyembunyikan
// kontrol desktop-only (folder picker, update, diagnostics). Upgrade ke
// @tauri-apps/plugin-os saat butuh platform pasti (bukan UA sniffing).
export const isMobile = /iphone|ipad|ipod|android/i.test(
  typeof navigator !== "undefined" ? navigator.userAgent : "",
);

// Batas lebar (px) di bawahnya layout kolom tablet berlaku. Disatukan di sini
// supaya breakpoint CSS (md Tailwind = 768px) dan JS sepakat.
export const TABLET_MAX_W = 768;
const SIDEBAR_W_KEY = "kuron-sidebar-w:v1";

/** Tablet = viewport sempit + pointer kasar. UA-sniff `isMobile` tetap dipakai
 *  untuk gating tombol desktop-only; fungsi ini hanya untuk perilaku sentuh
 *  (snap sheet, hit-area besar). Aman di SSR/vitest: false tanpa window. */
export function isTablet(): boolean {
  if (typeof window === "undefined" || typeof window.matchMedia === "undefined") return false;
  return (
    window.innerWidth < TABLET_MAX_W &&
    window.matchMedia("(pointer: coarse)").matches
  );
}

/** Viewport sempit tanpa peduli pointer — untuk susunan kolom CSS-independen.
 *  Window desktop 700px ikut kolom walau `pointer: fine`. */
export function isNarrow(): boolean {
  if (typeof window === "undefined") return false;
  return window.innerWidth < TABLET_MAX_W;
}

/** Langganan perubahan viewport (resize + rotasi). Kembalikan unwatch untuk
 *  cleanup di onDestroy/$effect. Listener pasif, tanpa ResizeObserver
 *  (ponytail itu tetap berlaku — ini matchMedia, bukan observe elemen). */
export function watchViewport(cb: () => void): () => void {
  if (typeof window === "undefined") return () => {};
  window.addEventListener("resize", cb);
  window.addEventListener("orientationchange", cb);
  return () => {
    window.removeEventListener("resize", cb);
    window.removeEventListener("orientationchange", cb);
  };
}

/** Lebar sidebar tersimpan (px), sudah di-clamp. Tak ada/invalid → null
 *  (pemanggil pakai default). try/catch: private mode tanpa storage. */
export function loadSidebarW(min: number, max: number, fallback: number): number {
  try {
    if (typeof localStorage === "undefined") return fallback;
    const raw = localStorage.getItem(SIDEBAR_W_KEY);
    if (raw === null) return fallback;
    const v = Math.round(Number(raw));
    if (!Number.isFinite(v)) return fallback;
    return Math.min(max, Math.max(min, v));
  } catch {
    return fallback;
  }
}

export function saveSidebarW(w: number): void {
  try {
    localStorage.setItem(SIDEBAR_W_KEY, String(Math.round(w)));
  } catch {
    /* private mode — lebar sesi ini saja */
  }
}
