import { describe, expect, it } from "vitest";
import { statusAfterBubbleSave, statusAfterTranslationSave, translationComplete } from "./status";
import panelSrc from "../editor/EditorPanel.svelte?raw";
import cardSrc from "../editor/TranslationCard.svelte?raw";
import apiSrc from "../contracts/api.ts?raw";

// page-status-truth: status turunan data, bukan last-write-wins.
// Pure rule diuji langsung; kabel UI dikunci via source-text.
describe("translationComplete", () => {
  it("lengkap bila semua terisi", () => {
    expect(translationComplete([{ translated: "a" }, { translated: "b" }])).toBe(true);
  });

  it("sebagian / kosong / blank tak lengkap", () => {
    expect(translationComplete([{ translated: "a" }, { translated: "" }])).toBe(false);
    expect(translationComplete([{ translated: "  " }])).toBe(false);
    expect(translationComplete([])).toBe(false);
  });

  it("flag manual menang", () => {
    expect(translationComplete([{ translated: "a" }, { translated: "" }], true)).toBe(true);
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
    expect(statusAfterTranslationSave("detected", [{ translated: "x" }], false)).toBe("translated");
  });

  it("sebagian mempertahankan status lama", () => {
    expect(statusAfterTranslationSave("detected", [{ translated: "x" }, { translated: "" }])).toBe("detected");
    expect(statusAfterTranslationSave("idle", [])).toBe("idle");
  });

  it("transient translating jatuh ke detected", () => {
    expect(statusAfterTranslationSave("translating", [{ translated: "" }])).toBe("detected");
  });
});

describe("page-status wiring", () => {
  it("EditorPanel memakai helper, bukan status hardcode", () => {
    expect(panelSrc).toContain("statusAfterBubbleSave(page.status");
    expect(panelSrc).toContain("statusAfterTranslationSave(page.status");
    expect(panelSrc).not.toContain('status: cleaned.length > 0 ? "detected" : "noBubbles"');
  });

  it("tombol Tandai selesai hanya untuk terjemahan sebagian", () => {
    expect(panelSrc).toContain("!translation.manualDone && !translationComplete(translation.bubbles)");
    expect(panelSrc).toContain("api.markPageCompleted(page.id, true)");
    // Sejak FE-4: tombol tinggal di TranslationCard, panel memasok wiring.
    expect(cardSrc).toContain('$t("markDone")');
    expect(panelSrc).toContain("onMarkDone={() => void markDone()}");
  });

  it("api + kontrak mengenal mark_page_completed", () => {
    expect(apiSrc).toContain('invoke<PageTranslation>("mark_page_completed", { page_id, completed })');
  });
});
