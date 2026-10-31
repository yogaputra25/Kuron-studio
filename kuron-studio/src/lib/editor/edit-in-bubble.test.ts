import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import cardSrc from "./TranslationCard.svelte?raw";
import hookSrc from "./useDraftText.svelte?raw";
import overlaySrc from "./render/overlay.ts?raw";
import editorSrc from "./useBubbleEditor.svelte?raw";
import canvasSrc from "./CanvasEditor.svelte?raw";

// Kontrak change edit-in-bubble, pola-sumber seperti manual-text-split.test.ts:
// Konva + Svelte runtime → mount unit murah tak bermakna; assert pola kode.
// (ponytail: @testing-library/svelte + jsdom + Konva asli saat butuh
// interaksi nyata; ceiling = dblclick/ketik/Esc beneran, bukan pola string.)
describe("in-bubble edit (§1 canvas)", () => {
  // Sejak FE-5 (3.3): mesin di useBubbleEditor, kanvas pasok deps + wiring.
  it("props onEditTranslation opsional, default mati (Before aman)", () => {
    expect(canvasSrc).toContain("onEditTranslation?: (i: number, value: string) => void");
    expect(canvasSrc).toMatch(/onEditTranslation, onDrawHint, onToolDone \}: Props/);
    // Editor hanya dibuka bila kabel dipasang.
    expect(editorSrc).toMatch(/if \(!deps\.getShowTranslation\(\) \|\| !deps\.getOnEditTranslation\(\)\) return;/);
    expect(canvasSrc).toContain("getOnEditTranslation: () => onEditTranslation,");
  });

  it("dblclick pada group → openEditor, hanya saat showTranslation + kabel", () => {
    expect(canvasSrc).toContain('group.on("dblclick"');
    expect(canvasSrc).toMatch(/if \(showTranslation && onEditTranslation\) \{\s*group\.on\("dblclick"/);
    expect(canvasSrc).toContain("be.openEditor(i)");
  });

  it("openEditor prefill dari tr.translated + posisi × scale", () => {
    expect(editorSrc).toContain('deps.setEditingValue(tr?.translated ?? "")');
    expect(editorSrc).toMatch(/left: b\.x \* s/);
    expect(editorSrc).toMatch(/top: b\.y \* s/);
    expect(editorSrc).toMatch(/width: Math\.max\(60, b\.w \* s\)/);
  });

  it("ketikan lokal: bind:value ke editingValue, overlay disembunyikan saat edit", () => {
    expect(canvasSrc).toContain("bind:value={editingValue}");
    // Satu titik commit: onEditTranslation hanya dipanggil di commitEditor hook.
    const commits = editorSrc.match(/getOnEditTranslation\(\)!\(i,/g) ?? [];
    expect(commits.length).toBe(1);
    // Sejak FE-5 (2.2): guard di appendOverlay, kanvas pasok editing flag.
    expect(overlaySrc).toContain("if (!tr?.translated || o.editing) return;");
    expect(canvasSrc).toContain("editing: be.editingIdx === i,");
  });

  it("commit via blur/Ctrl+Enter → onEditTranslation; Esc batal tanpa tulis", () => {
    expect(canvasSrc).toContain("onblur={() => be.commitEditor()}");
    expect(canvasSrc).toContain("onkeydown={(e) => be.onEditorKey(e)}");
    expect(editorSrc).toMatch(/e\.key === "Enter" && \(e\.ctrlKey \|\| e\.metaKey\)/);
    expect(editorSrc).toMatch(/if \(cur !== deps\.getEditingValue\(\)\) deps\.getOnEditTranslation/);
    const cancel = editorSrc.slice(editorSrc.indexOf("function cancelEditor"), editorSrc.indexOf("function cancelEditor") + 120);
    expect(cancel).not.toContain("onEditTranslation");
  });

  it("pindah bubble saat editor terbuka → commit otomatis", () => {
    expect(canvasSrc).toMatch(/if \(be\.editingIdx !== null && selectedIdx !== null && selectedIdx !== be\.editingIdx\) \{\s*be\.commitEditor\(\);/);
  });

  it("openEditor kunci seleksi baru dulu (anti-yank saat pindah bubble)", () => {
    const fn = editorSrc.slice(editorSrc.indexOf("function openEditor"), editorSrc.indexOf("function openEditor") + 700);
    // Urutan: select baru → commit lama. Commit lama (editTr) tak
    // boleh menarik manualSel balik ke bubble lama.
    expect(fn.indexOf("deps.selectBubble(i);")).toBeGreaterThan(-1);
    expect(fn.indexOf("commitEditor()")).toBeGreaterThan(fn.indexOf("deps.selectBubble(i);"));
  });

  it("commit tanpa perubahan tak tulis (tak tandai edited/antre save)", () => {
    const fn = editorSrc.slice(editorSrc.indexOf("function commitEditor"), editorSrc.indexOf("function commitEditor") + 500);
    expect(fn).toContain("if (cur !== deps.getEditingValue())");
  });

  it("textarea WYSIWYG: font ikut rumus fs overlay, tema ikut needsWhitePatch", () => {
    // Editor in-bubble memakai layout yang SAMA dengan overlay, bukan rumus
    // sendiri (dulu `overlayFontSize`) — kalau tidak, yang diketik tampak
    // beda dari hasilnya.
    expect(editorSrc).not.toContain("const overlayFontSize");
    expect(editorSrc).toContain("fontSize: lay.fontSize * s");
    expect(editorSrc).toContain("lineHeight: lay.lineHeight");
    expect(editorSrc).toContain("dark: !(tr?.needsWhitePatch ?? false)");
    expect(canvasSrc).toContain("min-height:{be.editBox.minH}px");
  });
});

describe("in-bubble edit (§2 panel)", () => {
  it("After pasang onEditTranslation → editTr translated (jalur save existing)", () => {
    expect(panelSrc).toContain('onEditTranslation={(i, v) => editTr(i, "translated", v)}');
    // Before tetap tanpa kabel (hanya satu pemasangan).
    const usages = panelSrc.match(/onEditTranslation=/g) ?? [];
    expect(usages.length).toBe(1);
  });

  it("sidebar 1 kartu terpilih + placeholder bila null", () => {
    expect(panelSrc).toContain("selectedBubble");
    expect(panelSrc).not.toMatch(/\{#each translation\.bubbles as b, i/);
    // Satu kartu via singleCard (fallback: translation 1 bubble) — bukan list.
    // Sejak FE-4: view-model di panel, render di TranslationCard.
    expect(panelSrc).toMatch(/const singleCard = \$derived\(/);
    expect(panelSrc).toContain("card={singleCard}");
    expect(cardSrc).toContain("{#if card}");
    expect(cardSrc).toContain("Klik bubble di After untuk memilih.");
  });

  it("isi kartu = 3 textarea + ↺ + badge + glossary pindah rumah", () => {
    const areas = cardSrc.match(/<textarea/g) ?? [];
    expect(areas.length).toBeGreaterThanOrEqual(3);
    // Sejak fix-edit-stuck: kartu tulis draft, commit via editTr(row, …).
    // Sejak FE-4: kartu panggil prop onDraftInput, panel teruskan ke draftInput.
    expect(cardSrc).toContain('onDraftInput("original"');
    expect(cardSrc).toContain('onDraftInput("reading"');
    expect(cardSrc).toContain('onDraftInput("translated"');
    expect(panelSrc).toContain("onDraftInput={(f, v) => draft.input(f, v)}");
    // Commit via hook deps (behavior dikunci unit useDraftText).
    expect(hookSrc).toContain('deps.commit(row, "translated", translated)');
    expect(cardSrc).toContain("onResetBubble(i)");
    expect(panelSrc).toContain("onResetBubble={(i) => resetBubble(i)}");
    expect(cardSrc).toContain(">edited<");
    expect(cardSrc).toContain("onPressStart(b.original, b.translated)");
  });

  it("label bahasa ikut target, bukan hardcoded JP/ID", () => {
    // Regresi: user ID→EN melihat "Original (JP)" + "Terjemahan (ID)".
    expect(cardSrc).not.toContain("Original (JP)");
    expect(cardSrc).not.toContain(">Terjemahan (ID)<");
    expect(cardSrc).toMatch(/Terjemahan \(\s*\{trgLabel\}\s*\)/);
    expect(panelSrc).toMatch(/TARGET_LANGS/);
    expect(panelSrc).toContain("trgLabel={trgLabel}");
  });
});

describe("sidebar footer identitas", () => {
  it("tampilkan Kuron Studio + versi tauri", () => {
    expect(panelSrc).toContain("getVersion");
    expect(panelSrc).toContain("Kuron Studio {appVersion ?");
  });

  it("footer full-width selebar header (di luar aside)", () => {
    const asideClose = panelSrc.indexOf("</aside>");
    const footerAt = panelSrc.indexOf("<footer");
    expect(asideClose, "aside ada").toBeGreaterThan(-1);
    expect(footerAt, "footer di luar aside").toBeGreaterThan(asideClose);
  });
});
