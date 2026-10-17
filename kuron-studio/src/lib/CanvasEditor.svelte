<!--
  Canvas editor: Konva stage di atas page image.
  Model koordinat: SEMUA state dalam px original image; scale = stageW / imgW
  hanya saat render/event. Bubble rect/ellipse/freeform + tail + chip RTL/LTR.
-->
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Konva from "konva";
  import type { BubbleBox, BubbleTranslation, ReadingDirection, Tool } from "../lib/types";
  import { chipNumbers, clampBubble, fitFontSize } from "../lib/bubble";
  import Button from "./ui/Button.svelte";

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

  // In-bubble editor: textarea HTML melayang di atas stage (Konva tak punya
  // text-editing). Ketikan ditahan lokal; commit sekali via onEditTranslation.
  let editingIdx = $state<number | null>(null);
  let editingValue = $state("");
  let editBox = $state<{ left: number; top: number; width: number; minH: number; fontSize: number; dark: boolean } | null>(null);
  let editorEl = $state<HTMLTextAreaElement | null>(null);

  const overlayFontSize = (b: BubbleBox, text: string, s: number): number =>
    Math.max(10, Math.min(18, (b.w * s) / Math.max(8, text.length / 2)));

  function openEditor(i: number) {
    if (!showTranslation || !onEditTranslation) return;
    const b = bubbles[i];
    const tr = translations.find((t) => t.index === i);
    if (!b) return;
    // Kunci seleksi ke bubble baru DULU sebelum commit editor lama, supaya
    // commit(editTr) tak menarik manualSel balik ke bubble lama (yank).
    selectedIdx = i;
    if (onSelectForward) onSelectForward(i);
    commitEditor();
    const s = scale();
    editingIdx = i;
    editingValue = tr?.translated ?? "";
    editBox = {
      left: b.x * s,
      top: b.y * s,
      width: Math.max(60, b.w * s),
      minH: Math.max(24, b.h * s),
      fontSize: overlayFontSize(b, editingValue || "…", s),
      dark: !(tr?.needsWhitePatch ?? false),
    };
    requestAnimationFrame(() => {
      editorEl?.focus();
      editorEl?.select();
    });
  }

  /** Commit dari luar (ganti tab): pola sama seperti deleteSelected. */
  export function commitEditor() {
    if (editingIdx === null || !onEditTranslation) {
      editingIdx = null;
      return;
    }
    const i = editingIdx;
    const cur = translations.find((t) => t.index === i)?.translated ?? "";
    editingIdx = null;
    editBox = null;
    // Nilai tak berubah → tutup saja, jangan tandai edited / antre save.
    if (cur !== editingValue) onEditTranslation(i, editingValue);
  }

  function cancelEditor() {
    editingIdx = null;
    editBox = null;
  }

  function onEditorKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      cancelEditor();
    } else if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      commitEditor();
    }
  }

  $effect(() => {
    translations;
    showTranslation;
    externalSelected;
    // fix-draw-tools A1: handle/vertex/draggable ikut mode tool, jadi ganti
    // tool harus me-render ulang (bukan hanya saat seleksi/terjemahan berubah).
    tool;
    if (externalSelected !== null) selectedIdx = externalSelected;
    // Editor terbuka tapi bubble pindah seleksi → commit otomatis lalu tutup.
    if (editingIdx !== null && selectedIdx !== null && selectedIdx !== editingIdx) {
      commitEditor();
    }
    redraw();
  });

  const OVERLAY_MIN_FS = 7;
  const OVERLAY_MAX_FS = 28;

  // Font overlay diambil dari token tipografi supaya mengikuti --font-sans;
  // di-memo karena getComputedStyle tidak murah dan nilainya konstan per sesi.
  let fontCache = "";
  function overlayFontFamily(): string {
    if (!fontCache) {
      fontCache =
        (typeof document !== "undefined"
          ? getComputedStyle(document.documentElement).getPropertyValue("--font-sans").trim()
          : "") || '"Inter Variable", sans-serif';
    }
    return fontCache;
  }
  // Gambar datang belakangan (App fetch async setelah mount) — lacak request
  // terakhir agar load basi tidak menimpa, dan tandai siap/gagal untuk hint.
  let requestedUrl = "";
  let imgReady = $state(false);
  let imgError = $state("");

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
    if (requestedUrl === imageUrl) return;
    requestedUrl = imageUrl;
    loadBackground(imageUrl);
  });

  const scale = (): number => {
    const w = holder?.clientWidth || imgW;
    return w / Math.max(1, imgW);
  };

  /**
   * Konva menggambar ke <canvas>, bukan DOM — dia tidak bisa pakai class
   * Tailwind, jadi warna harus dibaca dari CSS var yang sama dengan token
   * app.css. Dibaca per redraw supaya ganti tema ikut: menukar tema hanya
   * menukar nilai var, bukan hex yang di-hardcode di sini.
   */
  const paint = (): { accent: string; cyan: string; accentRgba: string; cyanRgba: string } => {
    const cs = getComputedStyle(holder ?? document.body);
    const v = (n: string, fallback: string) => cs.getPropertyValue(n).trim() || fallback;
    const hexToRgba = (hex: string, a: number) => {
      const h = hex.replace("#", "");
      const full = h.length === 3 ? [...h].map((c) => c + c).join("") : h;
      const [r, g, b] = [0, 2, 4].map((i) => parseInt(full.slice(i, i + 2), 16));
      return Number.isNaN(r) ? `rgba(53,231,245,${a})` : `rgba(${r},${g},${b},${a})`;
    };
    const accent = v("--ks-accent", "#ff6b5a");
    const cyan = v("--ks-cyan", "#35e7f5");
    return { accent, cyan, accentRgba: hexToRgba(accent, 0.18), cyanRgba: hexToRgba(cyan, 0.12) };
  };

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
      const group = new Konva.Group({
        x: b.x * s,
        y: b.y * s,
        draggable: true,
        name: `bubble-${i}`,
      });

      if (b.shape && b.shape.length >= 3) {
        group.add(
          new Konva.Line({
            // Relatif ke origin group (sudah di-offset b.x/b.y) — konsisten
            // dengan vertex Circle + tail; absolut = double-offset (H2).
            points: b.shape.flatMap(([px, py]) => [(px - b.x) * s, (py - b.y) * s]),
            closed: true,
            stroke: i === selectedIdx ? P.accent : P.cyan,
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
            stroke: i === selectedIdx ? P.accent : P.cyan,
            strokeWidth: 2,
            fill: P.cyanRgba,
          }),
        );
      } else {
        group.add(
          new Konva.Rect({
            width: b.w * s,
            height: b.h * s,
            stroke: i === selectedIdx ? P.accent : P.cyan,
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
            stroke: i === selectedIdx ? P.accent : P.cyan,
            strokeWidth: 2,
          }),
        );
      }

      // Chip nomor urutan baca.
      const chip = new Konva.Group({ x: -10, y: -10 });
      chip.add(new Konva.Circle({ radius: 11, fill: P.accent }));
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
        if (tr?.translated && editingIdx !== i) {
          const pad = 4;
          const boxW = Math.max(12, b.w * s - pad * 2);
          const boxH = Math.max(12, b.h * s - pad * 2);

          // Ukur dengan tinggi OTOMATIS dulu. Konva hanya menghitung tinggi
          // blok teks lewat getHeight() selama attrs.height belum di-set;
          // begitu height dikunci, getHeight() mengembalikan tinggi kotak.
          const txt = new Konva.Text({
            x: pad,
            y: pad,
            width: boxW,
            text: tr.translated,
            fontSize: OVERLAY_MAX_FS,
            fontFamily: overlayFontFamily(),
            fontStyle: "600",
            lineHeight: 1.25,
            align: "center",
            fill: "#fff",
            listening: false,
          });

          // Auto-fit: perbesar-kecilkan sampai teks pas muat. Konva tidak
          // pernah melakukan ini sendiri.
          txt.fontSize(
            fitFontSize(boxH, (fs) => {
              txt.fontSize(fs);
              return txt.getHeight();
            }, { min: OVERLAY_MIN_FS, max: OVERLAY_MAX_FS }),
          );

          // Kunci tinggi kotak SETELAH ukuran pas. Tanpa baris ini
          // verticalAlign "middle" adalah no-op: _getTextTop() memakai
          // getHeight() yang selama auto justru = tinggi teks, jadi ruang
          // bebas selalu 0 dan teks nempel di tepi atas bubble.
          txt.height(boxH);
          txt.verticalAlign("middle");

          const bgR = new Konva.Rect({
            x: pad,
            y: pad,
            width: boxW,
            height: boxH,
            fill: tr.needsWhitePatch ? "#ffffff" : "rgba(0,0,0,0.78)",
            cornerRadius: 4,
            listening: false,
          });
          if (tr.needsWhitePatch) txt.fill("#111");
          group.add(bgR);
          txt.moveToTop();
          group.add(txt);
        }
      }

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
          openEditor(i);
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

      // Resize handle kanan-bawah (rect/ellipse, editable, HANYA terpilih — D3)
      // agar pane After yang penuh overlay teks tak jadi hutan titik.
      if (!b.shape && editable && i === selectedIdx && tool === "select") {
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
          group.draggable(canMoveGroups());
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
          const cl = clampBubble({ ...b, w: pendW, h: pendH }, imgW, imgH);
          pendW = cl.w;
          pendH = cl.h;
          // Live-update visual saja (tanpa sentuh state).
          const rect = group.findOne("Rect");
          rect?.setAttrs({ width: pendW * s, height: pendH * s });
          const ell = group.findOne("Ellipse");
          ell?.setAttrs({ x: (pendW * s) / 2, y: (pendH * s) / 2, radiusX: (pendW * s) / 2, radiusY: (pendH * s) / 2 });
          handle.position({ x: pendW * s, y: pendH * s });
          overlayLayer?.batchDraw();
        });
        handle.on("dragend", () => {
          group.draggable(canMoveGroups());
          // Satu-satunya penulis state: commit + onChange sekali.
          commit(i, clampBubble({ ...b, w: pendW, h: pendH }, imgW, imgH));
        });
        handle.on("mousedown touchstart", (e) => {
          e.cancelBubble = true;
          // Kunci SEBELUM drag diputuskan (dragstart bisa kalah dari parent).
          group.draggable(false);
        });
        handle.on("mouseup", () => {
          // Klik-tanpa-drag tak memicu dragend — buka di sini juga.
          group.draggable(canMoveGroups());
        });
        group.add(handle);
      }

      // Vertex polygon freeform (bubble-resize-full D2 + repair R1): tiap
      // vertex jadi titik draggable; bbox = DERIVASI vertex (satu arah).
      // State lokal cur* (bukan closure b) agar drag beruntun tak meleset
      // setelah bbox bergeser.
      if (b.shape && b.shape.length >= 3 && editable && i === selectedIdx && tool === "select") {
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
            group.draggable(canMoveGroups());
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
                    Math.min(Math.max(0, nx), imgW - 1),
                    Math.min(Math.max(0, ny), imgH - 1),
                  ] as [number, number]
                : [qx, qy] as [number, number],
            );
            const xs = curShape.map((p) => p[0]);
            const ys = curShape.map((p) => p[1]);
            curX = Math.min(...xs);
            curY = Math.min(...ys);
            // Pending lokal saja — state ditulis sekali di dragend.
            syncVertexNodes();
            overlayLayer?.batchDraw();
          });
          v.on("dragend", () => {
            group.draggable(canMoveGroups());
            const xs = curShape.map((p) => p[0]);
            const ys = curShape.map((p) => p[1]);
            const fx = Math.min(...xs);
            const fy = Math.min(...ys);
            const fw = Math.max(8, Math.max(...xs) - fx);
            const fh = Math.max(8, Math.max(...ys) - fy);
            commit(i, clampBubble({ ...b, shape: [...curShape], x: fx, y: fy, w: fw, h: fh }, imgW, imgH));
          });
          v.on("mousedown touchstart", (e) => {
            e.cancelBubble = true;
            group.draggable(false);
          });
          v.on("mouseup", () => {
            group.draggable(canMoveGroups());
          });
          group.add(v);
        });
      }

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

  // --- Drawing tools: drag di stage kosong bikin bubble baru ---
  // Batas minimum bubble gambar, dalam px koordinat ASLI (bukan px layar).
  // Domein allerdings: YOLO11 discard < 8px, jadi manual tidak boleh lebih
  // kecil dari itu atau hasilnya tidak bisa dipakai translate.
  const MIN_BUBBLE_PX = 8;

  let drawing: { x0: number; y0: number; node: Konva.Rect | Konva.Ellipse | Konva.Line | null; pts: [number, number][] } | null = null;

  function stagePos(): [number, number] {
    const p = stage!.getPointerPosition()!;
    const s = scale();
    return [p.x / s, p.y / s];
  }

  function onStageDown() {
    if (!canDraw) {
      // B (fix-draw-tools): pane yang tak boleh menggambar tetap bersuara.
      if (tool !== "select") onDrawHint?.("Gambar hanya di pane Before");
      return;
    }
    if (tool === "select") return;
    // Ekor: user mengklik TEPAT pada bubble terpilih (itu titik melekat,
    // lalu drag keluar) atau di mana saja untuk menunjuk ujung ekor. Jadi
    // jangan pernah reject berdasarkan target — cukup butuh bubble terpilih.
    if (tool === "tail") {
      if (selectedIdx === null || !stage) { onDrawHint?.("Pilih bubble dulu, lalu Ekor"); return; }
      const p = stage.getPointerPosition();
      if (!p) return;
      const s = scale();
      setTailForSelected([Math.round(p.x / s), Math.round(p.y / s)]);
      return;
    }
    // A1 (fix-draw-tools): tool gambar aktif = canvas milik gambar. Guard
    // `e.target` dibuang — drag di atas bubble lama tetap menggambar
    // (sebelumnya seluruh badan bubble = zona mati untuk menggambar).
    const [x, y] = stagePos();
    drawing = { x0: x, y0: y, node: null, pts: [[x, y]] };
    // Gesture lock (draw-tools-feedback D4): holder overflow-auto bersarang
    // bisa merebut drag sebagai scroll → titik parsial → "terlalu kecil"
    // palsu. Kunci selama drag, kembalikan saat up (unlock juga di bawah).
    holder.style.overflow = "hidden";
    const s = scale();
    const P = paint();
    const layer = overlayLayer!;
    if (tool === "rect") {
      drawing.node = new Konva.Rect({ x: x * s, y: y * s, width: 0, height: 0, stroke: P.accent, strokeWidth: 2, dash: [6, 4] });
    } else if (tool === "ellipse") {
      drawing.node = new Konva.Ellipse({ x: x * s, y: y * s, radiusX: 0, radiusY: 0, stroke: P.accent, strokeWidth: 2, dash: [6, 4] });
    } else {
      drawing.node = new Konva.Line({ points: [x * s, y * s], stroke: P.accent, strokeWidth: 2, closed: false });
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

  /** Buka kunci scroll holder (pasangan lock di onStageDown). */
  function unlockScroll() {
    holder.style.overflow = "";
  }

  function onStageUp() {
    if (!drawing) { unlockScroll(); return; }
    const pts = drawing.pts;
    drawing.node?.destroy();
    drawing = null;
    unlockScroll();
    if (tool === "freeform") {
      // 6 = 3 titik (polyline minimal), bukan angka acak.
      if (pts.length < 6) { onDrawHint?.("Terlalu kecil, drag lebih lebar"); return; }
      const xs = pts.map((p) => p[0]);
      const ys = pts.map((p) => p[1]);
      const bbW = Math.max(...xs) - Math.min(...xs);
      const bbH = Math.max(...ys) - Math.min(...ys);
      if (bbW < MIN_BUBBLE_PX || bbH < MIN_BUBBLE_PX) { onDrawHint?.("Terlalu kecil, drag lebih lebar"); return; }
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
      // `pts` hanya berisi 2 titik setelah mousemove. Klik tanpa drag
      // (atau drag yang tidak sampai ke move) menyisakan 1 titik, dan
      // destructure di sini akan throw — jadi guard dulu.
      if (pts.length < 2) { onDrawHint?.("Terlalu kecil, drag lebih lebar"); return; }
      const [[x0, y0], [x1, y1]] = pts;
      const w = Math.abs(x1 - x0);
      const h = Math.abs(y1 - y0);
      if (w < MIN_BUBBLE_PX || h < MIN_BUBBLE_PX) { onDrawHint?.("Terlalu kecil, drag lebih lebar"); return; }
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
    onChange($state.snapshot(bubbles));
    // D (fix-draw-tools): sekali gambar, sekali tool — balik ke Pilih supaya
    // gestur berikutnya menyunting, bukan menggambar bubble tak sengaja.
    onToolDone?.();
    redraw();
  }

  // Antrian URL bila $effect imageUrl jalan sebelum onMount (urutan tidak
  // dijamin); onMount mengonsumsinya di kedua cabang agar tak ada sisa basi.
  let pendingUrl: string | null = null;

  // fix-preview-hang §3: tiap request bg harus terminasi ≤15s (siap/gagal+retry).
  // Token per-request — onload/onerror/timeout mana duluan menang, sisanya basi.
  const BG_LOAD_TIMEOUT_MS = 15_000;
  let bgToken = 0;
  let bgTimer: ReturnType<typeof setTimeout> | null = null;

  function clearBgTimer() {
    if (bgTimer) {
      clearTimeout(bgTimer);
      bgTimer = null;
    }
  }

  // Retry manual tanpa remount — stage + overlay bubble selamat.
  function retryLoad() {
    if (requestedUrl) loadBackground(requestedUrl);
  }

  function loadBackground(url: string, triedFallback = false) {
    // Panggil setelah stage ada; kalau stage belum dibuat (mount awal), antri.
    if (!stage || !overlayLayer) {
      pendingUrl = url;
      return;
    }
    clearBgTimer();
    requestedUrl = url;
    imgReady = false;
    imgError = "";
    const my = ++bgToken;
    const img = new Image();
    const fail = (msg: string) => {
      if (my !== bgToken || requestedUrl !== url) return;
      // Fallback otomatis sekali ke thumb sebelum menyerah.
      if (!triedFallback && fallbackUrl && fallbackUrl !== url) {
        loadBackground(fallbackUrl, true);
        return;
      }
      clearBgTimer();
      bgToken++; // invalidasi onload telat — yang menang tetap gagal
      imgError = msg;
    };
    bgTimer = setTimeout(() => {
      fail("Gambar timed out (15 dtk) — preview lambat, coba lagi.");
    }, BG_LOAD_TIMEOUT_MS);
    img.onload = () => {
      // Abaikan load basi (user sudah pindah halaman / retry baru menang) dan
      // unmount (komponen dihancurkan saat load berjalan — stage sudah null).
      if (my !== bgToken || requestedUrl !== url) return;
      if (!stage || !overlayLayer) return;
      clearBgTimer();
      const s = scale();
      if (bgImage) {
        bgImage.image(img);
        bgImage.size({ width: imgW * s, height: imgH * s });
      } else {
        const bg = new Konva.Layer();
        bgImage = new Konva.Image({ image: img, width: imgW * s, height: imgH * s, listening: false });
        bg.add(bgImage);
        stage!.add(bg);
        bg.moveToBottom();
        // Overlay harus tetap di atas background.
        overlayLayer!.moveToTop();
      }
      stage!.width(imgW * s);
      stage!.height(imgH * s);
      imgReady = true;
      overlayLayer!.batchDraw();
      redraw();
    };
    img.onerror = () => {
      if (my !== bgToken || requestedUrl !== url) return;
      if (!stage) return;
      fail("Gambar gagal dimuat (preview backend gagal).");
    };
    img.src = url;
  }

  onMount(() => {
    const s = holder.clientWidth / Math.max(1, imgW);
    // sharp-preview D2: buffer canvas ikut resolusi layar (tajam di HiDPI/
    // scale-Windows); fallback 1 bila devicePixelRatio tak ada (SSR/test).
    const dpr = typeof window !== "undefined" && window.devicePixelRatio ? window.devicePixelRatio : 1;
    stage = new Konva.Stage({ container: holder, width: imgW * s, height: imgH * s, pixelRatio: dpr });
    overlayLayer = new Konva.Layer();
    stage.add(overlayLayer);
    stage.on("mousedown touchstart", onStageDown);
    stage.on("mousemove touchmove", onStageMove);
    stage.on("mouseup touchend", onStageUp);
    if (imageUrl) {
      pendingUrl = null;
      requestedUrl = imageUrl;
      loadBackground(imageUrl);
    } else if (pendingUrl) {
      const u = pendingUrl;
      pendingUrl = null;
      requestedUrl = u;
      loadBackground(u);
    }
    redraw();
    // Webfont (--font-sans) tiba belakangan; canvas tak redraw otomatis
    // saat font siap, jadi jadwalkan sekali (aman bila API tak ada).
    void document.fonts?.ready?.then(() => {
      redraw();
    });
  });

  onDestroy(() => {
    bgToken++; // invalidasi onload/onerror/timeout telat
    clearBgTimer();
    unlockScroll();
    stage?.destroy();
    stage = null;
  });
</script>

<!-- Latar canvas tetap gelap di kedua tema: gambar manga di atas netral gelap
     jauh lebih terbaca daripada di atas kertas putih, dan area di luar
     gambar tidak akan terlihat salah warna di light mode. -->
<div class="relative w-full">
<div bind:this={holder} class="w-full cursor-crosshair overflow-auto rounded-md border border-line bg-black"></div>
{#if editingIdx !== null && editBox}
  <textarea
    bind:this={editorEl}
    rows="3"
    placeholder="ketik terjemahan"
    class="absolute z-10 rounded px-1 py-0.5 text-center outline-none ring-2 ring-accent {editBox.dark ? 'bg-black/70 text-white' : 'bg-surface text-ink'}"
    style="left:{editBox.left}px; top:{editBox.top}px; width:{editBox.width}px; min-height:{editBox.minH}px; font-size:{editBox.fontSize}px;"
    bind:value={editingValue}
    onblur={commitEditor}
    onkeydown={onEditorKey}
  ></textarea>
{/if}
</div>
{#if !imgReady && !imgError}
  <p class="mt-1 text-[11px] text-ink-3">Memuat gambar…</p>
{:else if imgError}
  <p class="mt-1 text-[11px] text-danger">
    {imgError}
    <Button variant="default" size="sm" class="ml-2" onclick={retryLoad}>Coba lagi</Button>
  </p>
{/if}
