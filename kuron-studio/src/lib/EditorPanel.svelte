<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
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
  let showTranslation = $state(false);
  let savingTr = $state(false);
  let glossMsg = $state("");

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
      showTranslation = !!translation;
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
    if (!translation) return;
    savingTr = true;
    error = "";
    try {
      const out = await api.saveTranslation(page.id, translation.bubbles);
      translation = out;
      onSaved({ ...page, status: "translated", translation: out });
    } catch (e) {
      error = String(e);
    } finally {
      savingTr = false;
    }
  }

  function onTranslated(t: PageTranslation) {
    translation = {
      ...t,
      bubbles: mergeUserEdits(translation?.bubbles, t.bubbles),
    };
    showTranslation = true;
    onSaved({ ...page, status: "translated", translation });
  }

  function editTr(i: number, field: "translated" | "reading" | "original", v: string) {
    if (!translation) return;
    translation.bubbles = translation.bubbles.map((b, j) =>
      j === i ? { ...b, [field]: v, isUserEdited: field === "translated" ? true : b.isUserEdited } : b,
    );
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
    <div class="min-w-0 flex-1 overflow-auto p-4">
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
        {showTranslation}
        onChange={(nb) => {
          bubbles = nb;
          dirty = true;
        }}
      />
    </div>
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
            class={`rounded px-2 py-0.5 ${showTranslation ? "bg-emerald-600" : "bg-zinc-800 hover:bg-zinc-700"}`}
            onclick={() => (showTranslation = !showTranslation)}
          >Overlay T</button>
          <button
            class="ml-auto rounded bg-emerald-600 px-2 py-0.5 font-semibold hover:bg-emerald-500 disabled:opacity-50"
            onclick={saveTranslationEdits} disabled={savingTr}
          >{savingTr ? "Simpan…" : "Save edits"}</button>
        </div>
        <ol class="mt-2 space-y-1.5">
          {#each translation.bubbles as b, i (b.index)}
            <li
              class="rounded bg-zinc-900 p-1.5"
              onpointerdown={() => pressStart(b.original, b.translated)}
              onpointerup={pressEnd}
              onpointerleave={pressEnd}
              title="Tahan 0.5 dtk untuk simpan ke Glossary"
            >
              <div class="mb-0.5 flex items-center gap-1 text-[10px] text-zinc-500">
                <span>#{b.index + 1}</span>
                {#if b.isUserEdited}<span class="rounded bg-amber-700 px-1 text-white">edited</span>{/if}
              </div>
              <p class="truncate text-zinc-500" title={b.original}>O: {b.original || "—"}</p>
              {#if b.reading}<p class="truncate text-zinc-400" title={b.reading}>R: {b.reading}</p>{/if}
              <input
                class="mt-0.5 w-full rounded bg-zinc-800 px-1.5 py-1 text-xs text-zinc-100 outline-none focus:ring-1 focus:ring-emerald-500"
                value={b.translated}
                oninput={(e) => editTr(i, "translated", (e.target as HTMLInputElement).value)}
              />
              {#if glossMsg}<p class="mt-0.5 text-[10px] text-emerald-300">{glossMsg}</p>{/if}
            </li>
          {/each}
        </ol>
      {/if}
    </aside>
  </div>
  {/if}
</div>
