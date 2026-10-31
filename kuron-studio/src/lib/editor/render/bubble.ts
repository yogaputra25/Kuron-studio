// Node bubble Konva (FE-5 canvas-redraw-split 2.1).
// Pindah verbatim dari redraw(): group draggable + shape
// (freeform/ellipse/rect) + tail + chip label. Overlay + handle + gestur
// tetap di CanvasEditor.
import Konva from "konva";
import type { BubbleBox } from "../../contracts/types";
import type { ThemeColors } from "../themeColors";

export interface BubbleNodeOpts {
  /** Indeks loop (untuk nama node `bubble-<i>`). */
  index: number;
  bubble: BubbleBox;
  selected: boolean;
  /** Faktor scale px halaman -> px layar. */
  scale: number;
  /** Label urutan baca (sudah diformat pemanggil). */
  label: string;
  theme: ThemeColors;
}

/** Bangun satu node bubble lengkap (belum ditempel ke layer). */
export function renderBubbleNode(o: BubbleNodeOpts): Konva.Group {
  const { bubble: b, index, selected, scale: s, label, theme: P } = o;
  const group = new Konva.Group({
    x: b.x * s,
    y: b.y * s,
    draggable: true,
    name: `bubble-${index}`,
  });

  if (b.shape && b.shape.length >= 3) {
    group.add(
      new Konva.Line({
        // Relatif ke origin group (sudah di-offset b.x/b.y) — konsisten
        // dengan vertex Circle + tail; absolut = double-offset (H2).
        points: b.shape.flatMap(([px, py]) => [(px - b.x) * s, (py - b.y) * s]),
        closed: true,
        stroke: selected ? P.accent : P.cyan,
        strokeWidth: 2,
        fill: P.cyanRgba,
      }),
    );
  } else if (b.kind === "ellipse") {
    group.add(
      new Konva.Ellipse({
        x: (b.w * s) / 2,
        y: (b.h * s) / 2,
        radiusX: (b.w * s) / 2,
        radiusY: (b.h * s) / 2,
        stroke: selected ? P.accent : P.cyan,
        strokeWidth: 2,
        fill: P.cyanRgba,
      }),
    );
  } else {
    group.add(
      new Konva.Rect({
        width: b.w * s,
        height: b.h * s,
        stroke: selected ? P.accent : P.cyan,
        strokeWidth: 2,
        fill: P.cyanRgba,
        cornerRadius: 6,
      }),
    );
  }

  // Ekor: garis base→tip (relatif ke group origin).
  if (b.tail && b.tail.length >= 2) {
    group.add(
      new Konva.Line({
        points: b.tail.flatMap(([px, py]) => [(px - b.x) * s, (py - b.y) * s]),
        stroke: selected ? P.accent : P.cyan,
        strokeWidth: 2,
      }),
    );
  }

  // Chip nomor urutan baca.
  const chip = new Konva.Group({ x: -10, y: -10 });
  chip.add(new Konva.Circle({ radius: 11, fill: P.accent }));
  chip.add(
    new Konva.Text({
      text: label,
      fontSize: 13,
      fontStyle: "bold",
      fill: P.accentFg,
      offsetX: label.length * 4,
      offsetY: 6.5,
    }),
  );
  group.add(chip);

  return group;
}
