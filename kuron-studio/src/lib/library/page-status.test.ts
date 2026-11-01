// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import { statusAfterBubbleSave, statusAfterTranslationSave, translationComplete } from "./status";
import TranslationCard from "../editor/TranslationCard.svelte";
import { api } from "../contracts/api";

describe("translationComplete", () => {
  it("lengkap bila semua BERISI terisi", () => {
    expect(translationComplete([{ original: "a", translated: "A" }, { original: "b", translated: "B" }])).toBe(true);
  });

  it("sebagian / kosong / blank tak lengkap", () => {
    expect(translationComplete([{ original: "a", translated: "A" }, { original: "b", translated: "" }])).toBe(false);
    expect(translationComplete([{ original: "a", translated: "  " }])).toBe(false);
    expect(translationComplete([])).toBe(false);
  });

  it("original kosong diabaikan (kepotong/tak terbaca)", () => {
    expect(translationComplete([{ original: "a", translated: "A" }, { original: "", translated: "" }, { original: "  ", translated: "" }])).toBe(true);
    expect(translationComplete([{ original: "", translated: "" }])).toBe(false);
    expect(translationComplete([{ original: "", translated: "x" }])).toBe(false);
  });

  it("flag manual menang", () => {
    expect(translationComplete([{ original: "a", translated: "A" }, { original: "b", translated: "" }], true)).toBe(true);
  });
});

describe("statusAfterBubbleSave", () => {
  it("translated bertahan walau geometri disimpan ulang", () => {
    expect(statusAfterBubbleSave("translated", true, 3)).toBe("translated");
    expect(statusAfterBubbleSave("translating", true, 3)).toBe("translating");
  });

  it("tanpa terjemahan ikut jumlah bubble", () => {
    expect(statusAfterBubbleSave("idle", false, 2)).toBe("detected");
    expect(statusAfterBubbleSave("idle", false, 0)).toBe("noBubbles");
  });
});

describe("statusAfterTranslationSave", () => {
  it("lengkap jadi translated", () => {
    expect(statusAfterTranslationSave("detected", [{ original: "o", translated: "x" }], false)).toBe("translated");
  });

  it("sebagian mempertahankan status lama", () => {
    expect(statusAfterTranslationSave("detected", [{ original: "o", translated: "x" }, { original: "o", translated: "" }])).toBe("detected");
    expect(statusAfterTranslationSave("idle", [])).toBe("idle");
  });

  it("transient translating jatuh ke detected", () => {
    expect(statusAfterTranslationSave("translating", [{ original: "o", translated: "" }])).toBe("detected");
  });
});

describe("page-status UI behavior", () => {
  it("TranslationCard renders mark done button when showMarkDone is true and invokes callback", async () => {
    const onMarkDone = vi.fn();
    const { getByRole } = renderWithI18n(TranslationCard, {
      props: {
        hasTranslation: true,
        card: {
          b: {
            index: 0,
            x: 0,
            y: 0,
            w: 10,
            h: 10,
            original: "orig",
            reading: "",
            translated: "",
            aiOriginal: "orig",
            aiReading: "",
            aiTranslated: "",
            needsWhitePatch: false,
            isUserEdited: false,
          },
          i: 0,
        },
        mode: "split",
        trgLabel: "Target",
        draft: null,
        dirtyDraft: false,
        glossMsg: "",
        savingTr: false,
        showMarkDone: true,
        hasBubbles: true,
        geomBox: null,
        geomIdx: 0,
        geomOutOfSync: false,
        onSelectBubble: vi.fn(),
        onResetBubble: vi.fn(),
        onDraftInput: vi.fn(),
        onCompStart: vi.fn(),
        onCompEnd: vi.fn(),
        onCommit: vi.fn(),
        onPressStart: vi.fn(),
        onPressEnd: vi.fn(),
        onMarkDone,
        onSaveAll: vi.fn(),
        onSetGeometry: vi.fn(),
        onScaleBubble: vi.fn(),
        onSetFontSize: vi.fn(),
        onCreateEmpty: vi.fn(),
      },
    });

    const markDoneBtn = getByRole("button", { name: /Tandai selesai|Mark done/i });
    expect(markDoneBtn).toBeTruthy();
    await fireEvent.click(markDoneBtn);
    expect(onMarkDone).toHaveBeenCalledTimes(1);
  });

  it("api exposes markPageCompleted method", () => {
    expect(typeof api.markPageCompleted).toBe("function");
  });
});
