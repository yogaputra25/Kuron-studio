<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { api } from "./api";
  import { initialBatchState, reduceProgress } from "./batch";
  import type { BatchOpts, Page, TranslateProgress } from "./types";

  interface Props {
    pages: Page[];
    opts: BatchOpts;
    onDone: () => void;
    onClose: () => void;
  }

  let { pages, opts, onDone, onClose }: Props = $props();

  let running = $state(false);
  const pageCount = $derived(pages.length);
  // ponytail: jangan namai `state` — svelte-check mengacaukannya dengan rune $state.
  let batch = $state(initialBatchState(pageCount));
  let error = $state("");

  const targets = $derived(pages.map((p) => p.id));
  const pct = $derived(batch.total > 0 ? Math.round((batch.done / batch.total) * 100) : 0);

  onMount(async () => {
    await listen<TranslateProgress>("translate_progress", (e) => {
      batch = reduceProgress(batch, e.payload);
    });
  });

  async function run(ids: string[]) {
    running = true;
    error = "";
    batch = initialBatchState(ids.length);
    try {
      await api.translateBatch({
        pageIds: ids,
        providerId: opts.providerId,
        targetLang: opts.targetLang,
        style: opts.style,
        skipSfx: opts.skipSfx,
        mosaicQuality: opts.mosaicQuality,
        readingDirection: opts.readingDirection,
      });
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
      onDone();
    }
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" role="dialog" aria-modal="true">
  <div class="w-full max-w-lg rounded-lg border border-zinc-800 bg-zinc-950 p-4 text-sm text-zinc-100">
    <div class="mb-2 flex items-center gap-2">
      <h2 class="font-bold">Batch translate ({targets.length} halaman · 3 paralel)</h2>
      <button class="ml-auto rounded bg-zinc-800 px-2 py-1 hover:bg-zinc-700" onclick={onClose}>Tutup</button>
    </div>
    <div class="mb-1 h-2 overflow-hidden rounded bg-zinc-800">
      <div class="h-full bg-emerald-500 transition-all" style={`width: ${pct}%`}></div>
    </div>
    <p class="mb-2 text-xs text-zinc-400">{batch.done}/{batch.total} · gagal: {batch.failed.length}</p>
    {#if error}<p class="mb-2 rounded bg-amber-950 px-2 py-1 text-xs text-amber-200">{error}</p>{/if}
    {#if batch.failed.length > 0}
      <ul class="mb-2 space-y-1">
        {#each batch.failed as id (id)}
          <li class="rounded bg-rose-950 px-2 py-1 text-xs text-rose-200">
            {pages.find((p) => p.id === id)?.path.split(/[/\\]/).pop() ?? id}: {batch.messages[id] ?? ""}
          </li>
        {/each}
      </ul>
      <button
        class="mb-2 w-full rounded bg-amber-700 px-2 py-1 text-xs font-semibold hover:bg-amber-600 disabled:opacity-50"
        onclick={() => run(batch.failed)} disabled={running}
      >Retry yang gagal ({batch.failed.length})</button>
    {/if}
    <button
      class="w-full rounded bg-emerald-600 px-2 py-1.5 font-semibold hover:bg-emerald-500 disabled:opacity-50"
      onclick={() => run(targets)} disabled={running || targets.length === 0}
    >{running ? `Jalan… ${pct}%` : `Translate ${targets.length} halaman`}</button>
  </div>
</div>
