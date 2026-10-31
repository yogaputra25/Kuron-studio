// Drawing tools kanvas (FE-5 canvas-redraw-split 3.2).
// Pindah verbatim dari CanvasEditor: drag di stage kosong bikin bubble baru.
// Modul murni posisi + emit (tanpa stage global): stage/holder/Konva milik
// panel dan masuk via deps. State drag (`drawing`) closure lokal transien.
import Konva from "konva";
import type { BubbleBox, Tool } from "../contracts/types";
import { clampBubble } from "./bubble";

export interface DrawingDeps {
  getTool: () => Tool;
  getCanDraw: () => boolean;
  getSelectedIdx: () => number | null;
  getStage: () => Konva.Stage | null;
  getOverlay: () => Konva.Layer | null;
  getHolder: () => HTMLDivElement;
  scale: () => number;
  accent: () => string;
  imgW: () => number;
  imgH: () => number;
  setTailForSelected: (tip: [number, number]) => void;
  emitHint: (msg: string) => void;
  /** Finalisasi: push + seleksi + onChange + onToolDone + redraw (milik panel). */
  finishDraw: (nb: BubbleBox) => void;
}

// Batas minimum bubble gambar, dalam px koordinat ASLI (bukan px layar).
// YOLO11 discard < 8px, jadi manual tidak boleh lebih kecil dari itu
// atau hasilnya tidak bisa dipakai translate.
export const MIN_BUBBLE_PX = 8;

interface Drawing {
  x0: number;
  y0: number;
  node: Konva.Rect | Konva.Ellipse | Konva.Line | null;
  pts: [number, number][];
}

export function useDrawingTools(deps: DrawingDeps) {
  let drawing: Drawing | null = null;

  function stagePos(): [number, number] {
    const p = deps.getStage()!.getPointerPosition()!;
    const s = deps.scale();
    return [p.x / s, p.y / s];
  }

  function down() {
    if (!deps.getCanDraw()) {
      // Pane yang tak boleh menggambar tetap bersuara.
      if (deps.getTool() !== "select") deps.emitHint("Gambar hanya di pane Before");
      return;
    }
    if (deps.getTool() === "select") return;
    // Ekor: user mengklik TEPAT pada bubble terpilih (itu titik melekat,
    // lalu drag keluar) atau di mana saja untuk menunjuk ujung ekor. Jadi
    // jangan pernah reject berdasarkan target — cukup butuh bubble terpilih.
    if (deps.getTool() === "tail") {
      if (deps.getSelectedIdx() === null || !deps.getStage()) { deps.emitHint("Pilih bubble dulu, lalu Ekor"); return; }
      const p = deps.getStage()!.getPointerPosition();
      if (!p) return;
      const s = deps.scale();
      deps.setTailForSelected([Math.round(p.x / s), Math.round(p.y / s)]);
      return;
    }
    // Tool gambar aktif = canvas milik gambar. Guard `e.target` dibuang —
    // drag di atas bubble lama tetap menggambar.
    const [x, y] = stagePos();
    drawing = { x0: x, y0: y, node: null, pts: [[x, y]] };
    // Gesture lock: holder overflow-auto bersarang bisa merebut drag sebagai
    // scroll → titik parsial → "terlalu kecil" palsu. Kunci selama drag.
    deps.getHolder().style.overflow = "hidden";
    const s = deps.scale();
    const accent = deps.accent();
    const layer = deps.getOverlay()!;
    const tool = deps.getTool();
    if (tool === "rect") {
      drawing.node = new Konva.Rect({ x: x * s, y: y * s, width: 0, height: 0, stroke: accent, strokeWidth: 2, dash: [6, 4] });
    } else if (tool === "ellipse") {
      drawing.node = new Konva.Ellipse({ x: x * s, y: y * s, radiusX: 0, radiusY: 0, stroke: accent, strokeWidth: 2, dash: [6, 4] });
    } else {
      drawing.node = new Konva.Line({ points: [x * s, y * s], stroke: accent, strokeWidth: 2, closed: false });
    }
    layer.add(drawing.node);
  }

  function move() {
    if (!drawing || !drawing.node) return;
    const [x, y] = stagePos();
    const s = deps.scale();
    const tool = deps.getTool();
    if (tool === "freeform") {
      drawing.pts.push([x, y]);
      (drawing.node as Konva.Line).points(drawing.pts.flatMap(([px, py]) => [px * s, py * s]));
      deps.getOverlay()?.batchDraw();
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
    deps.getOverlay()?.batchDraw();
  }

  /** Buka kunci scroll holder (pasangan lock di down). */
  function unlockScroll() {
    deps.getHolder().style.overflow = "";
  }

  function up() {
    if (!drawing) { unlockScroll(); return; }
    const tool = deps.getTool();
    const pts = drawing.pts;
    drawing.node?.destroy();
    drawing = null;
    unlockScroll();
    if (tool === "freeform") {
      // 6 = 3 titik (polyline minimal), bukan angka acak.
      if (pts.length < 6) { deps.emitHint("Terlalu kecil, drag lebih lebar"); return; }
      const xs = pts.map((p) => p[0]);
      const ys = pts.map((p) => p[1]);
      const bbW = Math.max(...xs) - Math.min(...xs);
      const bbH = Math.max(...ys) - Math.min(...ys);
      if (bbW < MIN_BUBBLE_PX || bbH < MIN_BUBBLE_PX) { deps.emitHint("Terlalu kecil, drag lebih lebar"); return; }
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
      deps.finishDraw(clampBubble(nb, deps.imgW(), deps.imgH()));
    } else {
      // `pts` hanya berisi 2 titik setelah mousemove. Klik tanpa drag
      // (atau drag yang tidak sampai ke move) menyisakan 1 titik, dan
      // destructure di sini akan throw — jadi guard dulu.
      if (pts.length < 2) { deps.emitHint("Terlalu kecil, drag lebih lebar"); return; }
      const [[x0, y0], [x1, y1]] = pts;
      const w = Math.abs(x1 - x0);
      const h = Math.abs(y1 - y0);
      if (w < MIN_BUBBLE_PX || h < MIN_BUBBLE_PX) { deps.emitHint("Terlalu kecil, drag lebih lebar"); return; }
      deps.finishDraw(
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
          deps.imgW(),
          deps.imgH(),
        ),
      );
    }
  }

  return { down, move, up, unlockScroll };
}

export type DrawingTools = ReturnType<typeof useDrawingTools>;
