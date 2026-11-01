import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import cardSrc from "./TranslationCard.svelte?raw";
import hookSrc from "./useDraftText.svelte?raw";
import toolbarSrc from "./EditorToolbar.svelte?raw";
import panesSrc from "./EditorPanes.svelte?raw";
import handlesSrc from "./render/handles.ts?raw";
import toolsSrc from "./drawingTools.ts?raw";
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
    // Sejak FE-4 (1.1): markup kartu tinggal di TranslationCard,
    // panel hanya memasok wiring draft via prop onDraftInput.
    expect(cardSrc).toMatch(/onDraftInput\("original"/);
    expect(cardSrc).toMatch(/onDraftInput\("reading"/);
    expect(cardSrc).toMatch(/onDraftInput\("translated"/);
    expect(panelSrc).toContain("onDraftInput={(f, v) => draft.input(f, v)}");
    // Commit via hook deps (behavior dikunci unit useDraftText).
    const commit = hookSrc.slice(hookSrc.indexOf("function doCommit"), hookSrc.indexOf("function doCommit") + 700);
    expect(commit).toContain('deps.commit(row, "original"');
    expect(commit).toContain('deps.commit(row, "reading"');
    expect(commit).toContain('deps.commit(row, "translated"');
    const areas = cardSrc.match(/<textarea/g) ?? [];
    expect(areas.length).toBeGreaterThanOrEqual(3);
    // Tak ada lagi <p>O:> read-only.
    expect(cardSrc).not.toContain("<p class=\"truncate text-zinc-500\"");
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
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 1800);
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
    // Sejak FE-4 (2.2): guard struktural via helper (behavior di useSyncedPage.test.ts).
    expect(panelSrc).toContain("shouldApplyBubbles({ samePage, localBusy, incoming, current: bubbles })");
  });
});

describe("before|split|after (3.1–3.4)", () => {
  it("state mode + switcher Before|Split|After", () => {
    // Sejak FE-4 (2.1): state di panel, switcher di EditorToolbar.
    expect(panelSrc).toContain('let mode = $state<PaneMode>("before")');
    expect(toolbarSrc).toContain('export type PaneMode = "before" | "split" | "after"');
    expect(toolbarSrc).toContain('[["before", "Before"], ["split", "Split"], ["after", "After"]]');
  });

  it("before = editable tanpa overlay; after = overlay + geometri editable", () => {
    // Sejak FE-4 (3.2): override per pane di EditorPanes, shared dari panel.
    expect(panesSrc).toMatch(/showTranslation=\{false\}[\s\S]{0,120}editable=\{true\}/);
    // After: overlay terjemahan + geometri editable (bubble-edit-any-pane), tanpa gambar.
    expect(panesSrc).toMatch(/showTranslation=\{true\}[\s\S]{0,160}editable=\{true\}[\s\S]{0,80}canDraw=\{false\}/);
    expect(panelSrc).toContain("translations: translation?.bubbles ?? [],");
  });

  it("split = dua pane flex-1 + satu data-URL dishare (get_image_preview sekali)", () => {
    // Sejak FE-4 (3.2): scroll-div di EditorPanes, satu URL di shared panel.
    const splits = panesSrc.match(/class="min-h-0 min-w-0 flex-1 overflow-auto p-4"/g) ?? [];
    expect(splits.length).toBe(2);
    expect(panelSrc).toContain("imageUrl: fullImageUrl,");
    expect(panesSrc).toContain("{...shared}");
    expect(panelSrc).not.toContain("getImagePreview");
  });

  it("scroll-sync: tiru scrollTop/scrollLeft di bawah guard flag anti-loop", () => {
    const fn = panelSrc.slice(panelSrc.indexOf("function syncScroll"), panelSrc.indexOf("function syncScroll") + 500);
    // Stacked (mobile/column): tiap pane scroll sendiri, sync dimatikan.
    expect(fn).toContain("if (resize.columnMode) return;");
    expect(fn).toContain("if (!from || !to || syncing) return;");
    expect(fn).toContain("to.scrollTop = from.scrollTop;");
    expect(fn).toContain("to.scrollLeft = from.scrollLeft;");
    expect(panesSrc).toContain("onScroll(beforeScrollEl, afterScrollEl)");
    expect(panesSrc).toContain("onScroll(afterScrollEl, beforeScrollEl)");
    expect(panelSrc).toContain("onScroll={(f, t) => syncScroll(f, t)}");
  });

  it("after editable geometri: seleksi selalu forward, gambar tetap Before-only", () => {
    // bubble-edit-any-pane D1: pane After boleh ubah geometri → onChange nyata.
    // Sejak FE-4 (3.2): wiring gabung di shared, override di EditorPanes.
    expect(panelSrc).toContain("onSelectForward: (i) => selectBubble(i),");
    expect(panesSrc).toContain("editable={true}");
    expect(panesSrc).toContain("canDraw={false}");
    expect(panesSrc).not.toContain("onChange={() => {}}");
    // Pane before & after berbagi seleksi via externalSelected (satu state).
    expect(panelSrc).toContain("externalSelected: manualSel,");
    // D2: seleksi SELALU diteruskan, bukan hanya saat read-only.
    expect(canvasSrc).toContain("if (onSelectForward) onSelectForward(i);");
    expect(canvasSrc).not.toContain("if (!editable && onSelectForward)");
    // D1: gerbang menggambar pakai canDraw (Before-only), bukan editable.
    // Sejak FE-5 (3.2): gate di drawingTools.down, kanvas pasok flag.
    expect(toolsSrc).toContain("if (!deps.getCanDraw()) {");
    expect(canvasSrc).toContain("getCanDraw: () => canDraw,");
    // D3: handle resize hanya bubble terpilih (guard di modul sejak FE-5 3.1).
    expect(handlesSrc).toContain("if (!(!b.shape && editable && selected && tool");
    // Canvas tetap jaga jalur non-editable (prop publik, default true).
    const drag = canvasSrc.slice(canvasSrc.indexOf('group.on("dragend"'), canvasSrc.indexOf('group.on("dragend"') + 250);
    expect(drag).toContain("if (!editable) { redraw(); return; }");
  });
});

describe("bubble edit any pane", () => {
  it("After bisa ubah geometri: onChange nyata + canDraw false", () => {
    // Sejak FE-4 (3.2): After di EditorPanes, onChange nyata via shared.
    expect(panelSrc).toContain("bind:afterApi={afterRef}");
    expect(panesSrc).toContain("editable={true}");
    expect(panesSrc).toContain("canDraw={false}");
    expect(panelSrc).toContain("onChange: (nb) => { bubbles = nb; dirty = true;");
    expect(panesSrc).toContain("{...shared}");
  });

  it("Delete hidup di After-only: coba editorRef lalu afterRef", () => {
    const key = panelSrc.slice(panelSrc.indexOf("function onKey"), panelSrc.indexOf("function onKey") + 700);
    expect(key).toContain("editorRef?.deleteSelected()");
    expect(key).toContain("afterRef?.deleteSelected()");
    // Tipe ref = CanvasApi EditorPanes (membawa deleteSelected).
    expect(panesSrc).toContain('"deleteSelected"');
    expect(panelSrc).toContain("CanvasApi");
  });

  it("canDraw prop di canvas: default literal false, gerbang onStageDown", () => {
    expect(canvasSrc).toContain("canDraw?: boolean;");
    expect(canvasSrc).toContain("canDraw = false");
  });
});
