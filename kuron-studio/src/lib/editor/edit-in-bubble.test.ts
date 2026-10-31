import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import canvasSrc from "./CanvasEditor.svelte?raw";

// Kontrak change edit-in-bubble, pola-sumber seperti manual-text-split.test.ts:
// Konva + Svelte runtime → mount unit murah tak bermakna; assert pola kode.
// (ponytail: @testing-library/svelte + jsdom + Konva asli saat butuh
// interaksi nyata; ceiling = dblclick/ketik/Esc beneran, bukan pola string.)
describe("in-bubble edit (§1 canvas)", () => {
  it("props onEditTranslation opsional, default mati (Before aman)", () => {
    expect(canvasSrc).toContain("onEditTranslation?: (i: number, value: string) => void");
    expect(canvasSrc).toMatch(/onEditTranslation, onDrawHint, onToolDone \}: Props/);
    // Editor hanya dibuka bila kabel dipasang.
    expect(canvasSrc).toMatch(/if \(!showTranslation \|\| !onEditTranslation\) return;/);
  });

  it("dblclick pada group → openEditor, hanya saat showTranslation + kabel", () => {
    expect(canvasSrc).toContain('group.on("dblclick"');
    expect(canvasSrc).toMatch(/if \(showTranslation && onEditTranslation\) \{\s*group\.on\("dblclick"/);
    expect(canvasSrc).toContain("openEditor(i)");
  });

  it("openEditor prefill dari tr.translated + posisi × scale", () => {
    expect(canvasSrc).toContain("editingValue = tr?.translated ?? \"\"");
    expect(canvasSrc).toMatch(/left: b\.x \* s/);
    expect(canvasSrc).toMatch(/top: b\.y \* s/);
    expect(canvasSrc).toMatch(/width: Math\.max\(60, b\.w \* s\)/);
  });

  it("ketikan lokal: bind:value ke editingValue, overlay disembunyikan saat edit", () => {
    expect(canvasSrc).toContain("bind:value={editingValue}");
    // Satu titik commit: onEditTranslation hanya dipanggil di commitEditor.
    const commits = canvasSrc.match(/onEditTranslation\(i, editingValue\)/g) ?? [];
    expect(commits.length).toBe(1);
    expect(canvasSrc).toContain("if (tr?.translated && editingIdx !== i) {");
  });

  it("commit via blur/Ctrl+Enter → onEditTranslation; Esc batal tanpa tulis", () => {
    expect(canvasSrc).toContain("onblur={commitEditor}");
    expect(canvasSrc).toMatch(/e\.key === "Enter" && \(e\.ctrlKey \|\| e\.metaKey\)/);
    expect(canvasSrc).toContain("onEditTranslation(i, editingValue)");
    const cancel = canvasSrc.slice(canvasSrc.indexOf("function cancelEditor"), canvasSrc.indexOf("function cancelEditor") + 120);
    expect(cancel).not.toContain("onEditTranslation");
  });

  it("pindah bubble saat editor terbuka → commit otomatis", () => {
    expect(canvasSrc).toMatch(/if \(editingIdx !== null && selectedIdx !== null && selectedIdx !== editingIdx\) \{\s*commitEditor\(\);/);
  });

  it("openEditor kunci seleksi baru dulu (anti-yank saat pindah bubble)", () => {
    const fn = canvasSrc.slice(canvasSrc.indexOf("function openEditor"), canvasSrc.indexOf("function openEditor") + 700);
    // Urutan: select baru → forward → commit lama. Commit lama (editTr) tak
    // boleh menarik manualSel balik ke bubble lama.
    expect(fn.indexOf("selectedIdx = i;")).toBeGreaterThan(-1);
    expect(fn.indexOf("onSelectForward(i)")).toBeGreaterThan(fn.indexOf("selectedIdx = i;"));
    expect(fn.indexOf("commitEditor()")).toBeGreaterThan(fn.indexOf("onSelectForward(i)"));
  });

  it("commit tanpa perubahan tak tulis (tak tandai edited/antre save)", () => {
    const fn = canvasSrc.slice(canvasSrc.indexOf("function commitEditor"), canvasSrc.indexOf("function commitEditor") + 500);
    expect(fn).toContain("if (cur !== editingValue) onEditTranslation(i, editingValue);");
  });

  it("textarea WYSIWYG: font ikut rumus fs overlay, tema ikut needsWhitePatch", () => {
    // Editor in-bubble memakai layout yang SAMA dengan overlay, bukan rumus
    // sendiri (dulu `overlayFontSize`) — kalau tidak, yang diketik tampak
    // beda dari hasilnya.
    expect(canvasSrc).not.toContain("const overlayFontSize");
    expect(canvasSrc).toContain("fontSize: lay.fontSize * s");
    expect(canvasSrc).toContain("lineHeight: lay.lineHeight");
    expect(canvasSrc).toContain("dark: !(tr?.needsWhitePatch ?? false)");
    expect(canvasSrc).toContain("min-height:{editBox.minH}px");
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
    expect(panelSrc).toMatch(/singleCard = selectedBubble && selectedRow/);
    expect(panelSrc).toContain("{#if singleCard}");
    expect(panelSrc).toContain("Klik bubble di After untuk memilih.");
  });

  it("isi kartu = 3 textarea + ↺ + badge + glossary pindah rumah", () => {
    const areas = panelSrc.match(/<textarea/g) ?? [];
    expect(areas.length).toBeGreaterThanOrEqual(3);
    // Sejak fix-edit-stuck: kartu tulis draft, commit via editTr(row, …).
    expect(panelSrc).toContain('draftInput("original"');
    expect(panelSrc).toContain('draftInput("reading"');
    expect(panelSrc).toContain('draftInput("translated"');
    expect(panelSrc).toContain('editTr(row, "translated"');
    expect(panelSrc).toContain("resetBubble(i)");
    expect(panelSrc).toContain(">edited<");
    expect(panelSrc).toContain("pressStart(b.original, b.translated)");
  });

  it("label bahasa ikut target, bukan hardcoded JP/ID", () => {
    // Regresi: user ID→EN melihat "Original (JP)" + "Terjemahan (ID)".
    expect(panelSrc).not.toContain("Original (JP)");
    expect(panelSrc).not.toContain(">Terjemahan (ID)<");
    expect(panelSrc).toMatch(/Terjemahan \(\s*\{trgLabel\}\s*\)/);
    expect(panelSrc).toMatch(/TARGET_LANGS/);
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
