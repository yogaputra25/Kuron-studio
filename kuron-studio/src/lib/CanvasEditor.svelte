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
  }

  let { imageUrl, fallbackUrl = "", imgW, imgH, initial, readingDir, tool, onChange, translations = [], showTranslation = false }: Props = $props();

  let holder: HTMLDivElement;
  let stage: Konva.Stage | null = null;
  let overlayLayer: Konva.Layer | null = null;
  let bgImage: Konva.Image | null = null;
  let selectedIdx = $state<number | null>(null);

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

  $effect(() => {
    translations;
    showTranslation;
    redraw();
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

  function redraw() {
    if (!stage || !overlayLayer) return;
    const s = scale();
    const P = paint();
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
        if (tr?.translated) {
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
        selectedIdx = i;
    redraw();
    // Canvas tidak digambar ulang saat webfont selesai. Kalau redraw pertama
    // jalan sebelum Inter siap, teks overlay terukur sekaligus tergambar pakai
    // font fallback — dan tetap begitu sampai ada perubahan state lain.
    // Itu sebabnya hasil terjemahan terlihat memakai font yang salah.
    void document.fonts?.ready.then(() => {
      fontCache = "";
      redraw();
    });
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
          fill: P.accent,
          draggable: true,
        });
        handle.on("dragmove", () => {
          const nw = Math.max(8, handle.x() / s);
          const nh = Math.max(8, handle.y() / s);
          bubbles[i] = clampBubble({ ...b, w: nw, h: nh }, imgW, imgH);
          redrawKeep(handle);
        });
        handle.on("dragend", () => {
          onChange($state.snapshot(bubbles));
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

  function onStageDown(e: Konva.KonvaEventObject<MouseEvent | TouchEvent>) {
    if (tool === "select") return;
    // Ekor: user mengklik TEPAT pada bubble terpilih (itu titik melekat,
    // lalu drag keluar) atau di mana saja untuk menunjuk ujung ekor. Jadi
    // jangan pernah reject berdasarkan target — cukup butuh bubble terpilih.
    if (tool === "tail") {
      if (selectedIdx === null || !stage) return;
      const p = stage.getPointerPosition();
      if (!p) return;
      const s = scale();
      setTailForSelected([Math.round(p.x / s), Math.round(p.y / s)]);
      return;
    }
    // Tool gambar baru mulai dari area kosong — klik bubble = pilih/pindah.
    if (e.target !== stage) return;
    const [x, y] = stagePos();
    drawing = { x0: x, y0: y, node: null, pts: [[x, y]] };
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

  function onStageUp() {
    if (!drawing) return;
    const pts = drawing.pts;
    drawing.node?.destroy();
    drawing = null;
    if (tool === "freeform") {
      // 6 = 3 titik (polyline minimal), bukan angka acak.
      if (pts.length < 6) return;
      const xs = pts.map((p) => p[0]);
      const ys = pts.map((p) => p[1]);
      const bbW = Math.max(...xs) - Math.min(...xs);
      const bbH = Math.max(...ys) - Math.min(...ys);
      if (bbW < MIN_BUBBLE_PX || bbH < MIN_BUBBLE_PX) return;
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
      if (pts.length < 2) return;
      const [[x0, y0], [x1, y1]] = pts;
      const w = Math.abs(x1 - x0);
      const h = Math.abs(y1 - y0);
      if (w < MIN_BUBBLE_PX || h < MIN_BUBBLE_PX) return;
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
    stage = new Konva.Stage({ container: holder, width: imgW * s, height: imgH * s });
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
  });

  onDestroy(() => {
    bgToken++; // invalidasi onload/onerror/timeout telat
    clearBgTimer();
    stage?.destroy();
    stage = null;
  });
</script>

<!-- Latar canvas tetap gelap di kedua tema: gambar manga di atas netral gelap
     jauh lebih terbaca daripada di atas kertas putih, dan area di luar
     gambar tidak akan terlihat salah warna di light mode. -->
<div bind:this={holder} class="w-full cursor-crosshair overflow-auto rounded-md border border-line bg-black"></div>
{#if !imgReady && !imgError}
  <p class="mt-1 text-[11px] text-ink-3">Memuat gambar…</p>
{:else if imgError}
  <p class="mt-1 text-[11px] text-danger">
    {imgError}
    <Button variant="default" size="sm" class="ml-2" onclick={retryLoad}>Coba lagi</Button>
  </p>
{/if}
