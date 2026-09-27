import { describe, expect, it } from "vitest";
import src from "./CanvasEditor.svelte?raw";
import panelSrc from "./EditorPanel.svelte?raw";

// Kontrak D3 change fix-editor-blank-canvas (§2), diverifikasi statis dari
// sumber komponen. Alasan: komponen terikat Konva + DOM + Svelte runtime
// ($effect/onMount) sehingga mount unit murah tidak bermakna; test palsu
// dengan mock Konva hanya menguji mock itu sendiri. Test ini assert pada pola
// kode yang menjamin kontrak — bila pola hilang, test gagal dan memaksa
// perbaikan sadar, bukan silent regress.
//
// (ponytail: upgrade ke component-test sungguhan — @testing-library/svelte +
// jsdom + Konva asli — saat repo butuh interaksi canvas; ceiling = render
// pixel, bukan pola string.)
describe("canvas-editor blank-canvas contract (D3)", () => {
  it("onMount membuat stage + overlay tanpa menunggu gambar", () => {
    const mount = src.slice(src.indexOf("onMount(() => {"));
    expect(mount).toMatch(/new Konva\.Stage/);
    expect(mount).toMatch(/overlayLayer = new Konva\.Layer/);
    // Tidak ada guard imageUrl yang menunda pembuatan stage.
    const mountHead = mount.slice(0, mount.indexOf("overlayLayer = new Konva.Layer"));
    expect(mountHead).not.toContain("if (imageUrl)");
  });

  it("$effect swap background saat imageUrl tiba + guard requestedUrl", () => {
    expect(src).toMatch(/\$effect\(\(\) => \{\s*if \(!imageUrl\) return;/);
    expect(src).toContain("if (requestedUrl === imageUrl) return;");
    expect(src).toContain("requestedUrl = imageUrl;");
    expect(src).toContain("loadBackground(imageUrl);");
  });

  it("load basi diabaikan di onload dan onerror", () => {
    expect(src).toContain("requestedUrl !== url) return;");
  });

  it("hint imgReady/imgError dirender di bawah canvas", () => {
    expect(src).toContain("Memuat gambar…");
    expect(src).toContain("{imgError}");
  });

  it("redraw() tidak bergantung pada bgImage non-null", () => {
    const redraw = src.slice(src.indexOf("function redraw()"), src.indexOf("/** Redraw tanpa melepas"));
    expect(redraw).toMatch(/if \(!stage \|\| !overlayLayer\) return;/);
    expect(redraw).not.toContain("bgImage.image");
    expect(redraw).not.toContain("if (!bgImage)");
  });

  it("loadBackground mengantre via pendingUrl bila stage belum ada, onMount mengonsumsinya", () => {
    expect(src).toContain("pendingUrl = url;");
    expect(src).toContain("} else if (pendingUrl) {");
    // Kedua cabang onMount membersihkan antrean agar tak ada sisa basi.
    expect(src).toContain("pendingUrl = null;\n      requestedUrl = imageUrl;");
  });

  it("callback async aman setelah unmount (tidak menyentuh stage yang sudah null)", () => {
    const onload = src.slice(src.indexOf("img.onload = () => {"));
    expect(onload).toContain("if (!stage || !overlayLayer) return;");
    const onerror = src.slice(src.indexOf("img.onerror = () => {"));
    expect(onerror).toContain("if (!stage) return;");
  });
});

// Kontrak fix-preview-hang §5: structuredClone() throw DataCloneError pada
// Svelte 5 $state proxy (non-kosong) — pakai $state.snapshot (rune bawaan,
// tanpa import, plain deep copy). Pola-sumber, gaya sama di atas.
describe("snapshot copy (fix-preview-hang §5)", () => {
  it("CanvasEditor: effect sync pakai $state.snapshot, bukan structuredClone", () => {
    const effect = src.slice(src.indexOf("$effect(() => {"), src.indexOf("loadBackground(imageUrl);"));
    expect(effect).toContain("const incoming = $state.snapshot(initial);");
    expect(effect).not.toContain("structuredClone");
  });

  it("CanvasEditor: semua onChange kirim snapshot (clone aman proxy non-kosong)", () => {
    const calls = src.match(/onChange\(\$state\.snapshot\(bubbles\)\)/g) ?? [];
    expect(calls.length).toBe(5);
    expect(src).not.toContain("structuredClone");
  });

  it("EditorPanel: syncFromPage + restore translation pakai $state.snapshot", () => {
    expect(panelSrc).toContain("const incoming = $state.snapshot(page.bubbles ?? []);");
    expect(panelSrc).toContain("translation = $state.snapshot(page.translation) ?? null;");
    expect(panelSrc).not.toContain("structuredClone");
  });
});

// Kontrak fix-preview-hang §6: hapus bubble = persist via save() — guard
// syncFromPage JANGAN diubah (tetap lindungi reset saat ngetik). Pola-sumber,
// gaya sama di atas.
describe("delete autosave (fix-preview-hang §6)", () => {
  it("deleteSelected return boolean (true bila menghapus, false bila no-op)", () => {
    expect(src).toContain("export function deleteSelected(): boolean {");
    expect(src).toContain("if (selectedIdx === null) return false;");
    const fn = src.slice(src.indexOf("export function deleteSelected"));
    expect(fn).toContain("return true;");
    expect(fn).toContain("onChange($state.snapshot(bubbles))");
  });

  it("tombol hapus sidebar: splice lalu save() + disabled saat saving", () => {
    const btn = panelSrc.slice(panelSrc.indexOf(">hapus</button>") - 400, panelSrc.indexOf(">hapus</button>"));
    expect(btn).toContain("bubbles.splice(i, 1);");
    expect(btn).toContain("void save();");
    expect(btn).toContain("disabled={saving}");
  });

  it("onKey Delete: abaikan saat saving, save() hanya bila deleteSelected true", () => {
    const key = panelSrc.slice(panelSrc.indexOf("function onKey"), panelSrc.indexOf("async function saveTranslationEdits"));
    expect(key).toContain("if (saving) return;");
    expect(key).toContain("if (editorRef?.deleteSelected()) void save();");
  });

  it("kontrak tak tersentuh: onChange umum tetap lokal+dirty, guard syncFromPage utuh", () => {
    expect(panelSrc).toMatch(/onChange=\{\(nb\) => \{\s*bubbles = nb;\s*dirty = true;\s*\}\}/);
    expect(panelSrc).toContain("if (syncedFor !== page.id || JSON.stringify(incoming) !== JSON.stringify(bubbles)) {");
  });
});
