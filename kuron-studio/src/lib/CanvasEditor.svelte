<!--
  Canvas editor: Konva stage di atas page image.
  Model koordinat: SEMUA state dalam px original image; scale = stageW / imgW
  hanya saat render/event. Bubble rect/ellipse/freeform + tail + chip RTL/LTR.
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Konva from "konva";
  import type { BubbleBox, BubbleTranslation, ReadingDirection, Tool } from "../lib/types";
  import { chipNumbers, clampBubble } from "../lib/bubble";

  interface Props {
    imageUrl: string;
    imgW: number;
    imgH: number;
    initial: BubbleBox[];
    readingDir: ReadingDirection;
    tool: Tool;
    onChange: (bubbles: BubbleBox[]) => void;
    translations?: BubbleTranslation[];
    showTranslation?: boolean;
  }

  let { imageUrl, imgW, imgH, initial, readingDir, tool, onChange, translations = [], showTranslation = false }: Props = $props();

  let holder: HTMLDivElement;
  let stage: Konva.Stage | null = null;
  let overlayLayer: Konva.Layer | null = null;
  let selectedIdx = $state<number | null>(null);

  // Salinan kerja — disinkron dari `initial` via effect di bawah (remount per page
  // via {#key}, jadi sinkronisasi awal + update parent pasca-detect aman).
  let bubbles = $state<BubbleBox[]>([]);

  // Sinkron dari parent hanya saat konten benar-benar berubah eksternal
  // (hasil Detect/Save). Edit lokal memanggil onChange → parent mengembalikan
  // konten identik, jadi guard JSON ini mencegah reset selectedIdx tiap ketik.
  $effect(() => {
    const incoming = structuredClone(initial);
    if (JSON.stringify(incoming) !== JSON.stringify(bubbles)) {
      bubbles = incoming;
      selectedIdx = null;
      redraw();
    }
  });

  $effect(() => {
    translations;
    showTranslation;
    redraw();
  });

  const scale = (): number => {
    const w = holder?.clientWidth || imgW;
    return w / Math.max(1, imgW);
  };

  function redraw() {
    if (!stage || !overlayLayer) return;
    const s = scale();
    stage.width(imgW * s);
    stage.height(imgH * s);
    overlayLayer.destroyChildren();
    const nums = chipNumbers(bubbles, readingDir);

    bubbles.forEach((b, i) => {
      const group = new Konva.Group({
        x: b.x * s,
        y: b.y * s,
        draggable: true,
        name: `bubble-${i}`,
      });

      if (b.shape && b.shape.length >= 3) {
        group.add(
          new Konva.Line({
            points: b.shape.flatMap(([px, py]) => [px * s, py * s]),
            closed: true,
            stroke: i === selectedIdx ? "#10b981" : "#38bdf8",
            strokeWidth: 2,
            fill: "rgba(56,189,248,0.12)",
          }),
        );
      } else if (b.kind === "ellipse") {
        group.add(
          new Konva.Ellipse({
            x: (b.w * s) / 2,
            y: (b.h * s) / 2,
            radiusX: (b.w * s) / 2,
            radiusY: (b.h * s) / 2,
            stroke: i === selectedIdx ? "#10b981" : "#38bdf8",
            strokeWidth: 2,
            fill: "rgba(56,189,248,0.12)",
          }),
        );
      } else {
        group.add(
          new Konva.Rect({
            width: b.w * s,
            height: b.h * s,
            stroke: i === selectedIdx ? "#10b981" : "#38bdf8",
            strokeWidth: 2,
            fill: "rgba(56,189,248,0.12)",
            cornerRadius: 6,
          }),
        );
      }

      // Ekor: garis base→tip (relatif ke group origin).
      if (b.tail && b.tail.length >= 2) {
        group.add(
          new Konva.Line({
            points: b.tail.flatMap(([px, py]) => [(px - b.x) * s, (py - b.y) * s]),
            stroke: i === selectedIdx ? "#10b981" : "#38bdf8",
            strokeWidth: 2,
          }),
        );
      }

      // Chip nomor urutan baca.
      const chip = new Konva.Group({ x: -10, y: -10 });
      chip.add(new Konva.Circle({ radius: 11, fill: "#10b981" }));
      chip.add(
        new Konva.Text({
          text: String(nums.get(i) ?? i + 1),
          fontSize: 13,
          fontStyle: "bold",
          fill: "#fff",
          offsetX: String(nums.get(i) ?? i + 1).length * 4,
          offsetY: 6.5,
        }),
      );
      group.add(chip);

      // Overlay terjemahan (append-only; jangan refactor redraw di atas).
      if (showTranslation) {
        const tr = translations.find((t) => t.index === i);
        if (tr?.translated) {
          const fs = Math.max(10, Math.min(18, (b.w * s) / Math.max(8, tr.translated.length / 2)));
          const txt = new Konva.Text({
            x: 2, y: 2, width: Math.max(10, b.w * s - 4),
            text: tr.translated, fontSize: fs, fill: "#fff",
            align: "center", listening: false,
          });
          const bgR = new Konva.Rect({
            x: 0, y: 0, width: b.w * s, height: Math.max(b.h * s, txt.height() + 6),
            fill: tr.needsWhitePatch ? "#ffffff" : "rgba(0,0,0,0.65)",
            cornerRadius: 4, listening: false,
          });
          if (tr.needsWhitePatch) txt.fill("#111");
          group.add(bgR);
          txt.moveToTop();
          group.add(txt);
        }
      }

      group.on("click tap", (e) => {
        e.cancelBubble = true;
        selectedIdx = i;
        redraw();
      });

      group.on("dragend", () => {
        const nb = { ...b, x: group.x() / s, y: group.y() / s };
        commit(i, clampBubble(nb, imgW, imgH));
      });

      // Resize handle kanan-bawah (rect/ellipse saja).
      if (!b.shape) {
        const handle = new Konva.Circle({
          x: b.w * s,
          y: b.h * s,
          radius: 6,
          fill: "#10b981",
          draggable: true,
        });
        handle.on("dragmove", () => {
          const nw = Math.max(8, handle.x() / s);
          const nh = Math.max(8, handle.y() / s);
          bubbles[i] = clampBubble({ ...b, w: nw, h: nh }, imgW, imgH);
          redrawKeep(handle);
        });
        handle.on("dragend", () => {
          onChange(structuredClone(bubbles));
        });
        handle.on("mousedown touchstart", (e) => e.cancelBubble = true);
        group.add(handle);
      }

      overlayLayer!.add(group);
    });
    overlayLayer.batchDraw();
  }

  /** Redraw tanpa melepas handle yang sedang di-drag. */
  function redrawKeep(activeHandle: Konva.Circle) {
    void activeHandle;
    // Sederhana: batchDraw layer yang sama (node handle tetap hidup karena
    // redraw() tidak dipanggil di tengah dragmove — hanya update rect attrs).
    const s = scale();
    const node = overlayLayer?.findOne(`.bubble-${selectedIdx ?? -1}`);
    void node;
    void s;
    overlayLayer?.batchDraw();
  }

  function commit(i: number, nb: BubbleBox) {
    bubbles[i] = nb;
    onChange(structuredClone(bubbles));
    redraw();
  }

  export function deleteSelected() {
    if (selectedIdx === null) return;
    bubbles.splice(selectedIdx, 1);
    selectedIdx = null;
    onChange(structuredClone(bubbles));
    redraw();
  }

  export function setTailForSelected(tipOriginal: [number, number]) {
    if (selectedIdx === null) return;
    const b = bubbles[selectedIdx];
    const cx = b.x + b.w / 2;
    const cy = b.y + b.h / 2;
    bubbles[selectedIdx] = { ...b, tail: [[Math.round(cx), Math.round(cy)], tipOriginal] };
    onChange(structuredClone(bubbles));
    redraw();
  }

  // --- Drawing tools: drag di stage kosong bikin bubble baru ---
  let drawing: { x0: number; y0: number; node: Konva.Rect | Konva.Ellipse | Konva.Line | null; pts: [number, number][] } | null = null;

  function stagePos(): [number, number] {
    const p = stage!.getPointerPosition()!;
    const s = scale();
    return [p.x / s, p.y / s];
  }

  function onStageDown(e: Konva.KonvaEventObject<MouseEvent | TouchEvent>) {
    if (tool === "select") return;
    if (e.target !== stage) return;
    const [x, y] = stagePos();
    if (tool === "tail") {
      if (selectedIdx !== null) setTailForSelected([Math.round(x), Math.round(y)]);
      return;
    }
    drawing = { x0: x, y0: y, node: null, pts: [[x, y]] };
    const s = scale();
    const layer = overlayLayer!;
    if (tool === "rect") {
      drawing.node = new Konva.Rect({ x: x * s, y: y * s, width: 0, height: 0, stroke: "#10b981", strokeWidth: 2, dash: [6, 4] });
    } else if (tool === "ellipse") {
      drawing.node = new Konva.Ellipse({ x: x * s, y: y * s, radiusX: 0, radiusY: 0, stroke: "#10b981", strokeWidth: 2, dash: [6, 4] });
    } else {
      drawing.node = new Konva.Line({ points: [x * s, y * s], stroke: "#10b981", strokeWidth: 2, closed: false });
    }
    layer.add(drawing.node);
  }

  function onStageMove() {
    if (!drawing || !drawing.node) return;
    const [x, y] = stagePos();
    const s = scale();
    if (tool === "freeform") {
      drawing.pts.push([x, y]);
      (drawing.node as Konva.Line).points(drawing.pts.flatMap(([px, py]) => [px * s, py * s]));
      overlayLayer?.batchDraw();
      return;
    }
    const x0 = Math.min(drawing.x0, x);
    const y0 = Math.min(drawing.y0, y);
    const w = Math.abs(x - drawing.x0);
    const h = Math.abs(y - drawing.y0);
    if (tool === "rect") {
      const r = drawing.node as Konva.Rect;
      r.setAttrs({ x: x0 * s, y: y0 * s, width: w * s, height: h * s });
    } else {
      const el = drawing.node as Konva.Ellipse;
      el.setAttrs({ x: (x0 + w / 2) * s, y: (y0 + h / 2) * s, radiusX: (w / 2) * s, radiusY: (h / 2) * s });
    }
    drawing.pts = [[x0, y0], [x0 + w, y0 + h]];
    overlayLayer?.batchDraw();
  }

  function onStageUp() {
    if (!drawing) return;
    const pts = drawing.pts;
    drawing.node?.destroy();
    drawing = null;
    if (tool === "freeform") {
      if (pts.length < 6) return;
      const xs = pts.map((p) => p[0]);
      const ys = pts.map((p) => p[1]);
      const nb: BubbleBox = {
        x: Math.floor(Math.min(...xs)),
        y: Math.floor(Math.min(...ys)),
        w: Math.ceil(Math.max(...xs) - Math.min(...xs)),
        h: Math.ceil(Math.max(...ys) - Math.min(...ys)),
        confidence: 1,
        shape: pts.map(([px, py]) => [Math.round(px), Math.round(py)] as [number, number]),
        kind: "freeform",
        tail: null,
      };
      bubbles.push(clampBubble(nb, imgW, imgH));
    } else {
      const [[x0, y0], [x1, y1]] = pts;
      const w = Math.abs(x1 - x0);
      const h = Math.abs(y1 - y0);
      if (w < 8 || h < 8) return;
      bubbles.push(
        clampBubble(
          {
            x: Math.round(Math.min(x0, x1)),
            y: Math.round(Math.min(y0, y1)),
            w: Math.round(w),
            h: Math.round(h),
            confidence: 1,
            shape: null,
            kind: tool === "ellipse" ? "ellipse" : "rect",
            tail: null,
          },
          imgW,
          imgH,
        ),
      );
    }
    selectedIdx = bubbles.length - 1;
    onChange(structuredClone(bubbles));
    redraw();
  }

  onMount(() => {
    const img = new Image();
    img.onload = () => {
      const s = holder.clientWidth / Math.max(1, imgW);
      stage = new Konva.Stage({ container: holder, width: imgW * s, height: imgH * s });
      const bg = new Konva.Layer();
      const kimg = new Konva.Image({ image: img, width: imgW * s, height: imgH * s, listening: false });
      bg.add(kimg);
      stage.add(bg);
      overlayLayer = new Konva.Layer();
      stage.add(overlayLayer);
      stage.on("mousedown touchstart", onStageDown);
      stage.on("mousemove touchmove", onStageMove);
      stage.on("mouseup touchend", onStageUp);
      redraw();
    };
    img.src = imageUrl;
  });

  onDestroy(() => {
    stage?.destroy();
    stage = null;
  });
</script>

<div bind:this={holder} class="w-full cursor-crosshair overflow-auto rounded border border-zinc-800 bg-black"></div>
