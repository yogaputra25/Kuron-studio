<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { onDestroy, onMount } from "svelte";
  import { api } from "../contracts/api";
  import TranslationCard from "./TranslationCard.svelte";
  import EditorToolbar, { type PaneMode } from "./EditorToolbar.svelte";
  import EditorPanes, { type CanvasApi } from "./EditorPanes.svelte";
  import TranslatePanel from "../provider/TranslatePanel.svelte";
  import { rev, shouldApplyBubbles, shouldApplyTranslation } from "./useSyncedPage.svelte";
  import { useDraftText } from "./useDraftText.svelte";
  import type { TranslateOpts } from "../provider/translateOpts";
  import { chipNumbers, clampBubble } from "./bubble";
  import { mergeUserEdits, syncGeometryToTranslation } from "./translation";
  import { computeTextLayout } from "./text/textLayout";
  import { textMeasure } from "./text/textMeasure";
  import type { BubbleBox, Page, PageTranslation, ReadingDirection, Tool } from "../contracts/types";
  import { TARGET_LANGS } from "../contracts/types";
  import { allPages } from "../library/library";
  import { t } from "../app/i18n";
  import { statusAfterBubbleSave, statusAfterTranslationSave, translationComplete } from "../library/status";
  import Button from "../ui/Button.svelte";
  import KuroMascot from "../ui/KuroMascot.svelte";
  import { usePanelResize } from "./usePanelResize.svelte";
  import Divider from "../ui/Divider.svelte";

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
  let editorRef = $state<CanvasApi | null>(null);

  // M2 translation state.
  let translation = $state<PageTranslation | null>(null);
  let providerId = $state("");
  /** Satu-satunya pemilik opts translate (FE-6): panel lain terima referensi. */
  let opts = $state<TranslateOpts | null>(null);
  let appVersion = $state("");
  // Label bahasa ikut target aktual, bukan hardcoded sepasang bahasa (#26
  // follow-up: user ID→EN melihat label JP di original dan ID di terjemahan).
  const trgLang = $derived(translation?.targetLang || opts?.targetLang || "id");
  const trgLabel = $derived(TARGET_LANGS.find((l) => l.value === trgLang)?.label ?? trgLang);
  let savingTr = $state(false);
  let glossMsg = $state("");
  let manualSel = $state<number | null>(null);

  // Draft ketikan kartu tinggal di useDraftText (FE-4, task 1.3): cache plain
  // + timer + IME terkungkung di hook; panel hanya pasok baris/komit.

  // Before|Split|After — after-lite: overlay Konva, bukan PNG typeset
  // (ponytail: after-true via command render_page_png bila diprotes).
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
  let afterRef = $state<CanvasApi | null>(null);
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
  // Resize panel + mode tablet milik usePanelResize (FE-4, task 3.1).
  // Panel simpan bodyEl (layout) + redrawAfterLayout (refs stage) sebagai deps.
  let bodyEl: HTMLDivElement | null = $state(null);
  /** Redraw kedua stage setelah layout settle (pola setMode yang sudah ada). */
  function redrawAfterLayout() {
    requestAnimationFrame(() => {
      editorRef?.redraw();
      afterRef?.redraw();
    });
  }
  const resize = usePanelResize({
    getBody: () => bodyEl,
    onLayoutSettled: () => redrawAfterLayout(),
  });

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
  // Guard rev: edit lokal → onSaved → prop `page` baru berisi konten
  // identik, jadi jangan reset dirty/bubbles karenanya.
  function syncFromPage() {
    const incoming = $state.snapshot(page.bubbles ?? []);
    const incomingTr = page.translation ? rev(page.translation) : "";
    // D1 (fix-geometry-revert): cabang bubbles dulu TANPA guard — echo onSaved
    // dari jalur save lain (debounce terjemahan) menimpa geometri lokal yang
    // belum tersimpan. Guard = ada edit lokal (dirty) atau save terbang;
    // ganti halaman tetap memaksa resync.
    const samePage = syncedFor === page.id;
    const localBusy = dirty || saving || savingTr || saveTimer !== null;
    if (shouldApplyBubbles({ samePage, localBusy, incoming, current: bubbles })) {
      bubbles = incoming;
      dirty = false;
      syncedFor = page.id;
    }
    // Sync-guard (typing-comfort D6): JANGAN timpa translation lokal yang
    // lebih baru dengan prop basi di jendela commit→save. Kondisi sibuk =
    // ketikan kotor (draft) ATAU save terbang (timer/saving). Saat tenang,
    // sync jalan normal (echo/mount/batch/retry lolos; kabar batch tertunda).
    // Cabang bubbles tak diubah (pelaku = cabang translation).
    const busyTyping = draft.cur !== null || saveTimer !== null || savingTr;
    if (shouldApplyTranslation({ busyTyping, incomingRev: incomingTr, current: translation })) {
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
      for (const p of allPages(proj)) onSaved(p);
      // fix-geometry-revert: guard `dirty` di syncFromPage menahan propagasi
      // prop untuk halaman ini. Detect = penggantian geometri yang disengaja,
      // jadi terapkan langsung (pola sama dengan runDetect).
      const mine = allPages(proj).find((p) => p.id === page.id);
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

  async function save(): Promise<boolean> {
    saving = true;
    error = "";
    try {
      const cleaned = await api.saveBubbles(page.id, bubbles);
      bubbles = cleaned;
      dirty = false;
      onSaved({ ...page, bubbles: cleaned, status: statusAfterBubbleSave(page.status, !!(translation ?? page.translation), cleaned.length) });
      return true;
    } catch (e) {
      error = String(e);
      return false;
    } finally {
      saving = false;
    }
  }

  /**
   * Translate backend membaca geometri TERSIMPAN (`load_snapshot`), bukan
   * kanvas. Tanpa ini bubble manual (atau hasil Detect yang belum disimpan —
   * tombol Simpan bertanda *) diam-diam tak ikut diterjemahkan.
   */
  async function saveBeforeTranslate(): Promise<boolean> {
    if (!dirty) return true;
    return await save();
  }

  /** Tandai-selesai manual: paksa Sudah walau sebagian bubble kosong (SFX). */
  async function markDone() {
    if (!translation || translation.manualDone) return;
    if (draft.cur) draft.commit();
    savingTr = true;
    error = "";
    try {
      const out = await api.markPageCompleted(page.id, true);
      translation = out;
      onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: "translated", translation: out });
    } catch (e) {
      error = String(e);
    } finally {
      savingTr = false;
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
    if (draft.cur) draft.commit();
    if (!translation) return;
    // Syarat sah resize (bubble-resize-full D1): salin geometri canvas
    // ke translation by index SEBELUM kirim — backend hanya sentuh teks/ai*.
    translation.bubbles = syncGeometryToTranslation(
      bubbles,
      translation.bubbles,
      page.width || 800,
      page.height || 1200,
      // Layout dihitung di sini agar nilai tersimpan = yang terlihat di kanvas.
      (text, w, h, fontSize, shape) => computeTextLayout(text, w, h, textMeasure, { fontSize, shape }),
    );
    if (saveTimer) { clearTimeout(saveTimer); saveTimer = null; }
    const my = ++saveGen;
    savingTr = true;
    error = "";
    try {
      const out = await api.saveTranslation(page.id, translation.bubbles);
      if (my !== saveGen) return; // respons basi: save lebih baru sudah dikirim
      translation = out;
      onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: statusAfterTranslationSave(page.status, out.bubbles, out.manualDone), translation: out });
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
  onDestroy(() => { if (draft.cur) draft.commit(); if (saveTimer) clearTimeout(saveTimer); draft.dispose(); resize.dispose(); });

  /** "Isi manual": translation null → bangun kosong lokal → save_translation. */
  async function createEmptyTranslation() {
    if (translation) return;
    if (bubbles.length === 0) { error = "Belum ada bubble — Detect atau gambar manual dulu."; return; }
    // Validasi ringan: index unik ikut urutan bubble (backend tanpa validasi).
    const idx = bubbles.map((_, i) => i);
    if (new Set(idx).size !== idx.length) { error = "Index bubble duplikat — simpan bubble dulu."; return; }
    const empty = {
      pageId: page.id, targetLang: opts?.targetLang ?? "id", style: opts?.style ?? "standard", model: "",
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
      onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: statusAfterTranslationSave(page.status, out.bubbles, out.manualDone), translation: out });
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
    if (draft.cur) draft.commit();
    translation = {
      ...t,
      bubbles: mergeUserEdits(translation?.bubbles, t.bubbles),
    };
    onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: statusAfterTranslationSave(page.status, translation.bubbles, translation.manualDone), translation });
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
  /** View-model kartu terjemahan (milik TranslationCard): satu kartu aktif + guard tombol. */
  const singleCard = $derived(
    translation
      ? selectedBubble && selectedRow >= 0
        ? { b: selectedBubble, i: selectedRow }
        : translation.bubbles.length === 1
          ? { b: translation.bubbles[0], i: 0 }
          : null
      : null,
  );
  const showMarkDone = $derived(
    !!translation && !translation.manualDone && !translationComplete(translation.bubbles),
  );
  /** Snapshot geometri sekali per render agar kartu tak panggil helper berulang. */
  const geom = $derived.by(() => {
    const idx = selectedIdxForGeometry();
    return { idx, box: idx >= 0 ? bubbles[idx] : null, outOfSync: geometryOutOfSync() };
  });

  function editTr(i: number, field: "translated" | "reading" | "original", v: string) {
    if (!translation) return;
    translation.bubbles = translation.bubbles.map((b, j) =>
      j === i ? { ...b, [field]: v, isUserEdited: true } : b,
    );
    // Tanpa yank: seleksi hanya via selectBubble.
    queueSaveTranslationEdits();
  }

  // Mesin draft kartu (FE-4): cache plain + timer + IME di hook; panel pasok
  // baris aktif, isi komit terakhir, dan commit via editTr existing.
  const draft = useDraftText({
    getRow: () => selectedRow,
    getBubble: (row) => translation?.bubbles[row],
    commit: (row, field, value) => editTr(row, field, value),
  });

  /** Pindah seleksi saat draft kotor → commit otomatis dulu (D2). */
  function selectBubble(idx: number | null) {
    if (draft.cur) draft.commit();
    manualSel = idx;
    if (idx === null) draft.revert();
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
    draft.revert();
    if (!translation) return;
    translation.bubbles = translation.bubbles.map((b, j) =>
      j === i ? { ...b, original: b.aiOriginal, reading: b.aiReading, translated: b.aiTranslated, isUserEdited: false } : b,
    );
    // Sinkron, bukan queueSave: generasi baru membuat respons save lama basi.
    void saveTranslationEdits();
  }

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch {
      appVersion = "";
    }
    const w0 = typeof window !== "undefined" ? window.innerWidth : 0;
    resize.mount(w0);
    try {
      const st = await api.detectStatus();
      detectEngine = st.engine;
    } catch {
      detectEngine = "offline";
    }
  });
</script>

<svelte:window onkeydown={onKey} />

<div class="flex h-full flex-col">
  <EditorToolbar
    fileName={page.path.split(/[/\\]/).pop() ?? page.path}
    filePath={page.path}
    status={page.status}
    detectEngine={detectEngine}
    bubbleCount={bubbles.length}
    mode={mode}
    tool={tool}
    readOnlyMode={readOnlyMode}
    collapsed={resize.collapsed}
    detecting={detecting}
    dirty={dirty}
    saving={saving}
    onClose={onClose}
    onSetMode={(m) => setMode(m)}
    onSetTool={(t) => setTool(t)}
    onToggleCollapse={() => resize.toggleCollapsed()}
    onRunDetect={() => void runDetect()}
    onSave={() => void save()}
  />

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
  <div class="flex min-h-0 flex-1" class:flex-col={resize.columnMode} bind:this={bodyEl}>
    <EditorPanes
      mode={mode}
      columnMode={resize.columnMode}
      shared={{
        imageUrl: fullImageUrl,
        fallbackUrl,
        imgW: page.width || 800,
        imgH: page.height || 1200,
        initial: bubbles,
        readingDir,
        tool,
        translations: translation?.bubbles ?? [],
        externalSelected: manualSel,
        onSelectForward: (i) => selectBubble(i),
        onDrawHint: (m) => { drawHint = m; },
        onChange: (nb) => { bubbles = nb; dirty = true; drawHint = ""; },
      }}
      bind:beforeApi={editorRef}
      bind:afterApi={afterRef}
      bind:beforeScrollEl={beforeScroll}
      bind:afterScrollEl={afterScroll}
      onScroll={(f, t) => syncScroll(f, t)}
      onToolDone={() => setTool("select")}
      onEditTranslation={(i, v) => editTr(i, "translated", v)}
    />
    {#if !resize.collapsed}
      <Divider
        orientation={resize.columnMode ? "horizontal" : "vertical"}
        handle={resize.columnMode && resize.tablet}
        onDown={(e) => resize.down(e)}
        onMove={(e) => resize.move(e)}
        onUp={() => resize.up()}
        onKey={(e) => resize.key(e)}
      />
    <!-- Lebar via inline style (D2): row = px commit, kolom = full + tinggi sheet %. -->
    <aside
      class="flex shrink-0 flex-col overflow-y-auto border-line bg-surface"
      class:w-full={resize.columnMode}
      class:border-t={resize.columnMode}
      class:border-l={!resize.columnMode}
      style:width={!resize.columnMode ? `${resize.displayW}px` : undefined}
      style:height={resize.columnMode ? `${resize.panelHPct}%` : undefined}
    >
      <div class="space-y-5 p-4">
        {#if resize.columnMode && resize.snap === 100}
          <Button variant="ghost" size="sm" onclick={() => { resize.setSnap(40); redrawAfterLayout(); }} title={$t("shrinkSheet")} aria-label={$t("shrinkSheet")}>∨ {$t("shrinkSheet")}</Button>
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
          >{#if batching}<span class="inline-flex items-center gap-1.5"><KuroMascot size={16} />{$t("running")}</span>{:else}Detect halaman ini{/if}</Button>
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
            onProviderChange={(id) => { providerId = id; if (opts) onBatchOpts({ providerId: id, opts }); }}
            onOptsChange={(o) => { opts = o; onBatchOpts({ providerId, opts: o }); }}
            onTranslated={onTranslated}
            onBeforeTranslate={saveBeforeTranslate}
            onError={(m) => {
              error = m;
              onSaved({ ...page, bubbles: $state.snapshot(bubbles), status: "failed" });
            }}
          />
        </section>

        <TranslationCard
          hasTranslation={translation !== null}
          card={singleCard}
          mode={mode}
          trgLabel={trgLabel}
          draft={draft.cur}
          dirtyDraft={draft.dirty}
          glossMsg={glossMsg}
          savingTr={savingTr}
          showMarkDone={showMarkDone}
          hasBubbles={bubbles.length > 0}
          geomBox={geom.box}
          geomIdx={geom.idx}
          geomOutOfSync={geom.outOfSync}
          onSelectBubble={(idx) => selectBubble(manualSel === idx ? null : idx)}
          onResetBubble={(i) => resetBubble(i)}
          onDraftInput={(f, v) => draft.input(f, v)}
          onCompStart={() => draft.compStart()}
          onCompEnd={() => draft.compEnd()}
          onCommit={() => draft.commit()}
          onPressStart={(o, t) => pressStart(o, t)}
          onPressEnd={() => pressEnd()}
          onMarkDone={() => void markDone()}
          onSaveAll={async () => { if (draft.cur) draft.commit(); if (dirty) await save(); await saveTranslationEdits(); }}
          onSetGeometry={(i, p) => setBubbleGeometry(i, p)}
          onScaleBubble={(f) => scaleSelectedBubble(f)}
          onSetFontSize={(v) => setBubbleFontSize(v)}
          onCreateEmpty={() => void createEmptyTranslation()}
        />
      </div>
    </aside>
    {/if}
  </div>
  {/if}
  <footer class="shrink-0 border-t border-line px-5 py-2 text-center text-sm font-semibold text-ink-3">
    Kuron Studio {appVersion ? `v${appVersion}` : ""}
  </footer>
</div>
