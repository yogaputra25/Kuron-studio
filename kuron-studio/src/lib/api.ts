import { invoke } from "@tauri-apps/api/core";
import type {
  AiModelOption,
  BubbleBox,
  BubbleTranslation,
  DetectStatus,
  GlossaryEntry,
  ImportResult,
  PageTranslation,
  Project,
  ProviderView,
  RetryBubbleInput,
  SaveProviderInput,
  TranslateBatchInput,
  TranslatePageArgs,
  ValidateResult,
} from "./types";

// NOTE: invoke keys must match Rust param names exactly (snake_case).
export const api = {
  listProjects: () => invoke<Project[]>("list_projects"),
  createProject: (name: string) => invoke<Project>("create_project", { name }),
  getProject: (project_id: string) => invoke<Project>("get_project", { project_id }),
  importPages: (project_id: string, paths: string[]) =>
    invoke<ImportResult>("import_pages", { project_id, paths }),
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

  // --- M3: export (path = file utk json/cbz, dir utk png) ---
  exportProject: (project_id: string, format: "json" | "png" | "cbz", path: string) =>
    invoke<string>("export_project", { project_id, format, path }),
};
