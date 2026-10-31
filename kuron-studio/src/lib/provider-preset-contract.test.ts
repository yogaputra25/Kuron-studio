import { describe, expect, it } from "vitest";
import configRs from "../../src-tauri/src/provider/config.rs?raw";
import { DEFAULT_BASE_URLS, PROVIDER_TYPES } from "./contracts/types";

// Preset base URL duplikat di Rust (AiProviderType::default_base_url) dan
// TS (DEFAULT_BASE_URLS). Kalau menyimpang, user dapat endpoint beda
// tergantung jalur (listing vs effective_base_url). Rust sumber kebenaran.
function rustDefaults(): Record<string, string> {
  const fnBody = /fn default_base_url\(&self\) -> &'static str \{([\s\S]*?)\n    \}/.exec(configRs);
  expect(fnBody, "default_base_url tidak ketemu di config.rs").not.toBeNull();
  const out: Record<string, string> = {};
  for (const m of fnBody![1].matchAll(/Self::(\w+) => "([^"]*)"/g)) {
    const key = m[1].charAt(0).toLowerCase() + m[1].slice(1);
    out[key] = m[2];
  }
  return out;
}

describe("provider preset contract (TS == Rust)", () => {
  it("base URL per tipe sama di kedua sisi", () => {
    expect({ ...DEFAULT_BASE_URLS }).toEqual(rustDefaults());
  });

  it("daftar dropdown == kunci preset, tanpa tipe hapus", () => {
    expect(PROVIDER_TYPES.map((p) => p.value).sort()).toEqual(
      Object.keys(DEFAULT_BASE_URLS).sort(),
    );
    for (const dead of ["Zen", "OpenCodeGo", "OpenRouter", "ClinePass"]) {
      expect(configRs).not.toMatch(new RegExp(`Self::${dead}\\b`));
    }
    // Satu-satunya tempat nama hapus boleh muncul: arm migrasi deserializer.
    expect(configRs).toContain('"zen" | "openCodeGo" | "openRouter" | "clinePass"');
  });

  it("preset Meta memakai api.meta.ai", () => {
    expect(DEFAULT_BASE_URLS.metaAi).toBe("https://api.meta.ai/v1");
  });
});
