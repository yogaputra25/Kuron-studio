<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./api";
  import { TARGET_LANGS, TRANSLATE_STYLES } from "./types";
  import type { MosaicQuality, PageTranslation, ProviderView, ReadingDirection, TranslateStyle } from "./types";

  export interface TranslateOpts {
    targetLang: string;
    style: TranslateStyle;
    skipSfx: boolean;
    mosaicQuality: MosaicQuality;
  }

  interface Props {
    pageId: string;
    providerId: string;
    bubblesCount: number;
    readingDir: ReadingDirection;
    originals: string[];
    onProviderChange: (id: string) => void;
    onOptsChange: (o: TranslateOpts) => void;
    onTranslated: (t: PageTranslation) => void;
    onError: (msg: string) => void;
  }

  let { pageId, providerId, bubblesCount, readingDir, originals, onProviderChange, onOptsChange, onTranslated, onError }: Props = $props();

  let providers = $state<ProviderView[]>([]);
  let targetLang = $state("id");
  let style = $state<TranslateStyle>("standard");
  let skipSfx = $state(true);
  let mosaicQuality = $state<MosaicQuality>("low");
  let glossary = $state("");
  let glossaryPreview = $state<string | null>(null);
  let busy = $state(false);
  let cancelled = $state(false);
  let info = $state("");

  // Angkat opts ke parent (Batch/Review) + preview glossary auto-context.
  $effect(() => {
    targetLang; style; skipSfx; mosaicQuality;
    onOptsChange({ targetLang, style, skipSfx, mosaicQuality });
  });

  $effect(() => {
    void originals;
    void glossary;
    if (glossary.trim()) { glossaryPreview = null; return; }
    api.glossaryContext(originals ?? []).then(
      (v) => { glossaryPreview = v; },
      () => { glossaryPreview = null; },
    );
  });

  onMount(async () => {
    try {
      providers = await api.getProviders();
    } catch (e) {
      onError(String(e));
    }
  });

  async function translate() {
    if (!providerId) { onError("Pilih provider dulu (tambah via Providers)."); return; }
    busy = true; cancelled = false; info = "";
    const t0 = performance.now();
    try {
      const t = await api.translatePage({
        pageId,
        providerId,
        targetLang,
        style,
        skipSfx,
        mosaicQuality,
        readingDirection: readingDir,
        ...(glossary.trim() ? { glossary: glossary.trim() } : {}),
      });
      if (cancelled) return; // response telat pasca-Batal: buang, tanpa persist ganda
      info = `${t.model} · ${t.bubbles.length} bubble · ${((performance.now() - t0) / 1000).toFixed(1)}s`;
      onTranslated(t);
    } catch (e) {
      if (cancelled) return; // reject telat: tanpa banner error
      onError(String(e));
    } finally {
      if (!cancelled) busy = false;
    }
  }

  // Batal (opsi B cancel): lepas UI seketika + minta backend berhenti.
  async function cancel() {
    cancelled = true;
    busy = false;
    info = "Dibatalkan";
    try {
      await api.cancelTranslate(pageId);
    } catch {
      // Backend tak sempat klaim (belum ada flag): UI tetap lepas.
    }
  }
</script>

<div class="rounded border border-zinc-800 bg-zinc-900 p-2">
  <h3 class="mb-1 font-semibold text-zinc-300">Translate ({bubblesCount})</h3>
  <label class="mb-1 block text-[11px] text-zinc-400">Provider
    <select class="mt-0.5 w-full rounded bg-zinc-800 px-1.5 py-1 text-xs text-zinc-100" value={providerId} onchange={(e) => onProviderChange((e.target as HTMLSelectElement).value)}>
      <option value="">— pilih —</option>
      {#each providers as p (p.id)}<option value={p.id}>{p.name} ({p.model})</option>{/each}
    </select>
  </label>
  <div class="grid grid-cols-2 gap-1">
    <label class="text-[11px] text-zinc-400">Target
      <select class="mt-0.5 w-full rounded bg-zinc-800 px-1.5 py-1 text-xs text-zinc-100" bind:value={targetLang}>
        {#each TARGET_LANGS as l (l.value)}<option value={l.value}>{l.label}</option>{/each}
      </select>
    </label>
    <label class="text-[11px] text-zinc-400">Style
      <select class="mt-0.5 w-full rounded bg-zinc-800 px-1.5 py-1 text-xs text-zinc-100" bind:value={style}>
        {#each TRANSLATE_STYLES as s (s.value)}<option value={s.value}>{s.label}</option>{/each}
      </select>
    </label>
  </div>
  <div class="mt-1 grid grid-cols-2 gap-1">
    <label class="flex items-center gap-1 text-[11px] text-zinc-400">
      <input type="checkbox" bind:checked={skipSfx} /> Skip SFX
    </label>
    <label class="text-[11px] text-zinc-400">Mosaic
      <select class="ml-1 rounded bg-zinc-800 px-1 py-0.5 text-xs text-zinc-100" bind:value={mosaicQuality}>
        <option value="low">low</option>
        <option value="high">high</option>
      </select>
    </label>
  </div>
  <textarea
    class="mt-1 w-full rounded bg-zinc-800 px-1.5 py-1 text-[11px] text-zinc-100 outline-none focus:ring-1 focus:ring-emerald-500"
    rows="2" bind:value={glossary} placeholder="Auto dari Glossary DB — override manual di sini"
  ></textarea>
  {#if glossaryPreview}
    <p class="mt-1 whitespace-pre-wrap font-mono text-[10px] text-emerald-300/80" title="Auto-context glossary">{glossaryPreview}</p>
  {/if}
  <button
    class="mt-1 w-full rounded bg-emerald-600 px-2 py-1 text-xs font-semibold hover:bg-emerald-500 disabled:opacity-50"
    onclick={busy ? cancel : translate} disabled={!busy && !providerId}
  >{busy ? "Batal" : "Translate"}</button>
  {#if info}<p class="mt-1 text-[10px] text-zinc-500">{info}</p>{/if}
</div>
