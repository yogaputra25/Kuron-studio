<!--
  Canvas editor: Konva stage di atas page image.
  Model koordinat: SEMUA state dalam px original image; scale = stageW / imgW
  hanya saat render/event. Bubble rect/ellipse/freeform + tail + chip RTL/LTR.
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Konva from "konva";
  import type { BubbleBox, BubbleTranslation, ReadingDirection, Tool } from "../contracts/types";
  import { chipNumbers, clampBubble } from "./bubble";
  import { readThemeColors, type ThemeColors } from "./themeColors";
  import { useBackgroundLoader } from "./backgroundLoader.svelte";
  import { renderBubbleNode } from "./render/bubble";
  import { appendOverlay } from "./render/overlay";
  import { attachResizeHandle, attachVertexHandles } from "./render/handles";
  import { useDrawingTools } from "./drawingTools";
  import { useBubbleEditor } from "./useBubbleEditor.svelte";
  /** Shape absolut (px halaman) -> relatif bubble untuk layout teks. */
  const relShape = (b: BubbleBox): Array<[number, number]> | null =>
    b.shape && b.shape.length >= 3 ? b.shape.map(([px, py]) => [px - b.x, py - b.y]) : null;
  import Button from "../ui/Button.svelte";

  interface Props {
    imageUrl: string;
    fallbackUrl?: string;
    imgW: number;
    imgH: number;
    initial: BubbleBox[];
    readingDir: ReadingDirection;
    tool: Tool;
    onChange: (bubbles: BubbleBox[]) => void;
    translations?: BubbleTranslation[];
    showTranslation?: boolean;
    editable?: boolean;
    /** Izin menggambar bubble BARU (Before-only). Default literal `false`;
     *  pane yang boleh menggambar mengisinya eksplisit. Jangan pakai
     *  `= editable`: compiler Svelte menerjemahkan fallback antar-prop jadi
     *  fungsi, bukan boolean (fix-draw-tools C). */
    canDraw?: boolean;
    /** Dipanggil setelah satu bubble berhasil digambar — panel memakainya
     *  untuk mengembalikan tool ke "Pilih" (fix-draw-tools D). */
    onToolDone?: () => void;
    onSelectForward?: (i: number) => void;
    externalSelected?: number | null;
    onEditTranslation?: (i: number, value: string) => void;
    onDrawHint?: (msg: string) => void;
  }

  let { imageUrl, fallbackUrl = "", imgW, imgH, initial, readingDir, tool, onChange, translations = [], showTranslation = false, editable = true, canDraw = false, onSelectForward, externalSelected = null, onEditTranslation, onDrawHint, onToolDone }: Props = $props();

  /** A1 (fix-draw-tools): geser/resize bubble hanya di mode Pilih. Saat tool
   *  gambar aktif, canvas milik gambar — bubble tak boleh mencuri gestur. */
  const canMoveGroups = (): boolean => editable && tool === "select";

  let holder: HTMLDivElement;
  let stage: Konva.Stage | null = null;
  let overlayLayer: Konva.Layer | null = null;
  let bgImage: Konva.Image | null = null;
  let selectedIdx = $state<number | null>(null);

  // In-bubble editor milik useBubbleEditor (FE-5 3.3). Nilai textarea +
  // DOM ref tetap di sini (`bind:value`/`bind:this` butuh lvalue lokal).
  let editingValue = $state("");
  let editorEl = $state<HTMLTextAreaElement | null>(null);
  const be = useBubbleEditor({
    getShowTranslation: () => showTranslation,
    getOnEditTranslation: () => onEditTranslation,
    getBubble: (i) => bubbles[i],
    findTranslation: (i) => translations.find((t) => t.index === i),
    selectBubble: (i) => { selectedIdx = i; onSelectForward?.(i); },
    scale: () => scale(),
    getRelShape: (b) => relShape(b),
    focusEditor: () => {
      requestAnimationFrame(() => {
        editorEl?.focus();
        editorEl?.select();
      });
    },
    getEditingValue: () => editingValue,
    setEditingValue: (v) => { editingValue = v; },
  });
  /** Commit dari luar (ganti tab): pola sama seperti deleteSelected. */
  export function commitEditor() { be.commitEditor(); }

  $effect(() => {
    translations;
    showTranslation;
    externalSelected;
    // fix-draw-tools A1: handle/vertex/draggable ikut mode tool, jadi ganti
    // tool harus me-render ulang (bukan hanya saat seleksi/terjemahan berubah).
    tool;
    if (externalSelected !== null) selectedIdx = externalSelected;
    // Editor terbuka tapi bubble pindah seleksi → commit otomatis lalu tutup.
    if (be.editingIdx !== null && selectedIdx !== null && selectedIdx !== be.editingIdx) {
      be.commitEditor();
    }
    redraw();
  });

  // Background loader milik useBackgroundLoader (FE-5 1.2): mesin state +
  // timeout + retry; stage/layer/Konva tetap milik komponen (via deps).
  const bg = useBackgroundLoader({
    getStage: () => stage,
    getOverlay: () => overlayLayer,
    getBg: () => bgImage,
    setBg: (im) => { bgImage = im; },
    scale: () => scale(),
    imgW: () => imgW,
    imgH: () => imgH,
    getFallbackUrl: () => fallbackUrl,
    redraw: () => redraw(),
  });

  // Salinan kerja — disinkron dari `initial` via effect di bawah (remount per page
  // via {#key}, jadi sinkronisasi awal + update parent pasca-detect aman).
  let bubbles = $state<BubbleBox[]>([]);

  // Sinkron dari parent hanya saat konten benar-benar berubah eksternal
  // (hasil Detect/Save). Edit lokal memanggil onChange → parent mengembalikan
  // konten identik, jadi guard JSON ini mencegah reset selectedIdx tiap ketik.
  $effect(() => {
    const incoming = $state.snapshot(initial);
    if (JSON.stringify(incoming) !== JSON.stringify(bubbles)) {
      bubbles = incoming;
      selectedIdx = null;
      redraw();
    }
  });

  // URL gambar tiba async (App fetch get_image_preview setelah mount panel).
  // Stage harus tercipta ulang / image di-swap saat URL berubah, kalau tidak
  // body editor tetap hitam walau bubble sudah kedetect (overlay di atas bg kosong).
  $effect(() => {
    if (!imageUrl) return;
    bg.load(imageUrl);
  });

  const scale = (): number => {
    const w = holder?.clientWidth || imgW;
    return w / Math.max(1, imgW);
  };

  /**
   * Warna kanvas via themeColors (FE-5 1.1): satu-satunya baca `--ks-*`.
   * Dibaca per redraw supaya ganti tema ikut: menukar tema hanya menukar
   * nilai var, bukan hex yang di-hardcode di sini.
   */
  const paint = (): ThemeColors => readThemeColors(holder);

  export function redraw() {
    if (!stage || !overlayLayer) return;
    const s = scale();
    const P = paint();
    stage.width(imgW * s);
    stage.height(imgH * s);
    // Background ikut scale baru (diset sekali saat load): tanpa ini gambar
    // diam sementara overlay pindah (split-brain) usai pane berganti lebar.
    if (bgImage) {
      bgImage.size({ width: imgW * s, height: imgH * s });
      bgImage.getLayer()?.batchDraw();
    }
    overlayLayer.destroyChildren();
    const nums = chipNumbers(bubbles, readingDir);

    bubbles.forEach((b, i) => {
      const group = renderBubbleNode({
        index: i,
        bubble: b,
        selected: i === selectedIdx,
        scale: s,
        label: String(nums.get(i) ?? i + 1),
        theme: P,
      });

      // Overlay terjemahan (append-only) via render/overlay (FE-5 2.2).
      appendOverlay({
        group,
        bubble: b,
        showTranslation,
        translation: translations.find((t) => t.index === i),
        editing: be.editingIdx === i,
        scale: s,
        shapeRel: relShape(b),
      });

      group.on("click tap", (e) => {
        e.cancelBubble = true;
        // A1 (fix-draw-tools): selama tool gambar aktif, klik tak menyeleksi —
        // canvas milik gambar, bukan milik bubble.
        if (tool !== "select") return;
        // Klik ulang bubble yang sama = no-op (redraw me-recreate node drag).
        if (selectedIdx === i) return;
        selectedIdx = i;
        // D2: seleksi SELALU diteruskan ke parent — pane mana pun boleh mengisi
        // sidebar & seleksi bersama (dulu hanya saat read-only).
        if (onSelectForward) onSelectForward(i);
        redraw();
      });

      // Double-klik → editor teks di badan bubble (After + onEditTranslation saja).
      if (showTranslation && onEditTranslation) {
        group.on("dblclick", (e) => {
          e.cancelBubble = true;
          be.openEditor(i);
        });
      }

      group.draggable(canMoveGroups());

      group.on("dragend", () => {
        if (!editable) { redraw(); return; }
        const nx = Math.round(group.x() / s);
        const ny = Math.round(group.y() / s);
        // Repair R3: shape absolut ikut bergeser sebesar delta; tanpa ini
        // polygon "ketinggalan" dan redraw berikutnya mismatch.
        const dx = nx - b.x;
        const dy = ny - b.y;
        const shape = b.shape ? b.shape.map(([px, py]) => [px + dx, py + dy] as [number, number]) : b.shape;
        const nb = { ...b, x: nx, y: ny, shape };
        commit(i, clampBubble(nb, imgW, imgH));
      });

      // Resize + vertex via render/handles (FE-5 3.1): guard tampil di modul.
      const handleOpts = {
        group,
        bubble: b,
        index: i,
        scale: s,
        selected: i === selectedIdx,
        editable,
        tool,
        theme: P,
        imgW,
        imgH,
        canMoveGroups: () => canMoveGroups(),
        commit: (j: number, nb: BubbleBox) => commit(j, nb),
        batchDraw: () => overlayLayer?.batchDraw(),
      };
      attachResizeHandle(handleOpts);
      attachVertexHandles(handleOpts);

      overlayLayer!.add(group);
    });
    overlayLayer.batchDraw();
  }


  function commit(i: number, nb: BubbleBox) {
    bubbles[i] = nb;
    onChange($state.snapshot(bubbles));
    redraw();
  }

  export function deleteSelected(): boolean {
    if (selectedIdx === null) return false;
    bubbles.splice(selectedIdx, 1);
    selectedIdx = null;
    onChange($state.snapshot(bubbles));
    redraw();
    return true;
  }

  /**
   * Ekor untuk bubble terpilih. Titik melekat = titik TEPAT yang diklik user
   * (bukan tengah bubble) — kalau dipatok ke tengah, garis ekor selalu
   * keluar dari dalam gelembung dan tidak pernah terlihat menempel di tepi.
   * Kalau user klik jauh dari bubble, dekati tepi terdekat.
   */
  export function setTailForSelected(tipOriginal: [number, number]) {
    if (selectedIdx === null) return;
    const b = bubbles[selectedIdx];
    const [px, py] = tipOriginal;
    const insideX = px >= b.x && px <= b.x + b.w;
    const insideY = py >= b.y && py <= b.y + b.h;
    const anchor: [number, number] = insideX && insideY
      ? [px, py]
      : [
          Math.min(Math.max(px, b.x), b.x + b.w),
          Math.min(Math.max(py, b.y), b.y + b.h),
        ];
    const tail = [
      anchor,
      [Math.min(Math.max(tipOriginal[0], 0), imgW), Math.min(Math.max(tipOriginal[1], 0), imgH)],
    ] as [[number, number], [number, number]];
    bubbles[selectedIdx] = { ...b, tail };
    onChange($state.snapshot(bubbles));
    redraw();
  }

  /** Punya ekor? — dipakai toolbar buat disable tombol Ekor dengan jujur. */
  export function hasTailSelected(): boolean {
    return selectedIdx !== null && !!bubbles[selectedIdx]?.tail;
  }

  // Drawing tools milik useDrawingTools (FE-5 3.2): posisi + emit, tanpa stage global.
  const dt = useDrawingTools({
    getTool: () => tool,
    getCanDraw: () => canDraw,
    getSelectedIdx: () => selectedIdx,
    getStage: () => stage,
    getOverlay: () => overlayLayer,
    getHolder: () => holder,
    scale: () => scale(),
    accent: () => paint().accent,
    imgW: () => imgW,
    imgH: () => imgH,
    setTailForSelected: (tip) => setTailForSelected(tip),
    emitHint: (m) => onDrawHint?.(m),
    finishDraw: (nb) => {
      bubbles.push(nb);
      selectedIdx = bubbles.length - 1;
      onChange($state.snapshot(bubbles));
      // D (fix-draw-tools): sekali gambar, sekali tool — balik ke Pilih supaya
      // gestur berikutnya menyunting, bukan menggambar bubble tak sengaja.
      onToolDone?.();
      redraw();
    },
  });

  // (Loader pindah ke useBackgroundLoader — FE-5 1.2.)

  onMount(() => {
    const s = holder.clientWidth / Math.max(1, imgW);
    // sharp-preview D2: buffer canvas ikut resolusi layar (tajam di HiDPI/
    // scale-Windows); fallback 1 bila devicePixelRatio tak ada (SSR/test).
    const dpr = typeof window !== "undefined" && window.devicePixelRatio ? window.devicePixelRatio : 1;
    stage = new Konva.Stage({ container: holder, width: imgW * s, height: imgH * s, pixelRatio: dpr });
    overlayLayer = new Konva.Layer();
    stage.add(overlayLayer);
    stage.on("mousedown touchstart", () => dt.down());
    stage.on("mousemove touchmove", () => dt.move());
    stage.on("mouseup touchend", () => dt.up());
    bg.consumePending(imageUrl);
    redraw();
    // Webfont (--font-sans) tiba belakangan; canvas tak redraw otomatis
    // saat font siap, jadi jadwalkan sekali (aman bila API tak ada).
    void document.fonts?.ready?.then(() => {
      redraw();
    });
  });

  onDestroy(() => {
    bg.invalidate(); // invalidasi onload/onerror/timeout telat
    dt.unlockScroll();
    stage?.destroy();
    stage = null;
  });
</script>

<!-- Latar canvas tetap gelap di kedua tema: gambar manga di atas netral gelap
     jauh lebih terbaca daripada di atas kertas putih, dan area di luar
     gambar tidak akan terlihat salah warna di light mode. -->
<div class="relative w-full">
<div bind:this={holder} class="w-full cursor-crosshair overflow-auto rounded-md border border-line bg-black"></div>
{#if be.editingIdx !== null && be.editBox}
  <textarea
    bind:this={editorEl}
    rows="3"
    placeholder="ketik terjemahan"
    class="absolute z-10 rounded px-1 py-0.5 text-center outline-none ring-2 ring-accent {be.editBox.dark ? 'bg-black/70 text-white' : 'bg-surface text-ink'}"
    style="left:{be.editBox.left}px; top:{be.editBox.top}px; width:{be.editBox.width}px; min-height:{be.editBox.minH}px; font-size:{be.editBox.fontSize}px; font-family:{be.editBox.fontFamily}; line-height:{be.editBox.lineHeight};"
    bind:value={editingValue}
    onblur={() => be.commitEditor()}
    onkeydown={(e) => be.onEditorKey(e)}
  ></textarea>
{/if}
</div>
{#if !bg.imgReady && !bg.imgError}
  <p class="mt-1 text-[11px] text-ink-3">Memuat gambar…</p>
{:else if bg.imgError}
  <p class="mt-1 text-[11px] text-danger">
    {bg.imgError}
    <Button variant="default" size="sm" class="ml-2" onclick={() => bg.retryLoad()}>Coba lagi</Button>
  </p>
{/if}
