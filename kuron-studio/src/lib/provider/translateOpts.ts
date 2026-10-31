// Satu sumber perakitan opsi translate (FE-6 translate-opts 1.1).
// Tipe + default + 3 konverter murni payload page/batch/retry-bubble.
// Panel hanya memanggil; tak ada rakitan field manual di tempat lain.
// Kontrak wire dipertahankan: key top-level snake_case, field struct
// camelCase; opsional kosong hilang (bukan string kosong).
import type {
  MosaicQuality,
  ReadingDirection,
  RetryBubbleInput,
  TranslateBatchInput,
  TranslatePageArgs,
  TranslateStyle,
} from "../contracts/types";

export interface TranslateOpts {
  targetLang: string;
  style: TranslateStyle;
  skipSfx: boolean;
  mosaicQuality: MosaicQuality;
  customPrompt: string;
  /** Model per-run (sesi-saja); kosong = fallback simpanan provider. */
  model: string;
}

export function defaultTranslateOpts(): TranslateOpts {
  return {
    targetLang: "id",
    style: "standard",
    skipSfx: true,
    mosaicQuality: "low",
    customPrompt: "",
    model: "",
  };
}

export interface TranslateExtra {
  pageId: string;
  providerId: string;
  readingDirection: ReadingDirection;
  /** Glosarium mentah; di-trim, hilang bila kosong (jalur page saja). */
  glossary?: string;
}

/** Payload translate_page: trim glossary/customPrompt/model, kosong hilang. */
export function toTranslateArgs(opts: TranslateOpts, extra: TranslateExtra): TranslatePageArgs {
  return {
    pageId: extra.pageId,
    providerId: extra.providerId,
    targetLang: opts.targetLang,
    style: opts.style,
    skipSfx: opts.skipSfx,
    mosaicQuality: opts.mosaicQuality,
    readingDirection: extra.readingDirection,
    ...(extra.glossary?.trim() ? { glossary: extra.glossary.trim() } : {}),
    ...(opts.customPrompt.trim() ? { customPrompt: opts.customPrompt.trim() } : {}),
    ...(opts.model.trim() ? { model: opts.model.trim() } : {}),
  };
}

export interface BatchExtra {
  pageIds: string[];
  providerId: string;
  readingDirection: ReadingDirection;
}

/** Payload translate_batch: tanpa trim, tanpa glossary (verbatim lama). */
export function toBatchInput(opts: TranslateOpts, extra: BatchExtra): TranslateBatchInput {
  return {
    pageIds: extra.pageIds,
    providerId: extra.providerId,
    targetLang: opts.targetLang,
    style: opts.style,
    skipSfx: opts.skipSfx,
    mosaicQuality: opts.mosaicQuality,
    readingDirection: extra.readingDirection,
    ...(opts.customPrompt ? { customPrompt: opts.customPrompt } : {}),
    ...(opts.model ? { model: opts.model } : {}),
  };
}

export interface RetryBubbleExtra {
  providerId: string;
  readingDirection: ReadingDirection;
  pageId: string;
  bubbleIndex: number;
}

/** Payload retry_bubble: tanpa trim, tanpa glossary (verbatim lama). */
export function toRetryBubbleInput(opts: TranslateOpts, extra: RetryBubbleExtra): RetryBubbleInput {
  return {
    pageId: extra.pageId,
    bubbleIndex: extra.bubbleIndex,
    providerId: extra.providerId,
    targetLang: opts.targetLang,
    style: opts.style,
    skipSfx: opts.skipSfx,
    mosaicQuality: opts.mosaicQuality,
    readingDirection: extra.readingDirection,
    ...(opts.customPrompt ? { customPrompt: opts.customPrompt } : {}),
    ...(opts.model ? { model: opts.model } : {}),
  };
}
