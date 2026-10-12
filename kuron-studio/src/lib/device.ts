// ponytail: deteksi mobile tanpa dep baru — UA cukup untuk sembunyikan
// kontrol desktop-only (folder picker, update, diagnostics). Upgrade ke
// @tauri-apps/plugin-os saat butuh platform pasti (bukan UA sniffing).
export const isMobile = /iphone|ipad|ipod|android/i.test(
  typeof navigator !== "undefined" ? navigator.userAgent : "",
);
