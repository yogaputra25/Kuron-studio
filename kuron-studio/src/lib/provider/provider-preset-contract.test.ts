import { describe, expect, it } from "vitest";
import { DEFAULT_BASE_URLS, PROVIDER_TYPES } from "../contracts/types";
import providers from "../contracts/providers.json";

// Kontrak lintas bahasa, sumber = `providers.json` — TANPA membaca source
// Rust (tidak ada `?raw` ke `.rs`, tidak ada path `src-tauri/`):
// - TS (`types.ts`) mengimpor JSON langsung (bukan salinan),
// - Rust (`AiProviderType::default_base_url`) membaca JSON yang sama via
//   `include_str!` + `OnceLock` (dipin test Rust
//   `default_base_url_matches_json_contract`; kompat nama preset lama dipin
//   `removed_preset_names_still_deserialize_to_custom`),
// - test ini memastikan pemakaian TS tetap sinkron dengan JSON.

describe("provider preset contract (TS == JSON)", () => {
  it("base URL per tipe sama dengan JSON", () => {
    expect({ ...DEFAULT_BASE_URLS }).toEqual({ ...providers });
  });

  it("daftar dropdown == kunci preset, tanpa tipe hapus", () => {
    expect(PROVIDER_TYPES.map((p) => p.value).sort()).toEqual(
      Object.keys(providers).sort(),
    );
    for (const dead of ["zen", "openCodeGo", "openRouter", "clinePass"]) {
      expect(Object.keys(providers)).not.toContain(dead);
    }
  });

  it("preset Meta memakai api.meta.ai", () => {
    expect(DEFAULT_BASE_URLS.metaAi).toBe("https://api.meta.ai/v1");
  });
});
