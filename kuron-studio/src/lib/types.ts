export type PageStatus =
  | "idle"
  | "detecting"
  | "detected"
  | "noBubbles"
  | "translating"
  | "translated"
  | "failed";

export interface Page {
  id: string;
  path: string;
  width: number;
  height: number;
  status: PageStatus;
  bubbles: BubbleBox[];
  translation?: PageTranslation | null;
}

export interface Project {
  id: string;
  name: string;
  pages: Page[];
}

export interface ImportResult {
  pages: Page[];
  skipped: number;
}

/** Mirror Rust `BubbleBox` — koordinat px original image. */
export interface BubbleBox {
  x: number;
  y: number;
  w: number;
  h: number;
  confidence: number;
  shape: [number, number][] | null;
  kind: "rect" | "ellipse" | "freeform" | null;
  tail: [number, number][] | null;
}

export type Tool = "select" | "rect" | "ellipse" | "freeform" | "tail";

export type ReadingDirection = "rtl" | "ltr";

export interface DetectStatus {
  engine: string;
  modelFound: boolean;
  modelPath: string | null;
}

// --- M2: AI providers + translation (kontrak Rust, invoke keys snake_case) ---
export type AiProviderType =
  | "zen"
  | "openCodeGo"
  | "gemini"
  | "openAi"
  | "openRouter"
  | "metaAi"
  | "clinePass"
  | "cohere"
  | "custom";

export interface ProviderView {
  id: string;
  providerType: AiProviderType;
  name: string;
  baseUrl: string;
  model: string;
  hasKey: boolean;
  isVisionCapable: boolean;
}

export interface SaveProviderInput {
  id?: string;
  providerType: AiProviderType;
  name: string;
  baseUrl: string;
  apiKey?: string;
  model: string;
}

export interface AiModelOption {
  id: string;
  label: string;
  vision: boolean;
}

export interface ValidateResult {
  ok: boolean;
  message: string;
}

export type TranslateStyle =
  | "standard"
  | "formal"
  | "casual"
  | "dramatic"
  | "humorous"
  | "literal"
  | "concise";

export type MosaicQuality = "low" | "high";

export interface BubbleTranslation {
  index: number;
  x: number;
  y: number;
  w: number;
  h: number;
  original: string;
  reading: string;
  translated: string;
  /** Baseline AI terakhir (reset-translation): diisi jalur AI, manual tak sentuh. */
  aiOriginal: string;
  aiReading: string;
  aiTranslated: string;
  needsWhitePatch: boolean;
  isUserEdited: boolean;
}

export interface PageTranslation {
  pageId: string;
  targetLang: string;
  style: string;
  model: string;
  bubbles: BubbleTranslation[];
}

// Struct fields follow Rust serde rename_all camelCase (TranslatePageInput);
// only top-level invoke arg names are snake_case ({ input }).
export interface TranslatePageArgs {
  pageId: string;
  providerId: string;
  targetLang: string;
  style: TranslateStyle;
  skipSfx: boolean;
  mosaicQuality: MosaicQuality;
  readingDirection: ReadingDirection;
  glossary?: string;
}

export const PROVIDER_TYPES: { value: AiProviderType; label: string; needsKey: boolean }[] = [
  { value: "zen", label: "Zen", needsKey: false },
  { value: "openCodeGo", label: "OpenCode Go", needsKey: true },
  { value: "gemini", label: "Gemini", needsKey: true },
  { value: "openAi", label: "OpenAI", needsKey: true },
  { value: "openRouter", label: "OpenRouter", needsKey: true },
  { value: "metaAi", label: "Meta AI", needsKey: true },
  { value: "clinePass", label: "Cline Pass", needsKey: true },
  { value: "cohere", label: "Cohere", needsKey: true },
  { value: "custom", label: "Custom", needsKey: true },
];

// Must match Rust AiProviderType::default_base_url exactly: Rust appends
// endpoint paths (/v1beta/models, /v2/chat, ...) so bases stay origin-level.
export const DEFAULT_BASE_URLS: Record<AiProviderType, string> = {
  zen: "http://localhost:8080",
  openCodeGo: "http://localhost:8080",
  gemini: "https://generativelanguage.googleapis.com",
  openAi: "https://api.openai.com/v1",
  openRouter: "https://openrouter.ai/api/v1",
  metaAi: "https://api.llama.com/v1",
  clinePass: "https://api.cline.bot/v1",
  cohere: "https://api.cohere.ai",
  custom: "",
};

export const TRANSLATE_STYLES: { value: TranslateStyle; label: string }[] = [
  { value: "standard", label: "Standard" },
  { value: "formal", label: "Formal" },
  { value: "casual", label: "Casual" },
  { value: "dramatic", label: "Dramatic" },
  { value: "humorous", label: "Humorous" },
  { value: "literal", label: "Literal" },
  { value: "concise", label: "Concise" },
];

export const TARGET_LANGS: { value: string; label: string }[] = [
  { value: "id", label: "Indonesia" },
  { value: "en", label: "English" },
  { value: "zh", label: "中文" },
  { value: "es", label: "Español" },
  { value: "fr", label: "Français" },
];

// --- M3: batch + glossary + export (kontrak Rust, invoke keys snake_case) ---
export interface GlossaryEntry {
  id: string;
  source: string;
  target: string;
  createdAt: number;
}

export interface BatchOpts {
  providerId: string;
  targetLang: string;
  style: TranslateStyle;
  skipSfx: boolean;
  mosaicQuality: MosaicQuality;
  readingDirection: ReadingDirection;
}

export interface TranslateBatchInput {
  pageIds: string[];
  providerId: string;
  targetLang: string;
  style: TranslateStyle;
  skipSfx: boolean;
  mosaicQuality: MosaicQuality;
  readingDirection: ReadingDirection;
  glossary?: string;
}

export interface RetryBubbleInput {
  pageId: string;
  bubbleIndex: number;
  providerId: string;
  targetLang: string;
  style: TranslateStyle;
  skipSfx: boolean;
  mosaicQuality: MosaicQuality;
  readingDirection: ReadingDirection;
  glossary?: string;
}

export interface TranslateProgress {
  pageId: string;
  done: number;
  total: number;
  ok: boolean;
  message: string;
}

export type ExportFormat = "json" | "png" | "cbz" | "psd";

// --- M5: TM + QA + share (kontrak Rust, serde rename_all camelCase) ---
export interface TmHit {
  projectName: string;
  pageFile: string;
  bubbleIndex: number;
  original: string;
  translated: string;
  score: number;
}

export interface QaIssue {
  pageFile: string;
  bubbleIndex: number;
  /** "untranslated" | "overflow" | "sfxLeak" | "noBubbles" */
  kind: string;
  detail: string;
}
