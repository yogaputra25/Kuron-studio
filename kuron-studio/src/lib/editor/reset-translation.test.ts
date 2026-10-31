// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import TranslationCard from "./TranslationCard.svelte";
import { mergeUserEdits } from "./translation";
import type { BubbleTranslation } from "../contracts/types";

describe("reset-translation behavior", () => {
  it("renders reset button ↺ only when bubble is user-edited and has ai baseline", async () => {
    const onResetBubble = vi.fn();
    const editedBubble: BubbleTranslation = {
      index: 0,
      x: 0,
      y: 0,
      w: 100,
      h: 50,
      original: "manual",
      reading: "",
      translated: "manual translation",
      aiOriginal: "ai-orig",
      aiReading: "",
      aiTranslated: "ai translation",
      needsWhitePatch: false,
      isUserEdited: true,
    };

    const { getByRole, queryByRole, rerender } = renderWithI18n(TranslationCard, {
      props: {
        hasTranslation: true,
        card: { b: editedBubble, i: 0 },
        mode: "split",
        trgLabel: "Target",
        draft: null,
        dirtyDraft: false,
        glossMsg: "",
        savingTr: false,
        showMarkDone: false,
        hasBubbles: true,
        geomBox: null,
        geomIdx: 0,
        geomOutOfSync: false,
        onSelectBubble: vi.fn(),
        onResetBubble,
        onDraftInput: vi.fn(),
        onCompStart: vi.fn(),
        onCompEnd: vi.fn(),
        onCommit: vi.fn(),
        onPressStart: vi.fn(),
        onPressEnd: vi.fn(),
        onMarkDone: vi.fn(),
        onSaveAll: vi.fn(),
        onSetGeometry: vi.fn(),
        onScaleBubble: vi.fn(),
        onSetFontSize: vi.fn(),
        onCreateEmpty: vi.fn(),
      },
    });

    const resetBtn = getByRole("button", { name: "↺" });
    expect(resetBtn).toBeTruthy();

    await fireEvent.click(resetBtn);
    expect(onResetBubble).toHaveBeenCalledWith(0);

    // Rerender with unedited bubble
    await rerender({
      card: {
        b: { ...editedBubble, isUserEdited: false },
        i: 0,
      },
    });
    expect(queryByRole("button", { name: "↺" })).toBeNull();
  });

  it("mergeUserEdits preserves user edited translations over new translations", () => {
    const prev: BubbleTranslation[] = [
      {
        index: 0,
        x: 0,
        y: 0,
        w: 100,
        h: 50,
        original: "orig-1",
        reading: "",
        translated: "my manual edit",
        aiOriginal: "orig-1",
        aiReading: "",
        aiTranslated: "old ai",
        needsWhitePatch: false,
        isUserEdited: true,
      },
    ];

    const next: BubbleTranslation[] = [
      {
        index: 0,
        x: 0,
        y: 0,
        w: 100,
        h: 50,
        original: "orig-1",
        reading: "",
        translated: "new ai translated",
        aiOriginal: "orig-1",
        aiReading: "",
        aiTranslated: "new ai translated",
        needsWhitePatch: false,
        isUserEdited: false,
      },
    ];

    const merged = mergeUserEdits(prev, next);
    expect(merged[0].translated).toBe("my manual edit");
    expect(merged[0].isUserEdited).toBe(true);
  });
});
