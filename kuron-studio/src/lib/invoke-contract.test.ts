import { describe, expect, it } from "vitest";
import apiSrc from "./api.ts?raw";

// Regression guard for the invoke-contract capability
// (openspec/changes/fix-import-pages-project-id/specs/invoke-contract/spec.md).
//
// Tauri maps invoke args by name to Rust `#[tauri::command]` params, so
// top-level invoke keys MUST be snake_case and identical to the Rust param
// names. Struct payloads inside `{ input }` are camelCase per serde
// `rename_all` and are exempt — only top-level keys are checked here.
//
// No new dependencies and no node: APIs (keeps `pnpm check` green under this
// repo's tsconfig): sources are read via Vite `?raw` imports.
// (ponytail: upgrade the api.ts parse to an AST if computed/dynamic invoke
// keys ever appear — today every call is a string literal.)

// Expected top-level invoke keys per command, mirrors
// kuron-studio/src-tauri/src/commands/*.rs (single source of truth = Rust).
const EXPECTED: Record<string, string[]> = {
  list_projects: [],
  create_project: ["name"],
  get_project: ["project_id"],
  import_pages: ["project_id", "paths"],
  clean_pages: ["project_id"],
  delete_project: ["project_id"],
  get_image_preview: ["path", "max_side"],
  detect_status: [],
  detect_bubbles: ["page_id"],
  detect_bubbles_batch: ["page_ids"],
  save_bubbles: ["page_id", "bubbles"],
  list_models: ["provider_id"],
  list_models_draft: ["provider_type", "base_url", "api_key"],
  validate_provider: ["provider_id"],
  save_provider: ["input"],
  get_providers: [],
  delete_provider: ["provider_id"],
  translate_page: ["input"],
  save_translation: ["page_id", "bubbles"],
  clear_cache: [],
  translate_batch: ["input"],
  retry_bubble: ["input"],
  glossary_list: [],
  glossary_add: ["source", "target"],
  glossary_update: ["id", "source", "target"],
  glossary_delete: ["id"],
  glossary_import_csv: ["csv"],
  glossary_export_csv: [],
  glossary_context: ["bubble_texts"],
  export_project: ["project_id", "format", "path"],
  tm_search: ["query", "limit"],
  qa_check: ["project_id"],
  share_project: ["project_id", "path"],
  diagnostics: [],
  read_log: ["lines"],
};

// Every frontend source as raw text (this file excluded via `!` pattern).
const SOURCES = import.meta.glob<string>(
  ["../**/*.{ts,svelte}", "!../**/*.test.ts", "!./api.ts"],
  { query: "?raw", import: "default", eager: true },
);

// invoke<...>("cmd") or invoke<...>("cmd", { a, b }) — captures cmd + raw args.
const INVOKE_RE = /invoke(?:<[^>]*>)?\(\s*"([a-z_]+)"\s*(?:,\s*\{([^}]*)\})?/g;

function topKeys(rawArgs: string): string[] {
  return rawArgs
    .split(",")
    .map((s: string) => s.trim().split(/[:=]/)[0].trim())
    .filter(Boolean);
}

describe("invoke contract (top-level keys match Rust params)", () => {
  it("api.ts keys match expected snake_case params per command", () => {
    const seen = new Set<string>();
    for (const m of apiSrc.matchAll(INVOKE_RE)) {
      const [cmd, rawArgs] = [m[1], m[2] ?? ""];
      seen.add(cmd);
      const keys = topKeys(rawArgs);
      expect(
        EXPECTED[cmd],
        `unknown command "${cmd}" — add it to EXPECTED with its Rust params`,
      ).toBeDefined();
      expect(keys.sort(), `invoke("${cmd}") keys`).toEqual(
        [...EXPECTED[cmd]].sort(),
      );
      for (const k of keys) {
        expect(
          k,
          `invoke("${cmd}") key "${k}" must be snake_case (use the Rust param name)`,
        ).toMatch(/^[a-z][a-z0-9_]*$/);
      }
    }
    // Every known command must be covered by the bridge (no silent drift).
    expect([...seen].sort()).toEqual(Object.keys(EXPECTED).sort());
  });

  it("no direct Tauri invoke outside the api.ts bridge", () => {
    // `log.ts` adalah bridge yang sah — dial wraps `invoke` untuk logging,
    // dan api.ts mengimpor `call` darinya, bukan langsung dari Tauri.
    const ALLOWED = new Set(["./log.ts"]);
    const bad = Object.entries(SOURCES)
      .filter(([path, src]) =>
        !ALLOWED.has(path) && /from\s+["']@tauri-apps\/api\/core["']/.test(src),
      )
      .map(([path]) => path);
    expect(
      bad,
      `direct Tauri invoke outside lib/api.ts: ${bad.join(", ")}`,
    ).toEqual([]);
  });

  it("api.ts tidak mengimpor invoke langsung dari Tauri", () => {
    expect(apiSrc, "api.ts harus lewat log.call, bukan @tauri-apps/api/core").not.toMatch(
      /from\s+["']@tauri-apps\/api\/core["']/,
    );
  });
});
