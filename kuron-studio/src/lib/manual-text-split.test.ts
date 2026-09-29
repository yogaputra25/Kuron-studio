import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import canvasSrc from "./CanvasEditor.svelte?raw";

// Kontrak change manual-text-before-after, diverifikasi statis dari sumber
// (pola sama seperti canvas-editor.test.ts): komponen terikat Konva + DOM +
// Svelte runtime sehingga mount unit murah tidak bermakna — assert pada pola
// kode yang menjamin kontrak.
//
// (ponytail: upgrade ke component-test sungguhan — @testing-library/svelte +
// jsdom + Konva asli — saat repo butuh interaksi nyata; ceiling = klik/ketik
// beneran, bukan pola string.)
describe("manual text entry (1.1–1.3, via draft fix-edit-stuck)", () => {
  it("original/reading/translated adalah textarea editable per bubble", () => {
    // Sejak fix-edit-stuck: oninput tulis draft lokal, commit via editTr.
    expect(panelSrc).toMatch(/draftInput\("original"/);
    expect(panelSrc).toMatch(/draftInput\("reading"/);
    expect(panelSrc).toMatch(/draftInput\("translated"/);
    const commit = panelSrc.slice(panelSrc.indexOf("function commitDraft"), panelSrc.indexOf("function commitDraft") + 700);
    expect(commit).toContain('editTr(row, "original"');
    expect(commit).toContain('editTr(row, "reading"');
    expect(commit).toContain('editTr(row, "translated"');
    const areas = panelSrc.match(/<textarea/g) ?? [];
    expect(areas.length).toBeGreaterThanOrEqual(3);
    // Tak ada lagi <p>O:> read-only.
    expect(panelSrc).not.toContain("<p class=\"truncate text-zinc-500\"");
  });

  it("editTr set isUserEdited=true untuk ketiga field", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function editTr"), panelSrc.indexOf("function editTr") + 400);
    expect(fn).toContain("isUserEdited: true");
    expect(fn).not.toContain('field === "translated"');
  });

  it("debounce ~500ms: ketik cepat → satu save via queueSaveTranslationEdits", () => {
    expect(panelSrc).toContain("queueSaveTranslationEdits");
    expect(panelSrc).toMatch(/setTimeout\(\(\) => \{ saveTimer = null; void saveTranslationEdits\(\); \}, 500\)/);
    // editTr antre, bukan save langsung.
    const fn = panelSrc.slice(panelSrc.indexOf("function editTr"), panelSrc.indexOf("function editTr") + 400);
    expect(fn).toContain("queueSaveTranslationEdits()");
    expect(fn).not.toContain("saveTranslationEdits()");
    // Error save tampil di banner existing, teks lokal tetap (catch hanya set error).
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 600);
    expect(save).toContain("error = String(e)");
    expect(save).not.toContain("translation = null");
  });

  it("tombol Isi manual bangun PageTranslation kosong → api.saveTranslation", () => {
    expect(panelSrc).toContain("Isi manual");
    expect(panelSrc).toContain("createEmptyTranslation");
    const fn = panelSrc.slice(panelSrc.indexOf("async function createEmptyTranslation"), panelSrc.indexOf("async function createEmptyTranslation") + 1200);
    expect(fn).toContain("original: \"\", reading: \"\", translated: \"\"");
    expect(fn).toContain("api.saveTranslation(page.id, empty.bubbles)");
    // Validasi ringan index unik (backend save_translation tanpa validasi).
    expect(fn).toContain("new Set(idx).size");
  });

  it("guard syncFromPage tak tersentuh: save identik tidak reset saat ngetik", () => {
    expect(panelSrc).toContain("if (syncedFor !== page.id || JSON.stringify(incoming) !== JSON.stringify(bubbles)) {");
  });
});

describe("before|split|after (3.1–3.4)", () => {
  it("state mode + switcher Before|Split|After", () => {
    expect(panelSrc).toContain('let mode = $state<PaneMode>("before")');
    expect(panelSrc).toContain('type PaneMode = "before" | "split" | "after"');
    expect(panelSrc).toContain('[["before", "Before"], ["split", "Split"], ["after", "After"]]');
  });

  it("before = editable tanpa overlay; after = read-only dengan overlay", () => {
    expect(panelSrc).toMatch(/showTranslation=\{false\}[\s\S]{0,120}editable=\{true\}/);
    expect(panelSrc).toMatch(/showTranslation=\{true\}[\s\S]{0,120}editable=\{false\}/);
  });

  it("split = dua pane flex-1 + satu data-URL dishare (get_image_preview sekali)", () => {
    const splits = panelSrc.match(/class="min-w-0 flex-1 overflow-auto p-4"/g) ?? [];
    expect(splits.length).toBe(2);
    // Kedua pane pakai referensi imageUrl yang sama — tak ada fetch kedua.
    const usages = panelSrc.match(/imageUrl=\{fullImageUrl\}/g) ?? [];
    expect(usages.length).toBe(2);
    expect(panelSrc).not.toContain("getImagePreview");
  });

  it("scroll-sync: tiru scrollTop/scrollLeft di bawah guard flag anti-loop", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function syncScroll"), panelSrc.indexOf("function syncScroll") + 300);
    expect(fn).toContain("if (!from || !to || syncing) return;");
    expect(fn).toContain("to.scrollTop = from.scrollTop;");
    expect(fn).toContain("to.scrollLeft = from.scrollLeft;");
    expect(panelSrc).toContain("onscroll={() => syncScroll(beforeScroll, afterScroll)}");
    expect(panelSrc).toContain("onscroll={() => syncScroll(afterScroll, beforeScroll)}");
  });

  it("after read-only: klik teruskan seleksi ke before, bubbles tak berubah", () => {
    // Pane after: onChange no-op (bubbles unchanged) + forward seleksi.
    // Sejak fix-edit-stuck: forward lewat selectBubble (commit draft kotor dulu).
    expect(panelSrc).toContain("onSelectForward={(i) => selectBubble(i)}");
    expect(panelSrc).toContain("onChange={() => {}}");
    // Pane before ikut seleksi via externalSelected (satu sumber state).
    expect(panelSrc).toContain("externalSelected={manualSel}");
    // Canvas: klik di mode read-only forward, bukan edit lokal saja.
    expect(canvasSrc).toContain("if (!editable && onSelectForward) onSelectForward(i);");
    // Canvas read-only: drag dibatalkan (redraw saja), tanpa onChange.
    const drag = canvasSrc.slice(canvasSrc.indexOf('group.on("dragend"'), canvasSrc.indexOf('group.on("dragend"') + 250);
    expect(drag).toContain("if (!editable) { redraw(); return; }");
    // Drawing tools mati saat read-only; resize handle hanya saat editable.
    expect(canvasSrc).toContain("function onStageDown(e: Konva.KonvaEventObject<MouseEvent | TouchEvent>) {\n    if (!editable) return;");
    expect(canvasSrc).toContain("if (!b.shape && editable) {");
  });
});
