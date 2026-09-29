<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import { api } from "../lib/api";
  import CanvasEditor from "../lib/CanvasEditor.svelte";
  import TranslatePanel from "../lib/TranslatePanel.svelte";
  import type { TranslateOpts } from "../lib/TranslatePanel.svelte";
  import { chipNumbers } from "../lib/bubble";
  import { mergeUserEdits } from "../lib/translation";
  import type { BubbleBox, Page, PageTranslation, ReadingDirection, Tool } from "../lib/types";
  import { statusBadgeClass } from "../lib/status";

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
  let tool = $state<Tool>("select");
  let saving = $state(false);
  let detecting = $state(false);
  let batching = $state(false);
  let detectEngine = $state("…");
  let error = $state("");
  let editorRef = $state<{ deleteSelected: () => boolean } | null>(null);

  // M2 translation state.
  let translation = $state<PageTranslation | null>(null);
  let providerId = $state("");
  let lastOpts = $state<TranslateOpts | null>(null);
  let savingTr = $state(false);
  let glossMsg = $state("");
  let manualSel = $state<number | null>(null);

  // Draft lokal kartu (fix-edit-stuck D1): ketikan ditahan di sini, BUKAN di
  // translation.bubbles — jadi value textarea tak di-set ulang per huruf dan
  // kursor stabil. Commit sekali via editTr existing (blur/debounce/pindah).
  let draft = $state<{ row: number; original: string; reading: string; translated: string } | null>(null);
  let draftTimer = $state<ReturnType<typeof setTimeout> | null>(null);

  // Before|Split|After — after-lite: overlay Konva, bukan PNG typeset
  // (ponytail: after-true via command render_page_png bila diprotes).
  type PaneMode = "before" | "split" | "after";
  let mode = $state<PaneMode>("before");
  let beforeScroll: HTMLDivElement | null = $state(null);
  let afterScroll: HTMLDivElement | null = $state(null);
  let syncing = false;
  let saveTimer = $state<ReturnType<typeof setTimeout> | null>(null);
  // §4 last-write-wins: naik tiap kirim; respons hanya menang bila gen-nya kini.
  let saveGen = 0;

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
    if (syncedFor !== page.id || JSON.stringify(incoming) !== JSON.stringify(bubbles)) {
      bubbles = incoming;
      dirty = false;
      syncedFor = page.id;
    }
    // Restore persisted translation (close → reopen); local edits win unless saved.
    if (incomingTr && JSON.stringify(translation) !== incomingTr) {
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
      if (editorRef?.deleteSelected()) void save();
    } else if (e.key === "s" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      void save();
    } else if (e.key === "Escape") {
      onClose();
    }
  }

  async function saveTranslationEdits() {
    if (draft) commitDraft();
    if (!translation) return;
    if (saveTimer) { clearTimeout(saveTimer); saveTimer = null; }
    const my = ++saveGen;
    savingTr = true;
    error = "";
    try {
      const out = await api.saveTranslation(page.id, translation.bubbles);
      if (my !== saveGen) return; // respons basi: save lebih baru sudah dikirim
      translation = out;
      onSaved({ ...page, status: "translated", translation: out });
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

  onDestroy(() => { if (saveTimer) clearTimeout(saveTimer); if (draftTimer) clearTimeout(draftTimer); });

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
        index: i, x: b.x, y: b.y, w: b.w, h: b.h,
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
      onSaved({ ...page, status: "translated", translation: out });
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
    if (draft) commitDraft();
    translation = {
      ...t,
      bubbles: mergeUserEdits(translation?.bubbles, t.bubbles),
    };
    onSaved({ ...page, status: "translated", translation });
  }

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
    manualSel = translation.bubbles[i]?.index ?? null;
    queueSaveTranslationEdits();
  }

  type DraftField = "original" | "reading" | "translated";

  /** Tulis ketikan kartu ke draft saja — translation.bubbles tak tersentuh. */
  function draftInput(field: DraftField, v: string) {
    if (!translation || selectedRow < 0) return;
    if (!draft || draft.row !== selectedRow) {
      const cur = translation.bubbles[selectedRow];
      if (!cur) return;
      draft = { row: selectedRow, original: cur.original, reading: cur.reading, translated: cur.translated };
    }
    draft[field] = v;
    // Ketik-tanpa-blur: debounce ~500ms commit (ritme autosave existing).
    if (draftTimer) clearTimeout(draftTimer);
    draftTimer = setTimeout(() => { draftTimer = null; commitDraft(); }, 500);
  }

  /** Commit field kotor via editTr existing (sekali, bukan per huruf). */
  function commitDraft() {
    if (draftTimer) { clearTimeout(draftTimer); draftTimer = null; }
    if (!draft || !translation) return;
    const { row, original, reading, translated } = draft;
    draft = null;
    const cur = translation.bubbles[row];
    if (!cur) return;
    if (cur.original !== original) editTr(row, "original", original);
    if (cur.reading !== reading) editTr(row, "reading", reading);
    if (cur.translated !== translated) editTr(row, "translated", translated);
  }

  /** Pindah seleksi saat draft kotor → commit otomatis dulu (D2). */
  function selectBubble(idx: number | null) {
    if (draft) commitDraft();
    manualSel = idx;
    if (idx === null) draft = null;
  }

  /** ↺ opsi A: current = baseline AI terakhir, kirim LANGSUNG (§4 D7). */
  function resetBubble(i: number) {
    if (draftTimer) { clearTimeout(draftTimer); draftTimer = null; }
    draft = null;
    if (!translation) return;
    translation.bubbles = translation.bubbles.map((b, j) =>
      j === i ? { ...b, original: b.aiOriginal, reading: b.aiReading, translated: b.aiTranslated, isUserEdited: false } : b,
    );
    // Sinkron, bukan queueSave: generasi baru membuat respons save lama basi.
    void saveTranslationEdits();
  }

  onMount(async () => {
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
  <div class="flex flex-wrap items-center gap-2 border-b border-zinc-800 px-4 py-2 text-sm">
    <button class="rounded bg-zinc-800 px-2 py-1 hover:bg-zinc-700" onclick={onClose}>← Grid</button>
    <span class="font-semibold">{page.path.split(/[/\\]/).pop()}</span>
    <span class={`rounded px-1.5 py-0.5 text-[10px] font-semibold text-white ${statusBadgeClass(page.status)}`}>{page.status}</span>
    <span class="text-xs text-zinc-500">engine: {detectEngine} · {bubbles.length} bubble</span>
    <div class="ml-auto flex items-center gap-1">
      <div class="mr-1 flex overflow-hidden rounded text-[11px]">
        {#each [["before", "Before"], ["split", "Split"], ["after", "After"]] as [m, label]}
          <button
            class={`px-2 py-1 ${mode === m ? "bg-sky-600 font-semibold text-white" : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"}`}
            onclick={() => (mode = m as PaneMode)}
          >{label}</button>
        {/each}
      </div>
      {#each [["select", "Pilih"], ["rect", "Rect"], ["ellipse", "Elips"], ["freeform", "Bebas"], ["tail", "Ekor"]] as [t, label]}
        <button
          class={`rounded px-2 py-1 ${tool === t ? "bg-emerald-600 font-semibold" : "bg-zinc-800 hover:bg-zinc-700"}`}
          onclick={() => (tool = t as Tool)}
        >{label}</button>
      {/each}
      <button class="rounded bg-sky-700 px-2 py-1 hover:bg-sky-600 disabled:opacity-50" onclick={runDetect} disabled={detecting}>
        {detecting ? "Detect…" : "Detect"}
      </button>
      <button
        class="rounded bg-emerald-600 px-3 py-1 font-semibold hover:bg-emerald-500 disabled:opacity-50"
        onclick={save}
        disabled={saving || !dirty}
      >{saving ? "Simpan…" : dirty ? "Simpan*" : "Tersimpan"}</button>
    </div>
  </div>

  {#if openError}
    <p class="border-b border-red-900 bg-red-950 px-4 py-1.5 text-xs text-red-200">{openError}</p>
  {/if}
  {#if error}
    <p class="border-b border-amber-900 bg-amber-950 px-4 py-1.5 text-xs text-amber-200">{error}</p>
  {/if}

  {#if openError && !fullImageUrl}
    <div class="flex flex-1 items-center justify-center p-8">
      <div class="max-w-sm text-center text-sm text-zinc-400">
        <p class="mb-2 font-semibold text-red-200">Gambar gagal dimuat.</p>
        <p class="mb-4">Kembali ke grid dan coba lagi — tidak perlu reload.</p>
        <button class="rounded bg-zinc-800 px-3 py-1.5 hover:bg-zinc-700" onclick={onClose}>← Grid</button>
      </div>
    </div>
  {:else}
  <div class="flex min-h-0 flex-1">
    {#if mode === "before" || mode === "split"}
      <div
        class="min-w-0 flex-1 overflow-auto p-4"
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
          externalSelected={manualSel}
          onSelectForward={(i) => selectBubble(i)}
          onChange={(nb) => {
            bubbles = nb;
            dirty = true;
          }}
        />
      </div>
    {/if}
    {#if mode === "after" || mode === "split"}
      <div
        class="min-w-0 flex-1 overflow-auto p-4"
        class:border-l={mode === "split"}
        class:border-zinc-800={mode === "split"}
        bind:this={afterScroll}
        onscroll={() => syncScroll(afterScroll, beforeScroll)}
      >
        <CanvasEditor
          imageUrl={fullImageUrl}
          fallbackUrl={fallbackUrl}
          imgW={page.width || 800}
          imgH={page.height || 1200}
          initial={bubbles}
          {readingDir}
          tool="select"
          translations={translation?.bubbles ?? []}
          showTranslation={true}
          editable={false}
          externalSelected={manualSel}
          onSelectForward={(i) => selectBubble(i)}
          onEditTranslation={(i, v) => editTr(i, "translated", v)}
          onChange={() => {}}
        />
      </div>
    {/if}
    <aside class="w-64 shrink-0 overflow-auto border-l border-zinc-800 p-3 text-xs">
      <h3 class="mb-2 font-semibold text-zinc-300">Bubble ({bubbles.length})</h3>
      {#if bubbles.length === 0}
        <p class="text-zinc-500">Belum ada bubble. Klik Detect atau gambar manual.</p>
      {:else}
        <ol class="space-y-1">
          {#each bubbles as b, i (i)}
            <li class="flex items-center gap-2 rounded bg-zinc-900 px-2 py-1">
              <span class="flex h-5 w-5 items-center justify-center rounded-full bg-emerald-600 text-[10px] font-bold">{nums.get(i) ?? i + 1}</span>
              <span class="text-zinc-400">{b.kind ?? "rect"} {b.w}×{b.h}</span>
              <button
                class="ml-auto text-rose-400 hover:text-rose-300 disabled:opacity-50"
                disabled={saving}
                onclick={() => {
                  bubbles.splice(i, 1);
                  bubbles = [...bubbles];
                  dirty = true;
                  void save();
                }}
              >hapus</button>
            </li>
          {/each}
        </ol>
      {/if}
      <h3 class="mb-1 mt-4 font-semibold text-zinc-300">Batch</h3>
      <button
        class="w-full rounded bg-zinc-800 px-2 py-1 hover:bg-zinc-700 disabled:opacity-50"
        onclick={() => runDetectAll([page.id])}
        disabled={batching}
      >{batching ? "Jalan…" : "Detect halaman ini"}</button>
      <p class="mt-2 text-[11px] text-zinc-500">Hapus: pilih bubble + Delete. Simpan: Ctrl+S.</p>

      <div class="mt-3 border-t border-zinc-800 pt-2">
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
            onSaved({ ...page, status: "failed" });
          }}
        />
      </div>

      {#if translation}
        <div class="mt-2 flex items-center gap-2">
          <span class={`rounded px-1.5 py-0.5 text-[10px] font-semibold text-white ${statusBadgeClass("translated")}`}>translated</span>
          <button
            class="ml-auto rounded bg-emerald-600 px-2 py-0.5 font-semibold hover:bg-emerald-500 disabled:opacity-50"
            onclick={() => void saveTranslationEdits()} disabled={savingTr}
          >{savingTr ? "Simpan…" : "Save edits"}</button>
        </div>
        {#if mode !== "before"}
          <p class="mt-1 text-[10px] text-zinc-500">Double-klik bubble di After untuk edit langsung.</p>
        {/if}
        <ol class="mt-2 space-y-1.5">
          {#if selectedBubble && selectedRow >= 0}
            {@const b = selectedBubble}
            {@const i = selectedRow}
            <li
              class="rounded bg-zinc-900 p-1.5 ring-1 ring-emerald-500"
              onpointerdown={() => pressStart(b.original, b.translated)}
              onpointerup={pressEnd}
              onpointerleave={pressEnd}
              title="Tahan 0.5 dtk untuk simpan ke Glossary"
            >
              <div class="mb-0.5 flex items-center gap-1 text-[10px] text-zinc-500">
                <button
                  class="rounded px-1 hover:bg-zinc-700 hover:text-zinc-200"
                  onclick={() => selectBubble(manualSel === b.index ? null : b.index)}
                  title="Pilih bubble ini"
                >#{b.index + 1}</button>
                {#if b.isUserEdited}<span class="rounded bg-amber-700 px-1 text-white">edited</span>{/if}
                {#if b.isUserEdited && b.aiTranslated}
                  <button
                    class="ml-auto rounded px-1 hover:bg-zinc-700 hover:text-zinc-200"
                    onclick={() => resetBubble(i)}
                    title="Kembalikan ke hasil AI terakhir"
                  >↺</button>
                {/if}
              </div>
              <p class="mt-0.5 block text-[10px] text-zinc-500">Original (JP)</p>
              <textarea
                rows="2"
                class="w-full rounded bg-zinc-800 px-1.5 py-1 text-xs text-zinc-100 outline-none focus:ring-1 focus:ring-emerald-500"
                value={draft && draft.row === i ? draft.original : b.original}
                oninput={(e) => draftInput("original", (e.target as HTMLTextAreaElement).value)}
                onblur={commitDraft}
              ></textarea>
              <p class="mt-0.5 block text-[10px] text-zinc-500">Reading</p>
              <textarea
                rows="1"
                class="w-full rounded bg-zinc-800 px-1.5 py-1 text-xs text-zinc-100 outline-none focus:ring-1 focus:ring-emerald-500"
                value={draft && draft.row === i ? draft.reading : b.reading}
                oninput={(e) => draftInput("reading", (e.target as HTMLTextAreaElement).value)}
                onblur={commitDraft}
              ></textarea>
              <p class="mt-0.5 block text-[10px] text-zinc-500">Terjemahan (ID)</p>
              <textarea
                rows="2"
                class="w-full rounded bg-zinc-800 px-1.5 py-1 text-xs text-zinc-100 outline-none focus:ring-1 focus:ring-emerald-500"
                value={draft && draft.row === i ? draft.translated : b.translated}
                oninput={(e) => draftInput("translated", (e.target as HTMLTextAreaElement).value)}
                onblur={commitDraft}
              ></textarea>
              {#if glossMsg}<p class="mt-0.5 text-[10px] text-emerald-300">{glossMsg}</p>{/if}
            </li>
          {:else}
            <p class="rounded bg-zinc-900 p-2 text-zinc-500">Klik / double-klik bubble di After untuk memilih.</p>
          {/if}
        </ol>
      {:else}
        <div class="mt-2">
          <button
            class="w-full rounded bg-sky-700 px-2 py-1 font-semibold hover:bg-sky-600 disabled:opacity-50"
            onclick={() => void createEmptyTranslation()} disabled={savingTr || bubbles.length === 0}
          >{savingTr ? "Simpan…" : "Isi manual"}</button>
          <p class="mt-1 text-[10px] text-zinc-500">Buat translation kosong tanpa AI, lalu ketik per bubble.</p>
        </div>
      {/if}
    </aside>
  </div>
  {/if}
</div>
