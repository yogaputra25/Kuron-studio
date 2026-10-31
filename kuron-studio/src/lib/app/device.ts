import { read, STORAGE_KEYS, write } from "./storage";

// ponytail: deteksi mobile tanpa dep baru — UA cukup untuk menyembunyikan
// kontrol desktop-only (folder picker, update, diagnostics). Upgrade ke
// @tauri-apps/plugin-os saat butuh platform pasti (bukan UA sniffing).
export const isMobile = /iphone|ipad|ipod|android/i.test(
  typeof navigator !== "undefined" ? navigator.userAgent : "",
);

// Batas lebar (px) di bawahnya layout kolom tablet berlaku. Disatukan di sini
// supaya breakpoint CSS (md Tailwind = 768px) dan JS sepakat.
export const TABLET_MAX_W = 768;

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

/** Lebar sidebar tersimpan (px), sudah di-clamp. Tak ada/invalid → fallback
 *  mentah (kompatibel lama: clamp hanya untuk nilai valid dari storage).
 *  Parse + try/catch pindah ke app/storage (FE-8 2.2). */
export function loadSidebarW(min: number, max: number, fallback: number): number {
  // NaN = penanda "tak ada/invalid" agar fallback kembali mentah.
  const v = read(STORAGE_KEYS.sidebarW, Number.NaN);
  if (!Number.isFinite(v)) return fallback;
  return Math.min(max, Math.max(min, Math.round(v)));
}

export function saveSidebarW(w: number): void {
  write(STORAGE_KEYS.sidebarW, Math.round(w));
}
