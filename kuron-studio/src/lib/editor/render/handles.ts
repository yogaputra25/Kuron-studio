// Handle resize + vertex Konva (FE-5 canvas-redraw-split 3.1).
// Pindah verbatim dari redraw(): resize handle kanan-bawah (rect/ellipse)
// + vertex polygon freeform. Guard tampil di dalam tiap fungsi. Live-update
// visual tanpa sentuh state; state ditulis sekali di dragend via commit.
// Stroke putih ring = konstanta kontras fungsional (lihat themeColors.ts).
import Konva from "konva";
import type { BubbleBox, Tool } from "../../contracts/types";
import { clampBubble } from "../bubble";
import type { ThemeColors } from "../themeColors";

export interface HandlesOpts {
  group: Konva.Group;
  bubble: BubbleBox;
  index: number;
  scale: number;
  selected: boolean;
  editable: boolean;
  tool: Tool;
  theme: ThemeColors;
  imgW: number;
  imgH: number;
  canMoveGroups: () => boolean;
  commit: (index: number, nb: BubbleBox) => void;
  batchDraw: () => void;
}

/** Resize handle kanan-bawah (rect/ellipse, editable, HANYA terpilih). */
export function attachResizeHandle(o: HandlesOpts): void {
  const { group, bubble: b, index: i, scale: s, selected, editable, tool, theme: P } = o;
  if (!(!b.shape && editable && selected && tool === "select")) return;
  const handle = new Konva.Circle({
    x: b.w * s,
    y: b.h * s,
    radius: 11,
    fill: P.accent,
    stroke: "#fff",
    strokeWidth: 2,
    draggable: true,
  });
  // R6 lanjutan: kunci sejak HOVER (mousedown terlambat — Konva sudah
  // memilih group sebagai drag-target). Buka saat mouseup/dragend/leave.
  handle.on("mouseenter", () => {
    group.draggable(false);
  });
  handle.on("mouseleave", () => {
    group.draggable(o.canMoveGroups());
  });
  handle.on("click tap", (e) => {
    // Jangan teruskan ke group: klik handle tak boleh redraw
    // (redraw membuat ulang node handle = drag target hilang + klik-2x).
    e.cancelBubble = true;
  });
  handle.on("dragstart", () => {
    // Repair R6: group draggable menelan drag anak — kunci selama
    // handle di-drag, kembalikan saat selesai.
    group.draggable(false);
  });
  // Pending lokal (bukan state): tulis bubbles[i] tiap move memicu
  // effect sync + parent round-trip yang me-recreate node drag.
  let pendW = b.w;
  let pendH = b.h;
  handle.on("dragmove", () => {
    pendW = Math.max(8, handle.x() / s);
    pendH = Math.max(8, handle.y() / s);
    const cl = clampBubble({ ...b, w: pendW, h: pendH }, o.imgW, o.imgH);
    pendW = cl.w;
    pendH = cl.h;
    // Live-update visual saja (tanpa sentuh state).
    const rect = group.findOne("Rect");
    rect?.setAttrs({ width: pendW * s, height: pendH * s });
    const ell = group.findOne("Ellipse");
    ell?.setAttrs({ x: (pendW * s) / 2, y: (pendH * s) / 2, radiusX: (pendW * s) / 2, radiusY: (pendH * s) / 2 });
    handle.position({ x: pendW * s, y: pendH * s });
    o.batchDraw();
  });
  handle.on("dragend", () => {
    group.draggable(o.canMoveGroups());
    // Satu-satunya penulis state: commit + onChange sekali.
    o.commit(i, clampBubble({ ...b, w: pendW, h: pendH }, o.imgW, o.imgH));
  });
  handle.on("mousedown touchstart", (e) => {
    e.cancelBubble = true;
    // Kunci SEBELUM drag diputuskan (dragstart bisa kalah dari parent).
    group.draggable(false);
  });
  handle.on("mouseup", () => {
    // Klik-tanpa-drag tak memicu dragend — buka di sini juga.
    group.draggable(o.canMoveGroups());
  });
  group.add(handle);
}

/** Vertex polygon freeform: tiap vertex titik draggable; bbox = DERIVASI vertex. */
export function attachVertexHandles(o: HandlesOpts): void {
  const { group, bubble: b, index: i, scale: s, selected, editable, tool, theme: P } = o;
  if (!(b.shape && b.shape.length >= 3 && editable && selected && tool === "select")) return;
  // State lokal cur* (bukan closure b) agar drag beruntun tak meleset
  // setelah bbox bergeser.
  let curShape: [number, number][] = b.shape.map(([qx, qy]) => [qx, qy]);
  let curX = b.x;
  let curY = b.y;
  const vNodes: Konva.Circle[] = [];
  const syncVertexNodes = () => {
    vNodes.forEach((node, j) => {
      node.position({ x: (curShape[j][0] - curX) * s, y: (curShape[j][1] - curY) * s });
    });
    const poly = group.findOne("Line");
    poly?.setAttrs({ points: curShape.flatMap(([qx, qy]) => [(qx - curX) * s, (qy - curY) * s]) });
    group.position({ x: curX * s, y: curY * s });
  };
  curShape.forEach(([px, py], k) => {
    const v = new Konva.Circle({
      x: (px - curX) * s,
      y: (py - curY) * s,
      radius: 10,
      fill: P.accent,
      stroke: "#fff",
      strokeWidth: 2,
      draggable: true,
    });
    vNodes.push(v);
    v.on("mouseenter", () => {
      group.draggable(false);
    });
    v.on("mouseleave", () => {
      group.draggable(o.canMoveGroups());
    });
    v.on("click tap", (e) => {
      e.cancelBubble = true;
    });
    v.on("dragstart", () => {
      group.draggable(false);
    });
    v.on("dragmove", () => {
      const nx = Math.round(curX + v.x() / s);
      const ny = Math.round(curY + v.y() / s);
      curShape = curShape.map(([qx, qy], j) =>
        j === k
          ? [
              Math.min(Math.max(0, nx), o.imgW - 1),
              Math.min(Math.max(0, ny), o.imgH - 1),
            ] as [number, number]
          : [qx, qy] as [number, number],
      );
      const xs = curShape.map((p) => p[0]);
      const ys = curShape.map((p) => p[1]);
      curX = Math.min(...xs);
      curY = Math.min(...ys);
      // Pending lokal saja — state ditulis sekali di dragend.
      syncVertexNodes();
      o.batchDraw();
    });
    v.on("dragend", () => {
      group.draggable(o.canMoveGroups());
      const xs = curShape.map((p) => p[0]);
      const ys = curShape.map((p) => p[1]);
      const fx = Math.min(...xs);
      const fy = Math.min(...ys);
      const fw = Math.max(8, Math.max(...xs) - fx);
      const fh = Math.max(8, Math.max(...ys) - fy);
      o.commit(i, clampBubble({ ...b, shape: [...curShape], x: fx, y: fy, w: fw, h: fh }, o.imgW, o.imgH));
    });
    v.on("mousedown touchstart", (e) => {
      e.cancelBubble = true;
      group.draggable(false);
    });
    v.on("mouseup", () => {
      group.draggable(o.canMoveGroups());
    });
    group.add(v);
  });
}
