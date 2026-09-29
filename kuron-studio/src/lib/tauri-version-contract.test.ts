import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

// `tauri build` menolak jalan kalau paket npm dan crate Rust tidak satu
// major/minor:
//
//     Found version mismatched Tauri packages.
//     tauri (v2.12.0) : @tauri-apps/api (v2.11.1)
//
// Penyebabnya: Cargo.toml memakai `version = "2"`, jadi cargo bebas mengambil
// minor terbaru begitu Cargo.lock diregenerasi, sementara package.json diam di
// tempat. Kegagalan ini baru muncul di job `build` — setelah `frontend` +
// `rust` jalan ±4 menit — jadi mahal untuk ditemukan lagi. Test ini menjaga
// kedua sisi selaras, dan memaksa pin eksak supaya `pnpm update` tidak bisa
// menggeser salah satu sisi sendirian.

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const REPO = join(ROOT, "..");

/** Pasangan crate Rust ↔ paket npm yang dibandingkan oleh CLI Tauri. */
const PAIRS: [rust: string, npm: string][] = [
  ["tauri", "@tauri-apps/api"],
  ["tauri-plugin-dialog", "@tauri-apps/plugin-dialog"],
  ["tauri-plugin-process", "@tauri-apps/plugin-process"],
  ["tauri-plugin-updater", "@tauri-apps/plugin-updater"],
];

function cargoLock(): Map<string, string> {
  const raw = readFileSync(join(ROOT, "..", "src-tauri", "Cargo.lock"), "utf8");
  const out = new Map<string, string>();
  for (const block of raw.split("[[package]]").slice(1)) {
    const name = block.match(/^\s*name = "([^"]+)"/m)?.[1];
    const version = block.match(/^\s*version = "([^"]+)"/m)?.[1];
    // Ambil yang pertama saja: cukup untuk crate tanpa multiple-versions.
    if (name && version && !out.has(name)) out.set(name, version);
  }
  return out;
}

function packageJson(): Record<string, string> {
  return JSON.parse(readFileSync(join(REPO, "package.json"), "utf8")).dependencies;
}

/** pnpm-lock memakai kunci `'nama@versi':` — bukti versi itulah yang terpasang. */
function lockedVersions(): Set<string> {
  const raw = readFileSync(join(REPO, "pnpm-lock.yaml"), "utf8");
  return new Set([...raw.matchAll(/^\s+'([^']+@[\d.]+)':/gm)].map((m) => m[1]));
}

const majorMinor = (v: string) => v.split(".").slice(0, 2).join(".");

describe("kontrak versi Tauri: npm ↔ Rust", () => {
  const crates = cargoLock();
  const deps = packageJson();
  const locked = lockedVersions();

  it.each(PAIRS)("%s ↔ %s satu major/minor", (rust, npm) => {
    const rustV = crates.get(rust);
    const npmV = deps[npm];
    expect(rustV, `crate ${rust} tidak ada di Cargo.lock`).toBeTruthy();
    expect(npmV, `paket ${npm} tidak ada di package.json`).toBeTruthy();
    expect(
      majorMinor(npmV),
      `${npm} (${npmV}) harus selaras dengan ${rust} (${rustV}) — ` +
        "tauri build menolak kalau major/minor beda",
    ).toBe(majorMinor(rustV!));
  });

  it.each(PAIRS)("%s ↔ %s dipin eksak (tanpa ^ ~ *)", (_rust, npm) => {
    // Rentang seperti "^2.11.1" mengizinkan pnpm naik sendiri tanpa Cargo.lock
    // ikut — persis cara mismatch di CI terjadi.
    expect(
      deps[npm],
      `${npm} harus dipin eksak supaya tidak bisa digeser terpisah dari Cargo.lock`,
    ).toMatch(/^\d+\.\d+\.\d+$/);
  });

  it("versi yang dipin benar-benar ada di pnpm-lock", () => {
    for (const [rust, npm] of PAIRS) {
      expect(
        locked.has(`${npm}@${deps[npm]}`),
        `${npm}@${deps[npm]} tidak terkunci di pnpm-lock.yaml — jalankan pnpm install`,
      ).toBe(true);
      expect(crates.get(rust)).toBeTruthy();
    }
  });
});
