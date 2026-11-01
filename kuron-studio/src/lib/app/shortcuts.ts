// Shortcut keyboard ala desktop (FE): Cmd di macOS, Ctrl di lain.
// Murni pemetaan tombol -> aksi; guard konteks (editor terbuka,
// fokus input, project/busy) milik pemanggil (App.onViewKey).
export type ShortcutAction = "providers" | "batch" | "results" | "detect" | "import-files";

interface KeyLike {
  key: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

/** Mod + satu tombol, tanpa Alt/Shift. Bukan mod -> null. */
export function matchShortcut(e: KeyLike): ShortcutAction | null {
  if (!(e.metaKey || e.ctrlKey) || e.altKey || e.shiftKey) return null;
  switch (e.key.toLowerCase()) {
    case ",": return "providers";
    case "b": return "batch";
    case "e": return "results";
    case "d": return "detect";
    case "o": return "import-files";
    default: return null;
  }
}
