// Persistensi localStorage berversi + berskema (FE-8 app-storage 1.1).
// Satu-satunya modul yang menyentuh `localStorage` langsung.
// Semantik: hilang/rusak/versi-beda/tipe-salah/melempar → fallback,
// tak pernah throw, tak pernah memakai bentuk salah diam-diam.
// Codec per key menjaga byte-identik dengan perilaku lama
// (string mentah vs angka, bukan seragam JSON).
export interface Codec<T> {
  parse: (raw: string) => T | undefined;
  serialize: (value: T) => string;
}

const rawString: Codec<string> = {
  parse: (raw) => raw,
  serialize: (v) => v,
};

const num: Codec<number> = {
  parse: (raw: string) => {
    const n = Number(raw);
    return Number.isFinite(n) ? n : undefined;
  },
  serialize: (v) => String(v),
};

export interface StorageKey<T> {
  /** Key berversi aktif. */
  key: string;
  /** Key lama tanpa versi (sekali-migrasi, bila ada). */
  legacy?: string;
  codec: Codec<T>;
}

function defineKey<T>(key: string, codec: Codec<T>, legacy?: string): StorageKey<T> {
  return legacy === undefined ? { key, codec } : { key, legacy, codec };
}

export const STORAGE_KEYS = {
  pageView: defineKey("kuron-page-view:v1", rawString, "kuron-page-view"),
  lang: defineKey("kuron-lang:v1", rawString, "kuron-lang"),
  theme: defineKey("kuron-theme:v1", rawString, "kuron-theme"),
  sidebarW: defineKey("kuron-sidebar-w:v1", num),
} as const;

function getItem(key: string): string | null {
  if (typeof localStorage === "undefined") return null;
  return localStorage.getItem(key);
}

/**
 * Baca key berversi. Urutan: key aktif → migrasi-sekali dari legacy →
 * fallback. Nilai rusak (parse gagal / guard tolak) dibuang (key dihapus)
 * lalu fallback. `localStorage` yang melempar = hilang.
 */
export function read<T>(def: StorageKey<T>, fallback: T, guard?: (value: T) => boolean): T {
  const ok = (v: T): boolean => (guard === undefined ? true : guard(v));
  try {
    const raw = getItem(def.key);
    if (raw !== null) {
      const v = def.codec.parse(raw);
      if (v !== undefined && ok(v)) return v;
      try { localStorage.removeItem(def.key); } catch { /* abaikan */ }
      return fallback;
    }
    // Migrasi sekali: tulis-baru → verifikasi baca-balik → hapus lama.
    if (def.legacy !== undefined) {
      const old = getItem(def.legacy);
      if (old !== null) {
        const v = def.codec.parse(old);
        if (v !== undefined && ok(v)) {
          try {
            localStorage.setItem(def.key, def.codec.serialize(v));
            if (getItem(def.key) === def.codec.serialize(v)) {
              try { localStorage.removeItem(def.legacy); } catch { /* abaikan */ }
            }
          } catch { /* storage penuh: key lama utuh, coba lagi lain waktu */ }
          return v;
        }
      }
    }
    return fallback;
  } catch {
    return fallback;
  }
}

/** Tulis key berversi; melempar-tidak (private mode = no-op diam). */
export function write<T>(def: StorageKey<T>, value: T): void {
  try {
    localStorage.setItem(def.key, def.codec.serialize(value));
  } catch {
    /* private mode: sesi ini saja */
  }
}

/** Hapus key berversi (dan legacy-nya bila ada). */
export function remove<T>(def: StorageKey<T>): void {
  try {
    localStorage.removeItem(def.key);
    if (def.legacy !== undefined) localStorage.removeItem(def.legacy);
  } catch {
    /* abaikan */
  }
}
