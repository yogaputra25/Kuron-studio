// Unit konverter opsi translate (FE-6 translate-opts 1.2).
// Mengunci key-by-key: daftar key eksak, casing preserved (snake_case
// top-level / camelCase field), opsional kosong hilang (bukan string kosong).
import { describe, expect, it } from "vitest";
import {
  defaultTranslateOpts,
  toBatchInput,
  toRetryBubbleInput,
  toTranslateArgs,
  type TranslateOpts,
} from "./translateOpts";

const base: TranslateOpts = {
  targetLang: "en",
  style: "formal",
  skipSfx: false,
  mosaicQuality: "high",
  customPrompt: "Sapa santai",
  model: "gemini-2.0",
};

describe("defaultTranslateOpts", () => {
  it("default = id/standard/skip-low/kosong", () => {
    expect(defaultTranslateOpts()).toEqual({
      targetLang: "id",
      style: "standard",
      skipSfx: true,
      mosaicQuality: "low",
      customPrompt: "",
      model: "",
    });
  });
});

describe("toTranslateArgs", () => {
  it("daftar key eksak + casing preserved", () => {
    const out = toTranslateArgs(base, { pageId: "p1", providerId: "g", readingDirection: "ltr", glossary: "kamu=you" });
    expect(Object.keys(out).sort()).toEqual(
      ["customPrompt", "glossary", "model", "mosaicQuality", "pageId", "providerId", "readingDirection", "skipSfx", "style", "targetLang"].sort(),
    );
    expect(out).toMatchObject({ pageId: "p1", providerId: "g", targetLang: "en", style: "formal", glossary: "kamu=you" });
  });

  it("trim glossary/customPrompt/model; kosong hilang", () => {
    const out = toTranslateArgs(
      { ...base, customPrompt: "  ", model: "" },
      { pageId: "p1", providerId: "g", readingDirection: "ltr", glossary: "  a=b  " },
    );
    expect(out).not.toHaveProperty("customPrompt");
    expect(out).not.toHaveProperty("model");
    expect(out.glossary).toBe("a=b");
    const bare = toTranslateArgs({ ...base, customPrompt: "", model: "" }, { pageId: "p1", providerId: "g", readingDirection: "ltr" });
    expect(bare).not.toHaveProperty("glossary");
    expect(bare).not.toHaveProperty("customPrompt");
    expect(bare).not.toHaveProperty("model");
  });

  it("snapshot teks payload (kontrak wire)", () => {
    const out = toTranslateArgs(base, { pageId: "p1", providerId: "g", readingDirection: "ltr" });
    expect(JSON.stringify(out)).toBe(
      '{"pageId":"p1","providerId":"g","targetLang":"en","style":"formal","skipSfx":false,"mosaicQuality":"high","readingDirection":"ltr","customPrompt":"Sapa santai","model":"gemini-2.0"}',
    );
  });
});

describe("toBatchInput", () => {
  it("daftar key eksak tanpa trim tanpa glossary", () => {
    const out = toBatchInput(base, { pageIds: ["p1", "p2"], providerId: "g", readingDirection: "rtl" });
    expect(Object.keys(out).sort()).toEqual(
      ["customPrompt", "model", "mosaicQuality", "pageIds", "providerId", "readingDirection", "skipSfx", "style", "targetLang"].sort(),
    );
    expect(out).not.toHaveProperty("glossary");
    // Verbatim lama: tanpa trim.
    const spaced = toBatchInput({ ...base, customPrompt: "  x  " }, { pageIds: [], providerId: "g", readingDirection: "ltr" });
    expect(spaced.customPrompt).toBe("  x  ");
  });

  it("opsional kosong hilang", () => {
    const out = toBatchInput({ ...base, customPrompt: "", model: "" }, { pageIds: [], providerId: "g", readingDirection: "ltr" });
    expect(out).not.toHaveProperty("customPrompt");
    expect(out).not.toHaveProperty("model");
  });
});

describe("toRetryBubbleInput", () => {
  it("daftar key eksak + bubbleIndex", () => {
    const out = toRetryBubbleInput(base, { pageId: "p1", bubbleIndex: 3, providerId: "g", readingDirection: "ltr" });
    expect(Object.keys(out).sort()).toEqual(
      ["bubbleIndex", "customPrompt", "model", "mosaicQuality", "pageId", "providerId", "readingDirection", "skipSfx", "style", "targetLang"].sort(),
    );
    expect(out.bubbleIndex).toBe(3);
  });
});
