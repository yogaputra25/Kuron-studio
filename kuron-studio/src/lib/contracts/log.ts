import { invoke as tauriInvoke } from "@tauri-apps/api/core";

/**
 * Bridge invoke + log.
 *
 * Semua panggilan Rust melewati `call()` supaya:
 * - argumen dan hasilnya tercatat sebagai JSON Lines,
 * - error tidak hilang diam-diam di panel (UI juga menampilkannya),
 * - satu tempat untuk mengukur durasi tiap command.
 *
 * Rahasia: `scrub()` membilas field sensitif SEBELUM ditulis ke log. API key
 * hanya boleh hidup di memori, tidak boleh sampai ke file log yang nanti
 * ditempel ke issue tracker.
 */

const SECRET_KEYS = [
  "apiKey",
  "api_key",
  "key",
  "authorization",
  "bearer",
  "password",
  "token",
];

const MAX = 300;

/** Versi objek untuk log — string dipotong, field sensitif dibersihkan. */
export function scrub(value: unknown, depth = 0): unknown {
  if (depth > 4) return "<max-depth>";
  if (value === null || value === undefined) return value;
  if (typeof value === "string") {
    return value.length > MAX ? value.slice(0, MAX) + "…" : value;
  }
  if (typeof value === "number" || typeof value === "boolean") return value;
  if (Array.isArray(value)) {
    // Array besar (daftar model, halaman) cukup direkam panjangnya.
    if (value.length > 8) return `<array len=${value.length}>`;
    return value.map((v) => scrub(v, depth + 1));
  }
  if (typeof value === "object") {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
      if (SECRET_KEYS.some((s) => s.toLowerCase() === k.toLowerCase())) {
        out[k] = v ? "<redacted>" : "<empty>";
      } else {
        out[k] = scrub(v, depth + 1);
      }
    }
    return out;
  }
  return String(value);
}

export interface InvokeMeta {
  command: string;
  args?: Record<string, unknown>;
  elapsedMs: number;
  ok: boolean;
  result?: unknown;
  error?: string;
}

type Listener = (m: InvokeMeta) => void;
const listeners = new Set<Listener>();

/** Panel Diagnostics berlangganan di sini. */
export function onInvoke(fn: Listener): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

function emit(m: InvokeMeta) {
  for (const fn of listeners) {
    try {
      fn(m);
    } catch {
      // Listener yang error tidak boleh membatalkan pemanggil.
    }
  }
}

let sequence = 0;

/** Log lokal, mirror dari Rust. Console devtools juga melihatnya. */
function write(m: InvokeMeta, extra?: Record<string, unknown>) {
  const line = JSON.stringify({ seq: sequence++, ...m, ...extra });
  if (m.ok) console.debug(line);
  else console.error(line);
}

/**
 * `invoke` dengan log. Nama argumen diteruskan apa adanya ke `tauriInvoke`
 * (kunci snake_case), jadi jangan diubah di sini.
 */
export async function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const t0 = performance.now();
  try {
    const value = await tauriInvoke<T>(command, args as never);
    const meta: InvokeMeta = {
      command,
      args: scrub(args) as Record<string, unknown> | undefined,
      elapsedMs: Math.round(performance.now() - t0),
      ok: true,
      result: scrub(value),
    };
    write(meta);
    emit(meta);
    return value;
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    const meta: InvokeMeta = {
      command,
      args: scrub(args) as Record<string, unknown> | undefined,
      elapsedMs: Math.round(performance.now() - t0),
      ok: false,
      error: msg,
    };
    write(meta);
    emit(meta);
    throw e;
  }
}
