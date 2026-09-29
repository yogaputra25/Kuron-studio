import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import canvasSrc from "./CanvasEditor.svelte?raw";

// Kontrak change fix-edit-stuck, pola-sumber seperti edit-in-bubble.test.ts:
// kursor/DOM tak bisa diassert statis — assert pemisahan draft vs state global
// yang menjamin kursor stabil + respons tak menimpa ketikan.
// (ponytail: @testing-library/svelte + jsdom saat butuh interaksi nyata;
// ceiling = ketik-tengah-kalimat + ketik-cepat-lawan-save beneran.)
describe("edit draft kartu (§1)", () => {
  it("oninput kartu tulis draft saja, tak sentuh translation.bubbles", () => {
    expect(panelSrc).toContain("let draft = $state<{ row: number; original: string; reading: string; translated: string } | null>(null)");
    expect(panelSrc).toContain("function draftInput(");
    const fn = panelSrc.slice(panelSrc.indexOf("function draftInput"), panelSrc.indexOf("function draftInput") + 700);
    expect(fn).toContain("draft[field] = v");
    expect(fn).not.toContain("translation.bubbles =");
    expect(fn).not.toContain("editTr(");
    // Ketiga textarea kartu memakai draftInput, bukan editTr langsung.
    const usages = panelSrc.match(/oninput=\{\(e\) => draftInput\("/g) ?? [];
    expect(usages.length).toBe(3);
    expect(panelSrc).not.toMatch(/oninput=\{\(e\) => editTr\(i, /);
  });

  it("textarea baca draft saat aktif, fallback ke b.field", () => {
    expect(panelSrc).toContain("value={draft && draft.row === i ? draft.original : b.original}");
    expect(panelSrc).toContain("value={draft && draft.row === i ? draft.reading : b.reading}");
    expect(panelSrc).toContain("value={draft && draft.row === i ? draft.translated : b.translated}");
  });

  it("blur/debounce commit via editTr existing (sekali, bukan per huruf)", () => {
    const blurs = panelSrc.match(/onblur=\{commitDraft\}/g) ?? [];
    expect(blurs.length).toBe(3);
    expect(panelSrc).toMatch(/draftTimer = setTimeout\(\(\) => \{ draftTimer = null; commitDraft\(\); \}, 500\)/);
    const commit = panelSrc.slice(panelSrc.indexOf("function commitDraft"), panelSrc.indexOf("function commitDraft") + 700);
    expect(commit).toContain('editTr(row, "original", original)');
    expect(commit).toContain('editTr(row, "reading", reading)');
    expect(commit).toContain('editTr(row, "translated", translated)');
    // Hanya field berubah yang di-commit (guard per field).
    expect(commit).toContain("if (cur.original !== original)");
  });

  it("pindah seleksi commit draft kotor; null buang draft", () => {
    expect(panelSrc).toContain("function selectBubble(");
    const fn = panelSrc.slice(panelSrc.indexOf("function selectBubble"), panelSrc.indexOf("function selectBubble") + 250);
    expect(fn).toContain("if (draft) commitDraft();");
    expect(fn).toContain("if (idx === null) draft = null;");
    // Semua jalur ganti manualSel lewat selectBubble (tak ada assignment langsung tersisa).
    expect(panelSrc).not.toContain("manualSel = i)");
    expect(panelSrc).not.toContain("(manualSel = manualSel === b.index ? null : b.index)");
  });

  it("Save edits + resetBubble + unmount amankan draft", () => {
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 200);
    expect(save).toContain("if (draft) commitDraft();");
    const reset = panelSrc.slice(panelSrc.indexOf("function resetBubble"), panelSrc.indexOf("function resetBubble") + 300);
    expect(reset).toContain("draft = null;");
    expect(panelSrc).toContain("onDestroy(() => { if (saveTimer) clearTimeout(saveTimer); if (draftTimer) clearTimeout(draftTimer); });");
  });

  it("hasil Translate baru commit draft dulu (tak hilang/timpa AI)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function onTranslated"), panelSrc.indexOf("function onTranslated") + 300);
    expect(fn).toContain("if (draft) commitDraft();");
  });

  it("editTr tak berubah (jalur save existing utuh)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function editTr"), panelSrc.indexOf("function editTr") + 400);
    expect(fn).toContain("translation.bubbles = translation.bubbles.map(");
    expect(fn).toContain("isUserEdited: true");
    expect(fn).toContain("queueSaveTranslationEdits()");
  });
});

describe("last-write-wins §4", () => {
  it("save kirim naikkan generasi; respons basi diabaikan sebelum tulis state", () => {
    expect(panelSrc).toContain("let saveGen = 0;");
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 800);
    expect(save).toContain("const my = ++saveGen;");
    expect(save).toContain("if (my !== saveGen) return;");
    // Guard sebelum KEDUA penulis state (translation + onSaved).
    const guardIdx = save.indexOf("if (my !== saveGen) return;");
    expect(save.indexOf("translation = out;", guardIdx)).toBeGreaterThan(guardIdx);
    expect(save.indexOf("onSaved(", guardIdx)).toBeGreaterThan(guardIdx);
  });

  it("↺ tulis ai* lalu save LANGSUNG (bukan debounce)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function resetBubble"), panelSrc.indexOf("function resetBubble") + 800);
    expect(fn).toContain("draft = null;");
    expect(fn).toContain("original: b.aiOriginal");
    expect(fn).toContain("isUserEdited: false");
    expect(fn).toContain("void saveTranslationEdits();");
    expect(fn).not.toContain("queueSaveTranslationEdits()");
  });

  it("error path tak menyentuh state (banner saja)", () => {
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 800);
    const catchBlock = save.slice(save.indexOf("} catch"));
    expect(catchBlock).toContain("error = String(e)");
    expect(catchBlock).not.toContain("translation =");
  });
});

describe("in-bubble bind:value (§2.1)", () => {
  it("textarea in-bubble two-way bind ke editingValue lokal", () => {
    expect(canvasSrc).toContain("bind:value={editingValue}");
    // [^:]: "bind:value=" mengandung "value=" — kecualikan prefix bind:.
    expect(canvasSrc).not.toMatch(/[^:]value=\{editingValue\}/);
  });
});
