import type { TextLayout } from "../editor/text/textLayout";
import type { TranslateOpts } from "../provider/translateOpts";
import extensions from "./extensions.json";
import providers from "./providers.json";

export type { TextLayout };

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

export interface ChapterFolder {
  id: string;
  name: string;
  pages: Page[];
  createdAt?: number;
  /** Parent folder; null/undefined = root. */
  parentId?: string | null;
}

export interface Project {
  id: string;
  name: string;
  /** Folder/chapter. Mode A (simple) = 1 folder `default`. */
  folders: ChapterFolder[];
  /** Epoch seconds terakhir dimodifikasi. */
  updatedAt?: number;
  /** Kompat baca: projects.json lama flat. Jangan dipakai kode baru. */
  pages?: Page[];
}

// --- M0: import contract (values, bukan type) ---
// SATU sumber kebenaran = `extensions.json` di folder ini (diimpor langsung,
// bukan disalin). Backend (`commands/project.rs`) membaca file yang sama via
// `include_str!`; jangan tambah ekstensi di satu sisi saja — kalau formatnya
// tidak bisa di-decode, import diam-diam jadi halaman rusak.
export const IMAGE_EXTS: readonly string[] = extensions.image;

export const ARCHIVE_EXTS: readonly string[] = extensions.archive;
// pdf-import: dikenali backend via is_pdf() terpisah (output = jpg/png,
// jadi TAK masuk IMAGE_EXTS). Sinkron via `extensions.json`.
export const PDF_EXTS: readonly string[] = extensions.pdf;

export interface ImportResult {
  pages: Page[];
  skipped: number;
  folderId: string;
}

/** Mirror Rust `MoveResult` — hasil `move_pages`. */
export interface MoveResult {
  moved: number;
  skipped: number;
}

/** Mirror Rust `CleanResult` — hasil `clean_pages`. */
export interface CleanResult {
  /** Halaman yang dibuang (file hilang / tidak bisa di-decode). */
  removed: number;
  /** Halaman yang masih tertinggal setelah bersih. */
  kept: number;
  /** Berapa file fisik yang ikut terhapus (hanya yang ada di store dir). */
  removedFiles: number;
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
  | "gemini"
  | "openAi"
  | "metaAi"
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
  /** Polygon freeform absolut (px asli); null = rect/ellipse. Repair R4. */
  shape?: [number, number][] | null;
  /** Layout teks final dari kanvas (export-text-parity). Ukuran + baris sudah
   *  diputuskan di sana; export menggambarnya apa adanya. Absen = file lama. */
  layout?: TextLayout | null;
  /** Ukuran font manual (px HALAMAN). Absen/null = auto-fit oleh kanvas. */
  fontSize?: number | null;
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
  /** Tandai-selesai manual (backend `manualDone`): Sudah walau sebagian kosong. Absen = false (data lama). */
  manualDone?: boolean;
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
  customPrompt?: string;
  model?: string;
}

export const PROVIDER_TYPES: { value: AiProviderType; label: string; needsKey: boolean }[] = [
  { value: "gemini", label: "Gemini", needsKey: true },
  { value: "openAi", label: "OpenAI", needsKey: true },
  { value: "metaAi", label: "Meta AI", needsKey: true },
  { value: "cohere", label: "Cohere", needsKey: true },
  { value: "custom", label: "Custom", needsKey: true },
];

// SATU sumber kebenaran = `providers.json` (diimpor langsung). Backend
// (`AiProviderType::default_base_url`) membaca file yang sama; Rust menambah
// endpoint path (/v1beta/models, /v2/chat, ...) jadi base tetap origin-level.
export const DEFAULT_BASE_URLS: Record<AiProviderType, string> = { ...providers };

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

/** Satu bentuk dengan TranslateOpts + identitas batch (FE-6 translate-opts). */
export type BatchOpts = TranslateOpts & {
  providerId: string;
  readingDirection: ReadingDirection;
};

export interface TranslateBatchInput {
  pageIds: string[];
  providerId: string;
  targetLang: string;
  style: TranslateStyle;
  skipSfx: boolean;
  mosaicQuality: MosaicQuality;
  readingDirection: ReadingDirection;
  glossary?: string;
  customPrompt?: string;
  model?: string;
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
  customPrompt?: string;
  model?: string;
}

export interface TranslateProgress {
  pageId: string;
  done: number;
  total: number;
  ok: boolean;
  message: string;
}

/// Event `detect_progress` dari Rust (`detect.rs`, camelCase): per halaman
/// selesai, termasuk yang gagal (ok=false + message) agar batch bisa lanjut.

export interface DetectProgress {
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
