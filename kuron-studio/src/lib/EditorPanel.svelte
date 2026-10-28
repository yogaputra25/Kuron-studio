<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { api } from "../lib/api";
  import CanvasEditor from "../lib/CanvasEditor.svelte";
  import TranslatePanel from "../lib/TranslatePanel.svelte";
  import type { TranslateOpts } from "../lib/TranslatePanel.svelte";
  import { chipNumbers, clampBubble } from "../lib/bubble";
  import { mergeUserEdits, syncGeometryToTranslation } from "../lib/translation";
  import { computeTextLayout } from "../lib/textLayout";
  import { textMeasure } from "../lib/textMeasure";
  import type { BubbleBox, Page, PageTranslation, ReadingDirection, Tool } from "../lib/types";
  import { t } from "./i18n";
  import Button from "./ui/Button.svelte";
  import StatusBadge from "./ui/StatusBadge.svelte";
  import { isNarrow, isTablet, loadSidebarW, saveSidebarW, watchViewport } from "./device";
  import { SIDEBAR_DEFAULT, SIDEBAR_MIN, SNAP_DEFAULT, COLLAPSE_THRESHOLD, clampWidth, nearestSnap, sheetHeightPct, sidebarMax } from "./panel";

  /** Toolbar tool. Label Bahasa Indonesia, nilai = Tool di types.ts. */
  const TOOLS: [Tool, string][] = [
    ["select", "Pilih"],
    ["rect", "Rect"],
    ["ellipse", "Elips"],
    ["freeform", "Bebas"],
    ["tail", "Ekor"],
  ];

  interface Props {
    projectId: string;
    page: Page;
    fullImageUrl: string;
    fallbackUrl?: string;
    openError?: string;
    readingDir: ReadingDirection;
    onClose: () => void;
    onSaved: (page: Page) => void;
    onBatchOpts: (o: { providerId: string; opts: TranslateOpts }) => void;
  }

  let { projectId, page, fullImageUrl, fallbackUrl = "", openError = "", readingDir, onClose, onSaved, onBatchOpts }: Props = $props();

  // Snapshot per halaman dibuka — App me-remount via {#key page.id},
  // jadi `page`/`projectId` stabil selama panel hidup. Effect di bawah
  // menyinkronkan ulang, bukan capture nilai awal langsung.

  // State lokal: disinkron dari `page` di onMount + saat `page.id` berganti
  // (bukan capture nilai awal props). Guard JSON di syncFromPage() cegah
  // reset dirty saat onSaved mengembalikan konten identik.
  let bubbles = $state<BubbleBox[]>([]);
  let dirty = $state(false);
  let syncedFor = $state<string | null>(null);
  let saving = $state(false);
  let detecting = $state(false);
  let batching = $state(false);
  let detectEngine = $state("…");
  let error = $state("");
  // Hint ringan tool gambar (draw-tools-feedback D3): terpisah dari error
  // agar tak berebut prioritas; dibersihkan saat sukses/ganti-tool/ganti-mode.
  let drawHint = $state("");
  let editorRef = $state<{ deleteSelected: () => boolean; redraw: () => void } | null>(null);

  // M2 translation state.
  let translation = $state<PageTranslation | null>(null);
  let providerId = $state("");
  let lastOpts = $state<TranslateOpts | null>(null);
  let savingTr = $state(false);
  let glossMsg = $state("");
  let manualSel = $state<number | null>(null);

  // Draft kartu responsif (fix-card-input-lag D1): ketikan per huruf TAK
  // menyentuh $state — DOM textarea milik browser penuh. draftBox.cur plain
  // (properti kontainer, tanpa render ulang); markup baca saat render,
  // selalu terbaru → set ulang value= identik = no-op kursor-aman.
  // Commit via editTr existing (blur/debounce/pindah).
  let draftBox: { cur: { row: number; original: string; reading: string; translated: string } | null } = { cur: null };
  let draftTimer: ReturnType<typeof setTimeout> | null = null;
  let composing = false;
  // Sinyal reactive murah: true sekali per mulai-rangkaian-ketik (bukan per
  // huruf) → 1 render per fokus untuk indikator dirty, bukan N× per huruf.
  let dirtyDraft = $state(false);

  // Before|Split|After — after-lite: overlay Konva, bukan PNG typeset
  // (ponytail: after-true via command render_page_png bila diprotes).
  type PaneMode = "before" | "split" | "after";
  let mode = $state<PaneMode>("before");
  let tool = $state<Tool>("select");
  // Mode After tunggal = tak ada pane editable → tool gambar tak berlaku (D1).
  const readOnlyMode = $derived(mode === "after");
  function setTool(t: Tool) {
    tool = t;
    drawHint = "";
  }
  // Ref instance After (Before sudah punya editorRef): commit editor in-bubble
  // yang terbuka sebelum pane berganti — tak ada textarea basi.
  let afterRef = $state<{ commitEditor: () => void; redraw: () => void; deleteSelected: () => boolean } | null>(null);
  function setMode(m: PaneMode) {
    afterRef?.commitEditor();
    drawHint = "";
    mode = m;
    // Layout pane baru settle setelah render: redraw di frame berikut agar
    // holder.clientWidth sudah lebar baru (sync = baca lebar lama).
    // Kedua instance hidup di Split; redraw keduanya agar Split→tunggal
    // tak menyisakan stage ber-scale lama di pane yang selamat.
    requestAnimationFrame(() => {
      editorRef?.redraw();
      afterRef?.redraw();
    });
  }
  let beforeScroll: HTMLDivElement | null = $state(null);
  let afterScroll: HTMLDivElement | null = $state(null);
  let syncing = false;
  let saveTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  // §4 last-write-wins: naik tiap kirim; respons hanya menang bila gen-nya kini.
  let saveGen = 0;
  // Panel geser + mode tablet (panel-resize-tablet D1-D6): lebar sidebar px,
  // collapse boolean (tak dipersist), drag-lokal-commit agar Konva redraw
  // sekali saat lepas. narrow (<768px) → kolom; tablet (+coarse) → sheet.
  let sidebarW = $state(SIDEBAR_DEFAULT);
  let collapsed = $state(false);
  let dragW = $state<number | null>(null);
  let dragH = $state<number | null>(null);
  let narrow = $state(false);
  let tablet = $state(false);
  let snap = $state(SNAP_DEFAULT);
  let bodyEl: HTMLDivElement | null = $state(null);
  let unwatchViewport: (() => void) | null = null;
  const columnMode = $derived(narrow);
  /** Lebar tampil = drag saat geser, commit setelah lepas. */
  const displayW = $derived(dragW ?? sidebarW);
  /** Tinggi panel bawah (% kontainer) saat kolom: drag atau snap terkunci. */
  const panelHPct = $derived(sheetHeightPct(dragH ?? snap));
  /** Lebar kontainer body (px) untuk clamp dinamis MAX = bodyW - lantai canvas. */
  function bodyW(): number {
    const r = bodyEl?.getBoundingClientRect();
    if (r && r.width > 0) return r.width;
    if (typeof window !== "undefined" && window.innerWidth > 0) return window.innerWidth;
    return 0;
  }
  function syncViewport() {
    narrow = isNarrow();
    tablet = isTablet();
  }
  /** Redraw kedua stage setelah layout settle (pola setMode yang sudah ada). */
  function redrawAfterLayout() {
    requestAnimationFrame(() => {
      editorRef?.redraw();
      afterRef?.redraw();
    });
  }
  function onDividerDown(e: PointerEvent) {
    if (collapsed || !bodyEl) return;
    (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
    const r = bodyEl.getBoundingClientRect();
    if (columnMode) {
      dragH = ((r.bottom - e.clientY) / r.height) * 100;
    } else {
      dragW = r.right - e.clientX;
    }
  }
  function onDividerMove(e: PointerEvent) {
    if (!bodyEl) return;
    if (dragW !== null && !columnMode) {
      const r = bodyEl.getBoundingClientRect();
      dragW = r.right - e.clientX;
    } else if (dragH !== null && columnMode) {
      const r = bodyEl.getBoundingClientRect();
      dragH = Math.min(100, Math.max(0, ((r.bottom - e.clientY) / r.height) * 100));
    }
  }
  function onDividerUp() {
    if (dragW !== null) {
      if (dragW < COLLAPSE_THRESHOLD) {
        collapsed = true;
      } else {
        sidebarW = clampWidth(dragW, bodyW() || undefined);
        saveSidebarW(sidebarW);
      }
      dragW = null;
      redrawAfterLayout();
    }
    if (dragH !== null) {
      snap = nearestSnap(dragH);
      dragH = null;
      redrawAfterLayout();
    }
  }
  /** Keyboard divider: row = ←/→ 8px langsung commit (mentok MIN/MAX, tak collapse); kolom = ↑/↓ pindah snap. */
  function onDividerKey(e: KeyboardEvent) {
    if (collapsed) return;
    if (!columnMode && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
      e.preventDefault();
      sidebarW = clampWidth(sidebarW + (e.key === "ArrowLeft" ? -8 : 8), bodyW() || undefined);
      saveSidebarW(sidebarW);
      redrawAfterLayout();
    } else if (columnMode && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      e.preventDefault();
      const order = [40, 70, 100];
      const i = order.indexOf(snap);
      const ni = Math.min(order.length - 1, Math.max(0, i + (e.key === "ArrowUp" ? 1 : -1)));
      snap = order[ni];
      redrawAfterLayout();
    }
  }

  const originals = $derived(
    (translation?.bubbles ?? []).map((b) => b.original).filter((s) => s.trim()),
  );

  // Long-press bubble → save to glossary (M3-7).
  let pressTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  function pressStart(original: string, translated: string) {
    pressEnd();
    pressTimer = setTimeout(() => void saveToGlossary(original, translated), 500);
  }

  function pressEnd() {
    if (pressTimer) { clearTimeout(pressTimer); pressTimer = null; }
  }

  async function saveToGlossary(original: string, translated: string) {
    if (!original.trim() || !translated.trim()) return;
    try {
      await api.glossaryAdd(original.trim(), translated.trim());
      glossMsg = `Glossary: ${original.trim()} → tersimpan`;
    } catch (e) {
      glossMsg = String(e);
    }
  }

  // Sinkron dari App di onMount (isi awal) + saat `page.id` berganti.
  // Guard JSON: edit lokal → onSaved → prop `page` baru berisi konten
  // identik, jadi jangan reset dirty/bubbles karenanya.
  function syncFromPage() {
    const incoming = $state.snapshot(page.bubbles ?? []);
    const incomingTr = page.translation ? JSON.stringify(page.translation) : "";
    // D1 (fix-geometry-revert): cabang bubbles dulu TANPA guard — echo onSaved
    // dari jalur save lain (debounce terjemahan) menimpa geometri lokal yang
    // belum tersimpan. Guard = ada edit lokal (dirty) atau save terbang;
    // ganti halaman tetap memaksa resync.
    const samePage = syncedFor === page.id;
    const localBusy = dirty || saving || savingTr || saveTimer !== null;
    if (!samePage || (!localBusy && JSON.stringify(incoming) !== JSON.stringify(bubbles))) {
      bubbles = incoming;
      dirty = false;
      syncedFor = page.id;
    }
    // Sync-guard (typing-comfort D6): JANGAN timpa translation lokal yang
    // lebih baru dengan prop basi di jendela commit→save. Kondisi sibuk =
    // ketikan kotor (draftBox) ATAU save terbang (timer/saving). Saat tenang,
    // sync jalan normal (echo/mount/batch/retry lolos; kabar batch tertunda).
    // Cabang bubbles tak diubah (pelaku = cabang translation).
    const busyTyping = draftBox.cur !== null || saveTimer !== null || savingTr;
    if (!busyTyping && incomingTr && JSON.stringify(translation) !== incomingTr) {
      translation = $state.snapshot(page.translation) ?? null;
    }
  }

  $effect(() => {
    page.id;
    page.bubbles;
    page.translation;
    syncFromPage();
  });

  const nums = $derived(chipNumbers(bubbles, readingDir));

  async function runDetect() {
    detecting = true;
    error = "";
    try {
      const out = await api.detectBubbles(page.id);
      if (out.length === 0 && detectEngine === "manual") {
        error = "Model ONNX belum ada — gambar bubble manual (Rect/Ellipse/Freeform).";
      } else {
        bubbles = out;
        dirty = true;
      }
    } catch (e) {
      error = String(e);
    } finally {
      detecting = false;
    }
  }

  async function runDetectAll(pageIds: string[]) {
    batching = true;
    try {
      await api.detectBatch(pageIds);
      const proj = await api.getProject(projectId);
      for (const p of proj.pages) onSaved(p);
      // fix-geometry-revert: guard `dirty` di syncFromPage menahan propagasi
      // prop untuk halaman ini. Detect = penggantian geometri yang disengaja,
      // jadi terapkan langsung (pola sama dengan runDetect).
      const mine = proj.pages.find((p) => p.id === page.id);
      if (mine) {
        bubbles = $state.snapshot(mine.bubbles ?? []);
        dirty = false;
      }
    } catch (e) {
      error = String(e);
    } finally {
      batching = false;
    }
  }

  async function save() {
    saving = true;
    error = "";
    try {
      const cleaned = await api.saveBubbles(page.id, bubbles);
      bubbles = cleaned;
      dirty = false;
      onSaved({ ...page, bubbles: cleaned, status: cleaned.length > 0 ? "detected" : "noBubbles" });
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  function onKey(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA") {
      if (e.key === "Escape") onClose();
      return;
    }
    if (e.key === "Delete" || e.key === "Backspace") {
      if (saving) return;
      // D4: Delete hidup di pane mana pun — Before (editorRef) atau After (afterRef).
      if (editorRef?.deleteSelected() || afterRef?.deleteSelected()) void save();
    } else if (e.key === "s" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      void save();
    } else if (e.key === "Escape") {
      onClose();
    }
  }

  async function saveTranslationEdits() {
    if (draftBox.cur) commitDraft();
    if (!translation) return;
    // Syarat sah resize (bubble-resize-full D1): salin geometri canvas
    // ke translation by index SEBELUM kirim — backend hanya sentuh teks/ai*.
    translation.bubbles = syncGeometryToTranslation(
      bubbles,
      translation.bubbles,
      page.width || 800,
      page.height || 1200,
      // Layout dihitung di sini agar nilai tersimpan = yang terlihat di kanvas.
      (text, w, h) => computeTextLayout(text, w, h, textMeasure),
    );
    if (saveTimer) { clearTimeout(saveTimer); saveTimer = null; }
    const my = ++saveGen;
    savingTr = true;
    error = "";
    try {
      const out = await api.saveTranslation(page.id, translation.bubbles);
      if (my !== saveGen) return; // respons basi: save lebih baru sudah dikirim
      translation = out;
      onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: "translated", translation: out });
    } catch (e) {
      error = String(e);
    } finally {
      savingTr = false;
    }
  }

  /** Debounce ~500ms: ketik → save → prop ikut → guard JSON diam (§6). */
  function queueSaveTranslationEdits() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => { saveTimer = null; void saveTranslationEdits(); }, 500);
  }

  // Flush best-effort: commit sinkron dulu (tanpa await) agar ketikan tak
  // dibuang diam-diam saat panel ditutup; save ikut antre fire-and-forget.
  onDestroy(() => { if (draftBox.cur) commitDraft(); if (saveTimer) clearTimeout(saveTimer); if (draftTimer) clearTimeout(draftTimer); unwatchViewport?.(); });

  /** "Isi manual": translation null → bangun kosong lokal → save_translation. */
  async function createEmptyTranslation() {
    if (translation) return;
    if (bubbles.length === 0) { error = "Belum ada bubble — Detect atau gambar manual dulu."; return; }
    // Validasi ringan: index unik ikut urutan bubble (backend tanpa validasi).
    const idx = bubbles.map((_, i) => i);
    if (new Set(idx).size !== idx.length) { error = "Index bubble duplikat — simpan bubble dulu."; return; }
    const empty = {
      pageId: page.id, targetLang: "id", style: lastOpts?.style ?? "standard", model: "",
      bubbles: bubbles.map((b, i) => ({
        index: i, x: b.x, y: b.y, w: b.w, h: b.h, shape: b.shape,
        original: "", reading: "", translated: "",
        aiOriginal: "", aiReading: "", aiTranslated: "",
        needsWhitePatch: false, isUserEdited: false,
      })),
    };
    savingTr = true;
    error = "";
    try {
      const out = await api.saveTranslation(page.id, empty.bubbles);
      translation = out;
      onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: "translated", translation: out });
    } catch (e) {
      error = String(e);
    } finally {
      savingTr = false;
    }
  }

  function syncScroll(from: HTMLDivElement | null, to: HTMLDivElement | null) {
    if (!from || !to || syncing) return;
    syncing = true;
    to.scrollTop = from.scrollTop;
    to.scrollLeft = from.scrollLeft;
    syncing = false;
  }

  function onTranslated(t: PageTranslation) {
    // Draft kartu yang sedang diketik jadi edited + ikut mergeUserEdits
    // (tanpa ini draft basi menimpa hasil AI baru / hilang).
    if (draftBox.cur) commitDraft();
    translation = {
      ...t,
      bubbles: mergeUserEdits(translation?.bubbles, t.bubbles),
    };
    onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: "translated", translation });
    // P9 (export-text-parity): terjemahan datang dari backend yang tak punya
    // metrik font browser, jadi bubble-nya lahir tanpa `layout`. Kirim satu
    // save supaya export memakai jalur presisi, bukan hanya jalur fallback.
    void saveTranslationEdits();
  }

  // P9 — penyelamat data lama: kalau translation yang terbuka belum punya
  // `layout` di sebagian bubble (proyek sebelum field ini ada), paksa satu
  // push. Guard `layoutPushedFor` mencegah percobaan berulang kalau push-nya
  // gagal; konvergen karena backend mengembalikan bubble yang sama + layout.
  let layoutPushedFor = "";
  $effect(() => {
    const t = translation;
    if (!t || layoutPushedFor === page.id) return;
    if (!t.bubbles.some((b) => !b.layout)) return;
    if (savingTr || saveTimer !== null) return;
    layoutPushedFor = page.id;
    void saveTranslationEdits();
  });

  const selectedBubble = $derived(
    translation && manualSel !== null
      ? (translation.bubbles.find((b) => b.index === manualSel) ?? null)
      : null,
  );
  const selectedRow = $derived(
    selectedBubble ? translation!.bubbles.findIndex((b) => b.index === selectedBubble.index) : -1,
  );

  function editTr(i: number, field: "translated" | "reading" | "original", v: string) {
    if (!translation) return;
    translation.bubbles = translation.bubbles.map((b, j) =>
      j === i ? { ...b, [field]: v, isUserEdited: true } : b,
    );
    // Tanpa yank: seleksi hanya via selectBubble.
    queueSaveTranslationEdits();
  }

  type DraftField = "original" | "reading" | "translated";

  /** Ketikan kartu → cache plain TANPA $state: nol render/commit per huruf.
   *  DOM textarea milik browser penuh. Commit hanya di batas alami
   *  (blur/pindah/Save/compositionend/jaring) — bukan tiap jeda mikro.
   *  Timer-jaring = pengaman sesi-ditinggal, bukan ritme kerja. */
  const DRAFT_SAFETY_MS = 10_000;
  function draftInput(field: DraftField, v: string) {
    if (!translation || selectedRow < 0) return;
    if (!draftBox.cur || draftBox.cur.row !== selectedRow) {
      const cur = translation.bubbles[selectedRow];
      if (!cur) return;
      draftBox.cur = { row: selectedRow, original: cur.original, reading: cur.reading, translated: cur.translated };
    }
    draftBox.cur[field] = v;
    // Sekali per rangkaian-ketik (bukan per huruf): 1 render indikator.
    if (!dirtyDraft) dirtyDraft = true;
    // IME: tampung cache tapi jangan antre apa pun tengah komposisi.
    if (composing) return;
    // Jaring sekali per baris kotor (bukan per huruf): selamatkan sesi
    // ditinggal tanpa blur; ketik normal blur/pindah jauh sebelum fire.
    if (!draftTimer) draftTimer = setTimeout(() => { draftTimer = null; commitDraft(); }, DRAFT_SAFETY_MS);
  }

  /** IME mulai: jangan antre commit tengah komposisi. */
  function compStart() {
    composing = true;
    if (draftTimer) { clearTimeout(draftTimer); draftTimer = null; }
  }

  /** IME selesai = batas kata alami → commit langsung (D2). */
  function compEnd() {
    composing = false;
    if (draftBox.cur) commitDraft();
  }

  /** Commit field kotor via editTr existing (sekali, bukan per huruf). */
  function commitDraft() {
    if (composing) return;
    if (draftTimer) { clearTimeout(draftTimer); draftTimer = null; }
    if (!draftBox.cur || !translation) return;
    const { row, original, reading, translated } = draftBox.cur;
    draftBox.cur = null;
    dirtyDraft = false;
    const cur = translation.bubbles[row];
    if (!cur) return;
    if (cur.original !== original) editTr(row, "original", original);
    if (cur.reading !== reading) editTr(row, "reading", reading);
    if (cur.translated !== translated) editTr(row, "translated", translated);
  }

  /** Pindah seleksi saat draft kotor → commit otomatis dulu (D2). */
  function selectBubble(idx: number | null) {
    if (draftBox.cur) commitDraft();
    manualSel = idx;
    if (idx === null) { draftBox.cur = null; dirtyDraft = false; }
  }

  /** Geometri bubble terpilih via angka (bubble-resize-full D4/C).
   *  Satu jalur onChange seperti drag-handle; clamp + min 8px konvensi existing. */
  function setBubbleGeometry(i: number, patch: Partial<Pick<BubbleBox, "x" | "y" | "w" | "h">>) {
    const b = bubbles[i];
    if (!b) return;
    const nb = {
      ...b,
      x: Math.round(patch.x ?? b.x),
      y: Math.round(patch.y ?? b.y),
      w: Math.round(patch.w ?? b.w),
      h: Math.round(patch.h ?? b.h),
    };
    bubbles[i] = clampBubble(nb, page.width || 800, page.height || 1200);
    bubbles = [...bubbles];
    dirty = true;
  }

  /** Skala dari PUSAT agar kotak tak "lari" (bubble-resize-full D4). */
  function scaleSelectedBubble(factor: number) {
    const b = bubbles[selectedIdxForGeometry()];
    if (!b) return;
    const cx = b.x + b.w / 2;
    const cy = b.y + b.h / 2;
    const nw = Math.max(8, Math.round(b.w * factor));
    const nh = Math.max(8, Math.round(b.h * factor));
    setBubbleGeometry(selectedIdxForGeometry(), {
      x: Math.round(cx - nw / 2),
      y: Math.round(cy - nh / 2),
      w: nw,
      h: nh,
    });
  }

  /** Index geometri: manualSel (translation.index) → posisi di bubbles[].
   *  Repair R5: korespondensi hanya sah bila jumlah sama (belum hapus/tambah
   *  tanpa save); tak sama → -1 agar tak edit bubble yang salah. */
/** Ukuran font manual bubble terpilih (px HALAMAN). Kosong/0 = auto-fit. */
function setBubbleFontSize(v: number | null) {
  if (!translation) return;
  const i = selectedRow;
  if (i < 0) return;
  const next = v && v > 0 ? Math.round(v) : null;
  translation.bubbles = translation.bubbles.map((b, j) =>
    j === i ? { ...b, fontSize: next } : b,
  );
  // Sama seperti ketikan teks: kanvas langsung ikut (prop `translations`),
  // penyimpanan menyusul lewat antrean yang sudah ada.
  queueSaveTranslationEdits();
}

  function selectedIdxForGeometry(): number {
    if (manualSel === null) return -1;
    if (translation && bubbles.length !== translation.bubbles.length) return -1;
    const byIndex = bubbles.findIndex((_, bi) => bi === manualSel);
    return byIndex >= 0 ? byIndex : manualSel < bubbles.length ? manualSel : -1;
  }
  /** Repair R5: jumlah beda = baru hapus/tambah tanpa save → kontrol sembunyi. */
  function geometryOutOfSync(): boolean {
    return !!translation && manualSel !== null && bubbles.length !== translation.bubbles.length;
  }

  /** ↺ opsi A: current = baseline AI terakhir, kirim LANGSUNG (§4 D7). */
  function resetBubble(i: number) {
    if (draftTimer) { clearTimeout(draftTimer); draftTimer = null; }
    draftBox.cur = null;
    dirtyDraft = false;
    if (!translation) return;
    translation.bubbles = translation.bubbles.map((b, j) =>
      j === i ? { ...b, original: b.aiOriginal, reading: b.aiReading, translated: b.aiTranslated, isUserEdited: false } : b,
    );
    // Sinkron, bukan queueSave: generasi baru membuat respons save lama basi.
    void saveTranslationEdits();
  }

  onMount(async () => {
    const w0 = typeof window !== "undefined" ? window.innerWidth : 0;
    sidebarW = loadSidebarW(SIDEBAR_MIN, sidebarMax(w0 || undefined), SIDEBAR_DEFAULT);
    syncViewport();
    unwatchViewport?.();
    unwatchViewport = watchViewport(syncViewport);
    try {
      const st = await api.detectStatus();
      detectEngine = st.engine;
    } catch {
      detectEngine = "offline";
    }
    await listen<{ page_id: string; done: number; total: number }>("detect_progress", () => {});
  });
</script>

<svelte:window onkeydown={onKey} />

<div class="flex h-full flex-col">
  <div class="flex flex-wrap items-center gap-x-3 gap-y-2 border-b border-line bg-surface px-5 py-2.5">
    <Button variant="ghost" size="sm" onclick={onClose}>← Grid</Button>

    <div class="flex min-w-0 items-center gap-2.5">
      <span class="font-display max-w-48 truncate text-sm font-semibold" title={page.path}>
        {page.path.split(/[/\\]/).pop()}
      </span>
      <StatusBadge status={page.status} />
      <span class="tnum hidden text-xs text-ink-3 lg:inline">
        engine: {detectEngine} · {bubbles.length} bubble
      </span>
    </div>

    <div class="ml-auto flex items-center gap-2">
      <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label="Mode pane">
        {#each [["before", "Before"], ["split", "Split"], ["after", "After"]] as [m, label] (m)}
          <button
            class={[
              "h-8 border-l border-line px-3 text-xs font-medium transition-colors first:border-l-0",
              mode === m
                ? "bg-accent text-accent-fg"
                : "bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink",
            ]}
            aria-pressed={mode === m}
            onclick={() => setMode(m as PaneMode)}
          >{label}</button>
        {/each}
      </div>
      <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label="Tool gambar" title={readOnlyMode ? "Gambar hanya di Before" : undefined}>
        {#each TOOLS as [t, label] (t)}
          {@const drawTool = t !== "select"}
          {@const off = drawTool && readOnlyMode}
          <button
            class={[
              "h-8 border-l border-line px-3 text-xs font-medium transition-colors first:border-l-0",
              tool === t
                ? "bg-accent text-accent-fg"
                : "bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink",
              off ? "cursor-not-allowed opacity-40" : "",
            ]}
            aria-pressed={tool === t}
            disabled={off}
            title={off ? "Gambar hanya di Before" : label}
            onclick={() => setTool(t as Tool)}
          >{label}</button>
        {/each}
      </div>
      <Button
        variant="ghost"
        size="sm"
        onclick={() => (collapsed = !collapsed)}
        title={collapsed ? $t("expandSidebar") : $t("collapseSidebar")}
        aria-label={collapsed ? $t("expandSidebar") : $t("collapseSidebar")}
        aria-pressed={collapsed}
      >{collapsed ? "»" : "«"}</Button>
      <Button
        variant="default"
        size="sm"
        onclick={runDetect}
        disabled={detecting}
      >{detecting ? "Detect…" : "Detect"}</Button>
      <!-- "Tersimpan" bukan primary: state itu sudah selesai, bukan aksi.
           Warna coral dipakai hanya saat ada perubahan yang belum disimpan. -->
      <Button
        variant={dirty && !saving ? "primary" : "default"}
        size="sm"
        onclick={save}
        disabled={saving || !dirty}
      >
        {saving ? "Simpan…" : dirty ? "Simpan*" : "Tersimpan"}
      </Button>
    </div>
  </div>

  {#if openError}
    <p role="alert" class="border-b border-danger/30 bg-danger-soft px-4 py-1.5 text-xs text-danger">{openError}</p>
  {/if}
  {#if error}
    <p role="alert" class="border-b border-warn-soft bg-warn-soft px-4 py-1.5 text-xs text-warn">{error}</p>
  {/if}
  {#if drawHint}
    <p role="status" class="border-b border-line bg-surface-2 px-4 py-1.5 text-xs text-ink-2">{drawHint}</p>
  {/if}

  {#if openError && !fullImageUrl}
    <div class="flex flex-1 items-center justify-center p-8">
      <div class="max-w-sm text-center text-sm text-ink-2">
        <p class="mb-2 font-semibold text-danger">Gambar gagal dimuat.</p>
        <p class="mb-4">Kembali ke grid dan coba lagi — tidak perlu reload.</p>
        <Button variant="default" onclick={onClose}>← Grid</Button>
      </div>
    </div>
  {:else}
  <div class="flex min-h-0 flex-1" class:flex-col={columnMode} bind:this={bodyEl}>
    {#if mode === "before" || mode === "split"}
      <div
        class="min-h-0 min-w-0 flex-1 overflow-auto p-4"
        bind:this={beforeScroll}
        onscroll={() => syncScroll(beforeScroll, afterScroll)}
      >
        <CanvasEditor
          bind:this={editorRef}
          imageUrl={fullImageUrl}
          fallbackUrl={fallbackUrl}
          imgW={page.width || 800}
          imgH={page.height || 1200}
          initial={bubbles}
          {readingDir}
          {tool}
          translations={translation?.bubbles ?? []}
          showTranslation={false}
          editable={true}
          canDraw={true}
          externalSelected={manualSel}
          onSelectForward={(i) => selectBubble(i)}
          onDrawHint={(m) => { drawHint = m; }}
          onToolDone={() => setTool("select")}
          onChange={(nb) => {
            bubbles = nb;
            dirty = true;
            drawHint = "";
          }}
        />
      </div>
    {/if}
    {#if mode === "after" || mode === "split"}
      <div
        class="min-h-0 min-w-0 flex-1 overflow-auto p-4"
        class:border-l={mode === "split" && !columnMode}
        class:border-t={mode === "split" && columnMode}
        class:border-line={mode === "split"}
        bind:this={afterScroll}
        onscroll={() => syncScroll(afterScroll, beforeScroll)}
      >
        <CanvasEditor
          bind:this={afterRef}
          imageUrl={fullImageUrl}
          fallbackUrl={fallbackUrl}
          imgW={page.width || 800}
          imgH={page.height || 1200}
          initial={bubbles}
          {readingDir}
          tool={tool}
          translations={translation?.bubbles ?? []}
          showTranslation={true}
          editable={true}
          canDraw={false}
          externalSelected={manualSel}
          onSelectForward={(i) => selectBubble(i)}
          onEditTranslation={(i, v) => editTr(i, "translated", v)}
          onDrawHint={(m) => { drawHint = m; }}
          onChange={(nb) => { bubbles = nb; dirty = true; drawHint = ""; }}
        />
      </div>
    {/if}
    {#if !collapsed}
      {#if columnMode}
        {#if tablet}
          <!-- Drag-handle sheet ≥44px khusus sentuh (tablet portrait). -->
          <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
          <!-- role=separator + tabindex = pola sash (VS Code). Svelte mengira div non-interaktif; abaikan dengan alasan. -->
          <div
            role="separator"
            aria-orientation="horizontal"
            aria-label={$t("sheetHandle")}
            tabindex="0"
            class="flex h-11 shrink-0 cursor-row-resize touch-none items-center justify-center border-t border-line bg-surface-2"
            onpointerdown={onDividerDown}
            onpointermove={onDividerMove}
            onpointerup={onDividerUp}
            onpointercancel={onDividerUp}
            onkeydown={onDividerKey}
          ><span class="h-1 w-12 rounded-full bg-line"></span></div>
        {:else}
          <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
          <div
            role="separator"
            aria-orientation="horizontal"
            aria-label={$t("sheetHandle")}
            tabindex="0"
            class="h-2 shrink-0 cursor-row-resize touch-none border-t border-line bg-surface-2 hover:bg-hover"
            onpointerdown={onDividerDown}
            onpointermove={onDividerMove}
            onpointerup={onDividerUp}
            onpointercancel={onDividerUp}
            onkeydown={onDividerKey}
          ></div>
        {/if}
      {:else}
        <!-- Divider vertikal 8px hit-area (cukup untuk sentuh) di batas canvas–sidebar. -->
        <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
        <div
          role="separator"
          aria-orientation="vertical"
          aria-label={$t("sheetHandle")}
          tabindex="0"
          class="w-2 shrink-0 cursor-col-resize touch-none border-l border-line bg-surface-2 hover:bg-hover"
          onpointerdown={onDividerDown}
          onpointermove={onDividerMove}
          onpointerup={onDividerUp}
          onpointercancel={onDividerUp}
          onkeydown={onDividerKey}
        ></div>
      {/if}
    <!-- Lebar via inline style (D2): row = px commit, kolom = full + tinggi sheet %. -->
    <aside
      class="flex shrink-0 flex-col overflow-y-auto border-line bg-surface"
      class:w-full={columnMode}
      class:border-t={columnMode}
      class:border-l={!columnMode}
      style:width={!columnMode ? `${displayW}px` : undefined}
      style:height={columnMode ? `${panelHPct}%` : undefined}
    >
      <div class="space-y-5 p-4">
        {#if columnMode && snap === 100}
          <Button variant="ghost" size="sm" onclick={() => { snap = 40; redrawAfterLayout(); }} title={$t("shrinkSheet")} aria-label={$t("shrinkSheet")}>∨ {$t("shrinkSheet")}</Button>
        {/if}
        <section>
          <h3 class="font-display mb-2.5 flex items-baseline justify-between text-sm font-semibold text-ink">
            Bubble
            <span class="tnum text-xs font-normal text-ink-3">{bubbles.length}</span>
          </h3>
          {#if bubbles.length === 0}
            <p class="rounded-md border border-dashed border-line px-3 py-4 text-center text-xs leading-relaxed text-ink-3">
              Belum ada bubble.
              <br />Klik Detect atau gambar manual.
            </p>
          {:else}
            <ol class="space-y-1.5">
              {#each bubbles as b, i (i)}
                <li
                  class="group flex items-center gap-2.5 rounded-md border border-line bg-surface-2 px-2.5 py-2
                         transition-colors hover:border-line-strong"
                >
                  <span class="tnum flex size-5 shrink-0 items-center justify-center rounded-full bg-accent-soft text-[10px] font-bold text-accent">
                    {nums.get(i) ?? i + 1}
                  </span>
                  <span class="tnum truncate text-[11px] text-ink-2">{b.kind ?? "rect"} · {b.w}×{b.h}</span>
                  <Button
                    variant="quiet"
                    size="sm"
                    class="ml-auto opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
                    disabled={saving}
                    title={`Hapus bubble ${(nums.get(i) ?? i + 1)}`}
                    onclick={() => {
                      bubbles.splice(i, 1);
                      bubbles = [...bubbles];
                      dirty = true;
                      void save();
                    }}
                  >{$t("remove")}</Button>
                </li>
              {/each}
            </ol>
          {/if}
        </section>

        <section class="border-t border-line pt-5">
          <h3 class="font-display mb-2.5 text-sm font-semibold text-ink">Batch</h3>
          <Button
            variant="default"
            block
            onclick={() => runDetectAll([page.id])}
            disabled={batching}
          >{batching ? $t("running") : "Detect halaman ini"}</Button>
          <p class="mt-2.5 text-[11px] leading-relaxed text-ink-3">
            Hapus: pilih bubble + <kbd class="font-mono text-ink-2">Delete</kbd>.
            Simpan: <kbd class="font-mono text-ink-2">Ctrl</kbd>+<kbd class="font-mono text-ink-2">S</kbd>.
          </p>
        </section>

        <section class="border-t border-line pt-4">
          <TranslatePanel
            pageId={page.id}
            providerId={providerId}
            bubblesCount={bubbles.length}
            {readingDir}
            {originals}
            onProviderChange={(id) => { providerId = id; if (lastOpts) onBatchOpts({ providerId: id, opts: lastOpts }); }}
            onOptsChange={(o) => { lastOpts = o; onBatchOpts({ providerId, opts: o }); }}
            onTranslated={onTranslated}
            onError={(m) => {
              error = m;
              onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: "failed" });
            }}
          />
        </section>

        {#if translation}
          {@const singleCard = selectedBubble && selectedRow >= 0
            ? { b: selectedBubble, i: selectedRow }
            : translation.bubbles.length === 1
              ? { b: translation.bubbles[0], i: 0 }
              : null}
          <section class="border-t border-line pt-4">
            <div class="mb-3 flex items-center gap-2">
              <h3 class="font-display text-sm font-semibold text-ink">Terjemahan</h3>
              <StatusBadge status="translated" class="ml-auto" />
            </div>

            <div class="mb-3 flex gap-2">
              {#if dirtyDraft}<span class="rounded bg-warn-soft px-1.5 py-0.5 text-[10px] text-warn">belum tersimpan</span>{/if}
              <Button
                variant="default"
                size="sm"
                class="ml-auto"
                onclick={async () => { if (draftBox.cur) commitDraft(); if (dirty) await save(); await saveTranslationEdits(); }}
                disabled={savingTr}
              >{savingTr ? "…" : dirtyDraft ? `${$t("save")}*` : $t("save")}</Button>
            </div>

            {#if mode !== "before"}
              <p class="mb-2 text-[11px] leading-relaxed text-ink-3">Double-klik bubble di After untuk edit langsung.</p>
            {/if}
            {#if singleCard}
              {@const b = singleCard.b}
              {@const i = singleCard.i}
              <ol class="space-y-2">
                <li
                  class="rounded-md border border-accent/40 bg-surface-2 p-2.5"
                  onpointerdown={() => pressStart(b.original, b.translated)}
                  onpointerup={pressEnd}
                  onpointerleave={pressEnd}
                  title="Tahan 0.5 dtk untuk simpan ke Glossary"
                >
                  <div class="mb-1.5 flex items-center gap-1.5 text-[10px] text-ink-3">
                    <button
                      class="tnum rounded px-1 font-medium hover:text-ink"
                      onclick={() => selectBubble(manualSel === b.index ? null : b.index)}
                      title="Pilih bubble ini"
                    >#{b.index + 1}</button>
                    {#if b.isUserEdited}
                      <span class="rounded bg-warn-soft px-1.5 py-0.5 text-warn">edited</span>
                    {/if}
                    {#if b.isUserEdited && b.aiTranslated}
                      <button
                        class="ml-auto rounded px-1 hover:text-ink"
                        onclick={() => resetBubble(i)}
                        title="Kembalikan ke hasil AI terakhir"
                      >↺</button>
                    {/if}
                  </div>
                  <p class="mt-0.5 block text-[10px] text-ink-3">Original (JP)</p>
                  <textarea
                    rows="2"
                    class="mt-1 w-full rounded-md border border-line bg-surface px-2.5 py-2 text-[13px] text-ink
                           outline-none transition-colors placeholder:text-ink-3
                           focus:border-accent focus:ring-2 focus:ring-accent/25"
                    value={draftBox.cur && draftBox.cur.row === i ? draftBox.cur.original : b.original}
                    oninput={(e) => draftInput("original", (e.target as HTMLTextAreaElement).value)}
                    oncompositionstart={compStart}
                    oncompositionend={compEnd}
                    onblur={commitDraft}
                  ></textarea>
                  <p class="mt-1.5 block text-[10px] text-ink-3">Reading</p>
                  <textarea
                    rows="1"
                    class="mt-1 w-full rounded-md border border-line bg-surface px-2.5 py-2 text-[13px] text-ink
                           outline-none transition-colors placeholder:text-ink-3
                           focus:border-accent focus:ring-2 focus:ring-accent/25"
                    value={draftBox.cur && draftBox.cur.row === i ? draftBox.cur.reading : b.reading}
                    oninput={(e) => draftInput("reading", (e.target as HTMLTextAreaElement).value)}
                    oncompositionstart={compStart}
                    oncompositionend={compEnd}
                    onblur={commitDraft}
                  ></textarea>
                  <p class="mt-1.5 block text-[10px] text-ink-3">Terjemahan (ID)</p>
                  <textarea
                    rows="2"
                    class="mt-1 w-full rounded-md border border-line bg-surface px-2.5 py-2 text-[13px] text-ink
                           outline-none transition-colors placeholder:text-ink-3
                           focus:border-accent focus:ring-2 focus:ring-accent/25"
                    value={draftBox.cur && draftBox.cur.row === i ? draftBox.cur.translated : b.translated}
                    oninput={(e) => draftInput("translated", (e.target as HTMLTextAreaElement).value)}
                    oncompositionstart={compStart}
                    oncompositionend={compEnd}
                    onblur={commitDraft}
                  ></textarea>
                  {#if glossMsg}<p class="mt-1.5 text-[10px] text-success" role="status">{glossMsg}</p>{/if}
                  {#if bubbles[selectedIdxForGeometry()]}
                    <p class="mt-2 block text-[10px] text-ink-3">Geometri (px asli)</p>
                    <div class="mt-1 grid grid-cols-4 gap-1.5">
                      {#each [["x"], ["y"], ["w"], ["h"]] as [k] (k)}
                        <label class="flex flex-col gap-0.5 text-[10px] text-ink-3">{k}
                          <input
                            type="number"
                            min="0"
                            class="tnum w-full rounded-md border border-line bg-surface px-1.5 py-1 text-[12px] text-ink outline-none focus:border-accent"
                            value={bubbles[selectedIdxForGeometry()][k as "x" | "y" | "w" | "h"]}
                            onchange={(e) => setBubbleGeometry(selectedIdxForGeometry(), { [k]: Number((e.target as HTMLInputElement).value) } as Partial<Pick<BubbleBox, "x" | "y" | "w" | "h">>)}
                          />
                        </label>
                      {/each}
                    </div>
                    <div class="mt-1.5 flex gap-1.5">
                      <Button variant="quiet" size="sm" title="Kecilkan 10% dari pusat" onclick={() => scaleSelectedBubble(0.9)}>−10%</Button>
                      <Button variant="quiet" size="sm" title="Besarkan 10% dari pusat" onclick={() => scaleSelectedBubble(1.1)}>+10%</Button>
                    </div>
                      <label class="ml-2 flex flex-col gap-0.5 text-[10px] text-ink-3">font (px)
                        <input
                          type="number"
                          min="4"
                          class="tnum w-16 rounded-md border border-line bg-surface px-1.5 py-1 text-[12px] text-ink outline-none focus:border-accent"
                          value={singleCard.b.fontSize ?? ""}
                          placeholder="auto"
                          onchange={(e) => setBubbleFontSize(Number((e.target as HTMLInputElement).value) || null)}
                        />
                      </label>
                      {#if singleCard.b.fontSize}
                        <Button variant="quiet" size="sm" title="Kembali ke ukuran otomatis" onclick={() => setBubbleFontSize(null)}>auto</Button>
                      {/if}
                  {:else if geometryOutOfSync()}
                    <p class="mt-2 text-[10px] leading-relaxed text-warn" role="status">Geometri belum sinkron (bubble baru dihapus/ditambah) — Simpan dulu agar kontrol angka menunjuk bubble yang benar.</p>
                  {/if}
                </li>
              </ol>
            {:else}
              <p class="rounded-md border border-dashed border-line px-3 py-4 text-center text-xs leading-relaxed text-ink-3">
                Klik bubble di After untuk memilih.
              </p>
            {/if}
          </section>
        {:else}
          <section class="border-t border-line pt-4">
            <Button
              variant="default"
              block
              onclick={() => void createEmptyTranslation()} disabled={savingTr || bubbles.length === 0}
            >{savingTr ? "…" : "Isi manual"}</Button>
            <p class="mt-2 text-[11px] leading-relaxed text-ink-3">Buat translation kosong tanpa AI, lalu ketik per bubble.</p>
          </section>
        {/if}
      </div>
    </aside>
    {/if}
  </div>
  {/if}
</div>
