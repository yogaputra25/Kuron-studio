import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import cardSrc from "./TranslationCard.svelte?raw";
import hookSrc from "./useDraftText.svelte?raw";
import canvasSrc from "./CanvasEditor.svelte?raw";

// Kontrak change fix-edit-stuck, pola-sumber seperti edit-in-bubble.test.ts:
// kursor/DOM tak bisa diassert statis — assert pemisahan draft vs state global
// yang menjamin kursor stabil + respons tak menimpa ketikan.
// (ponytail: @testing-library/svelte + jsdom saat butuh interaksi nyata;
// ceiling = ketik-tengah-kalimat + ketik-cepat-lawan-save beneran.)
describe("edit draft kartu (§1)", () => {
  it("oninput kartu tulis cache plain, tak sentuh $state/translation.bubbles", () => {
    // Sejak FE-4 (1.3): mesin di useDraftText, panel hanya wiring.
    expect(hookSrc).toContain("function input(field: DraftField, v: string)");
    const fn = hookSrc.slice(hookSrc.indexOf("function input("), hookSrc.indexOf("function input(") + 700);
    expect(fn).toContain("cur[field] = v");
    expect(fn).not.toContain("translation.bubbles =");
    expect(fn).not.toContain("editTr(");
    expect(fn).not.toContain("$state");
    // Ketiga textarea kartu memakai onDraftInput (prop ke draft.input panel).
    const usages = cardSrc.match(/oninput=\{\(e\) => onDraftInput\("/g) ?? [];
    expect(usages.length).toBe(3);
    expect(panelSrc).toContain("onDraftInput={(f, v) => draft.input(f, v)}");
    expect(cardSrc).not.toMatch(/oninput=\{\(e\) => editTr\(i, /);
  });

  it("textarea baca cache saat aktif, fallback ke b.field", () => {
    // Sejak FE-4: kartu terima draft sebagai prop (dulu baca draftBox.cur langsung).
    expect(cardSrc).toContain("value={draft && draft.row === i ? draft.original : b.original}");
    expect(cardSrc).toContain("value={draft && draft.row === i ? draft.reading : b.reading}");
    expect(cardSrc).toContain("value={draft && draft.row === i ? draft.translated : b.translated}");
    expect(panelSrc).toContain("draft={draft.cur}");
  });

  it("commit di batas alami via editTr existing (blur/pindah/Save/jaring, bukan tiap jeda)", () => {
    // Sejak FE-4 (1.3): blur kartu panggil prop onCommit, panel teruskan ke draft.
    const blurs = cardSrc.match(/onblur=\{onCommit\}/g) ?? [];
    expect(blurs.length).toBe(3);
    expect(panelSrc).toContain("onCommit={() => draft.commit()}");
    // input tanpa timer-500ms: tulis cache saja (mesin di hook).
    const input = hookSrc.slice(hookSrc.indexOf("function input("), hookSrc.indexOf("function input(") + 700);
    expect(input).not.toContain(", 500)");
    const commit = hookSrc.slice(hookSrc.indexOf("function doCommit"), hookSrc.indexOf("function doCommit") + 700);
    expect(commit).toContain('deps.commit(row, "original", original)');
    expect(commit).toContain('deps.commit(row, "reading", reading)');
    expect(commit).toContain('deps.commit(row, "translated", translated)');
    // Hanya field berubah yang di-commit (guard per field).
    expect(commit).toContain("if (b.original !== original)");
  });
  it("pindah seleksi commit draft kotor; null buang draft", () => {
    expect(panelSrc).toContain("function selectBubble(");
    const fn = panelSrc.slice(panelSrc.indexOf("function selectBubble"), panelSrc.indexOf("function selectBubble") + 250);
    expect(fn).toContain("if (draft.cur) draft.commit();");
    expect(fn).toContain("if (idx === null) draft.revert();");
    // Semua jalur ganti manualSel lewat selectBubble (tak ada assignment langsung tersisa).
    expect(panelSrc).not.toContain("manualSel = i)");
    expect(panelSrc).not.toContain("(manualSel = manualSel === b.index ? null : b.index)");
  });

  it("Save edits + resetBubble + unmount amankan draft", () => {
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 500);
    expect(save).toContain("if (draft.cur) draft.commit();");
    const reset = panelSrc.slice(panelSrc.indexOf("function resetBubble"), panelSrc.indexOf("function resetBubble") + 300);
    expect(reset).toContain("draft.revert();");
    expect(panelSrc).toContain("if (draft.cur) draft.commit(); if (saveTimer)");
  });

  it("hasil Translate baru commit draft dulu (tak hilang/timpa AI)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function onTranslated"), panelSrc.indexOf("function onTranslated") + 300);
    expect(fn).toContain("if (draft.cur) draft.commit();");
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
    // Sejak FE-4 (1.3): invarian di hook — cur plain, satu-satunya $state = dirty.
    expect(hookSrc).toContain("let cur: DraftValue | null = null;");
    expect(hookSrc).toContain("let dirty = $state(false);");
    const fn = hookSrc.slice(hookSrc.indexOf("function input("), hookSrc.indexOf("function input(") + 900);
    expect(fn).not.toMatch(/=\s*\$state/);
    expect(fn).not.toContain("draftShown");
  });

  it("guard IME: komposisi tunda commit, selesai langsung commit", () => {
    // Sejak FE-4 (1.3): guard di hook, panel hanya wiring prop.
    expect(hookSrc).toContain("function compStart()");
    expect(hookSrc).toContain("function compEnd()");
    const commit = hookSrc.slice(hookSrc.indexOf("function doCommit"), hookSrc.indexOf("function doCommit") + 200);
    expect(commit).toContain("if (composing) return;");
    const end = hookSrc.slice(hookSrc.indexOf("function compEnd()"), hookSrc.indexOf("function compEnd()") + 200);
    expect(end).toContain("if (cur) doCommit();");
    const areas = cardSrc.match(/oncompositionstart=\{onCompStart\}/g) ?? [];
    expect(areas.length).toBe(3);
    const ends = cardSrc.match(/oncompositionend=\{onCompEnd\}/g) ?? [];
    expect(ends.length).toBe(3);
    expect(panelSrc).toContain("onCompStart={() => draft.compStart()}");
    expect(panelSrc).toContain("onCompEnd={() => draft.compEnd()}");
  });

  it("timer-jaring 10dtk sekali per baris kotor (bukan ritme kerja)", () => {
    expect(hookSrc).toContain("export const DRAFT_SAFETY_MS = 10_000;");
    // Penjaga sekali-arm di helper arm(), dipanggil di ujung input().
    const arm = hookSrc.slice(hookSrc.indexOf("function arm()"), hookSrc.indexOf("function arm()") + 300);
    expect(arm).toContain("if (!timer) timer = setTimeout(");
    expect(arm).toContain("DRAFT_SAFETY_MS");
    const input = hookSrc.slice(hookSrc.indexOf("function input("), hookSrc.indexOf("function input(") + 900);
    expect(input).toContain("arm();");
  });

  it("indikator dirty + Save flush eksplisit + flush-on-destroy", () => {
    // Sejak FE-4 (1.3): dirty $state di hook, badge di kartu, flush di panel.
    expect(hookSrc).toContain("let dirty = $state(false);");
    expect(cardSrc).toContain("belum tersimpan");
    expect(panelSrc).toContain("if (draft.cur) draft.commit(); if (dirty) await save(); await saveTranslationEdits();");
    expect(panelSrc).toContain("if (draft.cur) draft.commit(); if (saveTimer)");
  });
});

describe("last-write-wins §4", () => {
  it("save kirim naikkan generasi; respons basi diabaikan sebelum tulis state", () => {
    expect(panelSrc).toContain("let saveGen = 0;");
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 1800);
    expect(save).toContain("const my = ++saveGen;");
    expect(save).toContain("if (my !== saveGen) return;");
    // Guard sebelum KEDUA penulis state (translation + onSaved).
    const guardIdx = save.indexOf("if (my !== saveGen) return;");
    expect(save.indexOf("translation = out;", guardIdx)).toBeGreaterThan(guardIdx);
    expect(save.indexOf("onSaved(", guardIdx)).toBeGreaterThan(guardIdx);
  });

  it("↺ tulis ai* lalu save LANGSUNG (bukan debounce)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function resetBubble"), panelSrc.indexOf("function resetBubble") + 800);
    expect(fn).toContain("draft.revert();");
    expect(fn).toContain("original: b.aiOriginal");
    expect(fn).toContain("isUserEdited: false");
    expect(fn).toContain("void saveTranslationEdits();");
    expect(fn).not.toContain("queueSaveTranslationEdits()");
  });

  it("error path tak menyentuh state (banner saja)", () => {
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 2000);
    const catchBlock = save.slice(save.indexOf("} catch"));
    expect(catchBlock).toContain("error = String(e)");
    expect(catchBlock).not.toContain("translation =");
  });
});

describe("sync-guard translation (typing-comfort D6)", () => {
  it("skip timpa prop saat kotor/terbang; sync normal saat tenang", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function syncFromPage"), panelSrc.indexOf("function syncFromPage") + 2000);
    // Kondisi sibuk: cache kotor ATAU save terbang (timer/saving).
    expect(fn).toContain("draft.cur !== null || saveTimer !== null || savingTr");
    // Sejak FE-4 (2.2): keputusan via helper (behavior di useSyncedPage.test.ts).
    expect(fn).toContain("shouldApplyTranslation({ busyTyping, incomingRev: incomingTr, current: translation })");
    // busyTyping hanya dipakai guard cabang translation.
    const uses = panelSrc.match(/busyTyping/g) ?? [];
    expect(uses.length).toBe(2);
  });

  it("cabang bubbles punya guard sendiri (fix-geometry-revert D1)", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function syncFromPage"), panelSrc.indexOf("function syncFromPage") + 900);
    // Guard = edit lokal atau save terbang; ganti halaman tetap memaksa resync.
    expect(fn).toContain("const samePage = syncedFor === page.id;");
    expect(fn).toContain("const localBusy = dirty || saving || savingTr || saveTimer !== null;");
    expect(fn).toContain("shouldApplyBubbles({ samePage, localBusy, incoming, current: bubbles })");
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
    // saveTranslationEdits + createEmptyTranslation + onTranslated + markDone + error.
    expect(echoes.length).toBe(5);
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
    expect(panelSrc).toContain("const mine = allPages(proj).find((p) => p.id === page.id);");
    expect(panelSrc).toContain("bubbles = $state.snapshot(mine.bubbles ?? []);");
  });
});

describe("P9 push layout otomatis (export-text-parity)", () => {
  it("terjemahan baru langsung memicu pengiriman layout", () => {
    // Backend tak punya metrik font browser: tanpa push ini, terjemahan hasil
    // AI tersimpan dengan `layout` kosong dan export jatuh ke jalur fallback.
    const fn = panelSrc.slice(
      panelSrc.indexOf("function onTranslated"),
      panelSrc.indexOf("function onTranslated") + 800,
    );
    expect(fn).toContain("void saveTranslationEdits();");
  });

  it("data lama diselamatkan sekali per halaman, dengan guard", () => {
    expect(panelSrc).toContain('let layoutPushedFor = "";');
    const eff = panelSrc.slice(
      panelSrc.indexOf("layoutPushedFor === page.id"),
      panelSrc.indexOf("layoutPushedFor === page.id") + 500,
    );
    // Hanya bila ada bubble yang belum punya layout, dan tak menabrak save.
    expect(eff).toContain("b.layout");
    expect(eff).toContain("savingTr || saveTimer !== null");
    expect(eff).toContain("void saveTranslationEdits();");
  });
});
