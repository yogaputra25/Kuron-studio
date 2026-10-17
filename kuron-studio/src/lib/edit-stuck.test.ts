import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import canvasSrc from "./CanvasEditor.svelte?raw";

// Kontrak change fix-edit-stuck, pola-sumber seperti edit-in-bubble.test.ts:
// kursor/DOM tak bisa diassert statis — assert pemisahan draft vs state global
// yang menjamin kursor stabil + respons tak menimpa ketikan.
// (ponytail: @testing-library/svelte + jsdom saat butuh interaksi nyata;
// ceiling = ketik-tengah-kalimat + ketik-cepat-lawan-save beneran.)
describe("edit draft kartu (§1)", () => {
  it("oninput kartu tulis cache plain, tak sentuh $state/translation.bubbles", () => {
    expect(panelSrc).toContain("let draftBox: { cur:");
    expect(panelSrc).toContain("function draftInput(");
    const fn = panelSrc.slice(panelSrc.indexOf("function draftInput"), panelSrc.indexOf("function draftInput") + 900);
    expect(fn).toContain("draftBox.cur[field] = v");
    expect(fn).not.toContain("translation.bubbles =");
    expect(fn).not.toContain("editTr(");
    expect(fn).not.toContain("$state");
    // Ketiga textarea kartu memakai draftInput, bukan editTr langsung.
    const usages = panelSrc.match(/oninput=\{\(e\) => draftInput\("/g) ?? [];
    expect(usages.length).toBe(3);
    expect(panelSrc).not.toMatch(/oninput=\{\(e\) => editTr\(i, /);
  });

  it("textarea baca cache saat aktif, fallback ke b.field", () => {
    expect(panelSrc).toContain("value={draftBox.cur && draftBox.cur.row === i ? draftBox.cur.original : b.original}");
    expect(panelSrc).toContain("value={draftBox.cur && draftBox.cur.row === i ? draftBox.cur.reading : b.reading}");
    expect(panelSrc).toContain("value={draftBox.cur && draftBox.cur.row === i ? draftBox.cur.translated : b.translated}");
  });

  it("commit di batas alami via editTr existing (blur/pindah/Save/jaring, bukan tiap jeda)", () => {
    const blurs = panelSrc.match(/onblur=\{commitDraft\}/g) ?? [];
    expect(blurs.length).toBe(3);
    // draftInput tanpa timer-500ms: tulis cache saja.
    const input = panelSrc.slice(panelSrc.indexOf("function draftInput"), panelSrc.indexOf("function draftInput") + 900);
    expect(input).not.toContain(", 500)");
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
    expect(fn).toContain("if (draftBox.cur) commitDraft();");
    expect(fn).toContain("draftBox.cur = null;");
    // Semua jalur ganti manualSel lewat selectBubble (tak ada assignment langsung tersisa).
    expect(panelSrc).not.toContain("manualSel = i)");
    expect(panelSrc).not.toContain("(manualSel = manualSel === b.index ? null : b.index)");
  });

  it("Save edits + resetBubble + unmount amankan draft", () => {
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 200);
    expect(save).toContain("if (draftBox.cur) commitDraft();");
    const reset = panelSrc.slice(panelSrc.indexOf("function resetBubble"), panelSrc.indexOf("function resetBubble") + 300);
    expect(reset).toContain("draftBox.cur = null;");
    expect(panelSrc).toContain("if (draftBox.cur) commitDraft(); if (saveTimer)");
  });

  it("hasil Translate baru commit draft dulu (tak hilang/timpa AI)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function onTranslated"), panelSrc.indexOf("function onTranslated") + 300);
    expect(fn).toContain("if (draftBox.cur) commitDraft();");
  });

  it("editTr tulis teks saja, tak sentuh seleksi (anti-yank)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function editTr"), panelSrc.indexOf("function editTr") + 400);
    expect(fn).toContain("translation.bubbles = translation.bubbles.map(");
    expect(fn).toContain("isUserEdited: true");
    expect(fn).toContain("queueSaveTranslationEdits()");
    // Seleksi milik selectBubble saja — commit editor lama tak boleh yank.
    expect(fn).not.toContain("manualSel");
  });

  it("oninput tak tulis reactive per huruf (cache plain, nol render)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function draftInput"), panelSrc.indexOf("function draftInput") + 900);
    expect(fn).not.toMatch(/=\s*\$state/);
    expect(fn).not.toContain("draftShown");
  });

  it("guard IME: komposisi tunda commit, selesai langsung commit", () => {
    expect(panelSrc).toContain("function compStart()");
    expect(panelSrc).toContain("function compEnd()");
    const commit = panelSrc.slice(panelSrc.indexOf("function commitDraft"), panelSrc.indexOf("function commitDraft") + 200);
    expect(commit).toContain("if (composing) return;");
    const end = panelSrc.slice(panelSrc.indexOf("function compEnd()"), panelSrc.indexOf("function compEnd()") + 200);
    expect(end).toContain("if (draftBox.cur) commitDraft();");
    const areas = panelSrc.match(/oncompositionstart=\{compStart\}/g) ?? [];
    expect(areas.length).toBe(3);
    const ends = panelSrc.match(/oncompositionend=\{compEnd\}/g) ?? [];
    expect(ends.length).toBe(3);
  });

  it("timer-jaring 10dtk sekali per baris kotor (bukan ritme kerja)", () => {
    expect(panelSrc).toContain("const DRAFT_SAFETY_MS = 10_000;");
    const input = panelSrc.slice(panelSrc.indexOf("function draftInput"), panelSrc.indexOf("function draftInput") + 900);
    expect(input).toContain("if (!draftTimer) draftTimer = setTimeout(");
    expect(input).toContain("DRAFT_SAFETY_MS");
  });

  it("indikator dirty + Save flush eksplisit + flush-on-destroy", () => {
    expect(panelSrc).toContain("let dirtyDraft = $state(false);");
    expect(panelSrc).toContain("belum tersimpan");
    expect(panelSrc).toContain("if (draftBox.cur) commitDraft(); if (dirty) await save(); await saveTranslationEdits();");
    expect(panelSrc).toContain("if (draftBox.cur) commitDraft(); if (saveTimer)");
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
    expect(fn).toContain("draftBox.cur = null;");
    expect(fn).toContain("original: b.aiOriginal");
    expect(fn).toContain("isUserEdited: false");
    expect(fn).toContain("void saveTranslationEdits();");
    expect(fn).not.toContain("queueSaveTranslationEdits()");
  });

  it("error path tak menyentuh state (banner saja)", () => {
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 1000);
    const catchBlock = save.slice(save.indexOf("} catch"));
    expect(catchBlock).toContain("error = String(e)");
    expect(catchBlock).not.toContain("translation =");
  });
});

describe("sync-guard translation (typing-comfort D6)", () => {
  it("skip timpa prop saat kotor/terbang; sync normal saat tenang", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function syncFromPage"), panelSrc.indexOf("function syncFromPage") + 2000);
    // Kondisi sibuk: cache kotor ATAU save terbang (timer/saving).
    expect(fn).toContain("draftBox.cur !== null || saveTimer !== null || savingTr");
    expect(fn).toContain("if (!busyTyping && incomingTr");
    // busyTyping hanya dipakai guard cabang translation.
    const uses = panelSrc.match(/busyTyping/g) ?? [];
    expect(uses.length).toBe(2);
  });

  it("cabang bubbles punya guard sendiri (fix-geometry-revert D1)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function syncFromPage"), panelSrc.indexOf("function syncFromPage") + 900);
    // Guard = edit lokal atau save terbang; ganti halaman tetap memaksa resync.
    expect(fn).toContain("const samePage = syncedFor === page.id;");
    expect(fn).toContain("const localBusy = dirty || saving || savingTr || saveTimer !== null;");
    expect(fn).toContain("if (!samePage || (!localBusy &&");
  });
});

describe("in-bubble bind:value (§2.1)", () => {
  it("textarea in-bubble two-way bind ke editingValue lokal", () => {
    expect(canvasSrc).toContain("bind:value={editingValue}");
    // [^:]: "bind:value=" mengandung "value=" — kecualikan prefix bind:.
    expect(canvasSrc).not.toMatch(/[^:]value=\{editingValue\}/);
  });
});

describe("geometry persistence (fix-geometry-revert)", () => {
  it("D2: echo onSaved membawa bubbles lokal (bukan prop basi)", () => {
    const echoes = panelSrc.match(/onSaved\(\{ \.\.\.page, bubbles: \$state\.snapshot\(bubbles\)/g) ?? [];
    // saveTranslationEdits + createEmptyTranslation + onTranslated + error.
    expect(echoes.length).toBe(4);
    // Tak ada jalur echo yang masih mengirim page.bubbles basi.
    expect(panelSrc).not.toContain('onSaved({ ...page, status: "translated"');
    expect(panelSrc).not.toContain('onSaved({ ...page, status: "failed" })');
  });

  it("D3: Save sidebar menuntaskan geometri lebih dulu bila dirty", () => {
    expect(panelSrc).toContain("if (dirty) await save(); await saveTranslationEdits();");
  });
});

describe("guard tidak memblokir niat (fix-geometry-revert D1)", () => {
  it("detect halaman ini tetap diterapkan walau dirty", () => {
    // Guard `dirty` menahan propagasi prop; Detect = penggantian yang
    // disengaja, jadi jalur detect-all menulis state lokal langsung.
    expect(panelSrc).toContain("const mine = proj.pages.find((p) => p.id === page.id);");
    expect(panelSrc).toContain("bubbles = $state.snapshot(mine.bubbles ?? []);");
  });
});
