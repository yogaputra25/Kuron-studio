// Semua invoke lewat `call` supaya tercatat sebagai JSON Lines.
import { call as invoke } from "./log";
import type {
  AiModelOption,
  AiProviderType,
  BubbleBox,
  BubbleTranslation,
  CleanResult,
  DetectStatus,
  ExportFormat,
  GlossaryEntry,
  ImportResult,
  PageTranslation,
  Project,
  ProviderView,
  QaIssue,
  RetryBubbleInput,
  SaveProviderInput,
  TmHit,
  TranslateBatchInput,
  TranslatePageArgs,
  ValidateResult,
} from "./types";

// NOTE (invoke-contract): two layers, don't mix them.
// Layer 1 — top-level invoke keys: MUST match Rust #[tauri::command] param
// names exactly (snake_case): { project_id, page_id, page_ids, ... }.
// Layer 2 — fields inside `{ input }` structs: camelCase per serde
// `rename_all` (pageId, providerId, targetLang, ...). See
// openspec/changes/fix-import-pages-project-id/specs/invoke-contract/spec.md.
export const api = {
  listProjects: () => invoke<Project[]>("list_projects"),
  createProject: (name: string) => invoke<Project>("create_project", { name }),
  getProject: (project_id: string) => invoke<Project>("get_project", { project_id }),
  importPages: (project_id: string, paths: string[]) =>
    invoke<ImportResult>("import_pages", { project_id, paths }),
  cleanPages: (project_id: string) => invoke<CleanResult>("clean_pages", { project_id }),
  deleteProject: (project_id: string) => invoke<void>("delete_project", { project_id }),
  getImagePreview: (path: string, max_side = 512) =>
    invoke<string>("get_image_preview", { path, max_side }),
  detectStatus: () => invoke<DetectStatus>("detect_status"),
  detectBubbles: (page_id: string) => invoke<BubbleBox[]>("detect_bubbles", { page_id }),
  detectBatch: (page_ids: string[]) =>
    invoke<BubbleBox[][]>("detect_bubbles_batch", { page_ids }),
  saveBubbles: (page_id: string, bubbles: BubbleBox[]) =>
    invoke<BubbleBox[]>("save_bubbles", { page_id, bubbles }),

  listModels: (provider_id: string) =>
    invoke<AiModelOption[]>("list_models", { provider_id }),
  /** Model untuk form yang belum tersimpan (butuh draft, bukan id). */
  listModelsDraft: (input: {
    providerType: AiProviderType;
    baseUrl: string;
    apiKey: string;
  }) =>
    invoke<AiModelOption[]>("list_models_draft", {
      provider_type: input.providerType,
      base_url: input.baseUrl,
      api_key: input.apiKey,
    }),
  validateProvider: (provider_id: string) =>
    invoke<ValidateResult>("validate_provider", { provider_id }),
  saveProvider: (input: SaveProviderInput) =>
    invoke<ProviderView>("save_provider", { input }),
  getProviders: () => invoke<ProviderView[]>("get_providers"),
  deleteProvider: (provider_id: string) =>
    invoke<void>("delete_provider", { provider_id }),
  translatePage: (input: TranslatePageArgs) =>
    invoke<PageTranslation>("translate_page", { input }),
  saveTranslation: (page_id: string, bubbles: BubbleTranslation[]) =>
    invoke<PageTranslation>("save_translation", { page_id, bubbles }),
  clearCache: () => invoke<number>("clear_cache"),

  // --- M3: batch + retry ---
  translateBatch: (input: TranslateBatchInput) =>
    invoke<PageTranslation[]>("translate_batch", { input }),
  retryBubble: (input: RetryBubbleInput) =>
    invoke<PageTranslation>("retry_bubble", { input }),

  // --- M3: glossary (invoke keys snake_case, struct fields camelCase) ---
  glossaryList: () => invoke<GlossaryEntry[]>("glossary_list"),
  glossaryAdd: (source: string, target: string) =>
    invoke<GlossaryEntry>("glossary_add", { source, target }),
  glossaryUpdate: (id: string, source: string, target: string) =>
    invoke<GlossaryEntry>("glossary_update", { id, source, target }),
  glossaryDelete: (id: string) => invoke<void>("glossary_delete", { id }),
  glossaryImportCsv: (csv: string) => invoke<number>("glossary_import_csv", { csv }),
  glossaryExportCsv: () => invoke<string>("glossary_export_csv"),
  glossaryContext: (bubble_texts: string[]) =>
    invoke<string | null>("glossary_context", { bubble_texts }),

  // --- M3: export (path = file utk json/cbz, dir utk png/psd) ---
  exportProject: (project_id: string, format: ExportFormat, path: string) =>
    invoke<string>("export_project", { project_id, format, path }),

  // --- M5: TM + QA + share ---
  tmSearch: (query: string, limit = 10) =>
    invoke<TmHit[]>("tm_search", { query, limit }),
  qaCheck: (project_id: string) => invoke<QaIssue[]>("qa_check", { project_id }),
  shareProject: (project_id: string, path: string) =>
    invoke<string>("share_project", { project_id, path }),

  // --- diagnostics: log JSON Lines + info environment ---
  diagnostics: () => invoke<DiagnosticsInfo>("diagnostics"),
  readLog: (lines = 200) => invoke<string>("read_log", { lines }),
};

/** Mirror Rust `logging::diagnostics` — tidak memuat secret. */
export interface DiagnosticsInfo {
  appVersion: string;
  os: string;
  arch: string;
  /** true kalau file log berhasil dibuka; stderr tetap jalan kalau false. */
  logActive: boolean;
  dataDir: string;
}
