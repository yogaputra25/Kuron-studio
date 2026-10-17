import { describe, expect, it } from "vitest";
import panelSrc from "./EditorPanel.svelte?raw";
import canvasSrc from "./CanvasEditor.svelte?raw";

// Kontrak change draw-tools-feedback: tool tak-pernah-tampak-rusak.
// Pola-sumber seperti edit-in-bubble.test.ts (Konva runtime tak di-mount).
describe("draw tools feedback (§1)", () => {
  it("toolbar draw disabled saat After + alasan terlihat", () => {
    expect(panelSrc).toContain("readOnlyMode");
    expect(panelSrc).toContain("Gambar hanya di Before");
    expect(panelSrc).toContain("disabled={off}");
  });

  it("callback onDrawHint opsional, Before pasang + clear saat sukses", () => {
    expect(canvasSrc).toContain("onDrawHint?: (msg: string) => void");
    expect(panelSrc).toContain("onDrawHint={(m) => { drawHint = m; }}");
    // fix-draw-tools B: After kini IKUT memasang (suara di tiap pane).
    const usages = panelSrc.match(/onDrawHint=/g) ?? [];
    expect(usages.length).toBe(2);
    expect(panelSrc).toContain('drawHint = "";');
  });

  it("titik tolak bersuara (bukan diam)", () => {
    // fix-draw-tools A1: penolakan "klik area kosong" dibuang — drag di atas
    // bubble lama kini menggambar. Sisa titik tolak + titik baru (B).
    expect(canvasSrc).not.toContain("Klik area kosong untuk menggambar");
    expect(canvasSrc).toContain('onDrawHint?.("Pilih bubble dulu, lalu Ekor")');
    expect(canvasSrc).toContain('onDrawHint?.("Terlalu kecil, drag lebih lebar")');
    expect(canvasSrc).toContain('onDrawHint?.("Gambar hanya di pane Before")');
  });

  it("hint ringan terpisah dari error (role status, bukan alert)", () => {
    expect(panelSrc).toContain('let drawHint = $state("");');
    expect(panelSrc).toContain('role="status"');
  });

  it("syarat ukuran + geometri tak berubah", () => {
    expect(canvasSrc).toContain("MIN_BUBBLE_PX");
    expect(canvasSrc).toContain("if (pts.length < 6)");
  });

  it("gesture lock: kunci scroll saat drag, buka saat up/destroy", () => {
    expect(canvasSrc).toContain('holder.style.overflow = "hidden"');
    expect(canvasSrc).toContain("function unlockScroll()");
    expect(canvasSrc).toContain('holder.style.overflow = ""');
  });
});

describe("bubble resize full (§1.5)", () => {
  it("sinkron geometri di jalur save (bukan kosmetik)", () => {
    expect(panelSrc).toContain("syncGeometryToTranslation(bubbles, translation.bubbles");
    const save = panelSrc.slice(panelSrc.indexOf("async function saveTranslationEdits"), panelSrc.indexOf("async function saveTranslationEdits") + 600);
    expect(save).toContain("syncGeometryToTranslation");
  });

  it("vertex polygon draggable + bbox derivasi + tolak degenerat", () => {
    // Repair R1: state lokal curShape (bukan b.shape.forEach) agar drag
    // beruntun tak meleset setelah bbox bergeser (stale-closure fix).
    expect(canvasSrc).toContain("let curShape:");
    expect(canvasSrc).toContain("curShape.forEach(");
    expect(canvasSrc).toContain("Math.min(...xs)");
  });

  it("kontrol angka + skala-dari-pusat + clamp di sidebar", () => {
    expect(panelSrc).toContain("setBubbleGeometry");
    expect(panelSrc).toContain("scaleSelectedBubble");
    expect(panelSrc).toContain("Geometri (px asli)");
  });

  it("whitePatch dihitung ulang dari geometri baru (port Rust)", () => {
    expect(panelSrc).toContain("needsWhitePatch");
  });
});

describe("kepemilikan canvas oleh mode tool (fix-draw-tools)", () => {
  it("A1: mode gambar mematikan geser, seleksi, dan handle", () => {
    expect(canvasSrc).toContain('const canMoveGroups = (): boolean => editable && tool === "select";');
    expect(canvasSrc).toContain("group.draggable(canMoveGroups());");
    expect(canvasSrc).toContain('if (!b.shape && editable && i === selectedIdx && tool === "select") {');
    expect(canvasSrc).toContain('if (tool !== "select") return;');
  });

  it("A1: guard e.target dibuang dari jalur gambar", () => {
    expect(canvasSrc).not.toContain("e.target !== stage");
    expect(canvasSrc).toContain("tool gambar aktif = canvas milik gambar");
  });

  it("C: canDraw default literal false, bukan = editable", () => {
    expect(canvasSrc).toContain("canDraw = false");
    expect(canvasSrc).not.toContain("canDraw = editable");
  });

  it("B: After menerima tool asli + hint; Before canDraw true", () => {
    expect(panelSrc).toContain("tool={tool}");
    expect(panelSrc).toContain("canDraw={true}");
    expect(panelSrc).toContain("canDraw={false}");
    expect(canvasSrc).toContain('onDrawHint?.("Gambar hanya di pane Before")');
  });

  it("D: onToolDone dipanggil setelah bubble jadi", () => {
    expect(canvasSrc).toContain("onToolDone?.();");
    expect(panelSrc).toContain('onToolDone={() => setTool("select")}');
  });
});
