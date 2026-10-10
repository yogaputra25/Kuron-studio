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
    // After tidak pasang (hanya satu pemasangan = Before).
    const usages = panelSrc.match(/onDrawHint=/g) ?? [];
    expect(usages.length).toBe(1);
    expect(panelSrc).toContain('drawHint = "";');
  });

  it("3 titik tolak bersuara (bukan diam)", () => {
    expect(canvasSrc).toContain('onDrawHint?.("Klik area kosong untuk menggambar")');
    expect(canvasSrc).toContain('onDrawHint?.("Pilih bubble dulu, lalu Ekor")');
    expect(canvasSrc).toContain('onDrawHint?.("Terlalu kecil, drag lebih lebar")');
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
