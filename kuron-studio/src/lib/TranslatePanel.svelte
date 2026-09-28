<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./api";
  import { TARGET_LANGS, TRANSLATE_STYLES } from "./types";
  import type { MosaicQuality, PageTranslation, ProviderView, ReadingDirection, TranslateStyle } from "./types";
  import Button from "./ui/Button.svelte";

  import Select from "./ui/Select.svelte";

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
    busy = true; info = "";
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
      info = `${t.model} · ${t.bubbles.length} bubble · ${((performance.now() - t0) / 1000).toFixed(1)}s`;
      onTranslated(t);
    } catch (e) {
      onError(String(e));
    } finally {
      busy = false;
    }
  }
</script>

<div class="rounded-lg border border-line bg-surface-2/60 p-4">
  <h3 class="font-display mb-4 flex items-baseline gap-1.5 text-sm font-semibold text-ink">
    Translate
    <span class="tnum text-xs font-normal text-ink-3">· {bubblesCount} bubble</span>
  </h3>

  <div class="space-y-4">
    <Select
      label="Provider"
      value={providerId}
      onchange={(e) => onProviderChange((e.target as HTMLSelectElement).value)}
      hint={providerId ? undefined : "Wajib pilih provider sebelum translate."}
    >
      <option value="">— pilih —</option>
      {#each providers as p (p.id)}<option value={p.id}>{p.name} ({p.model})</option>{/each}
    </Select>

    <div class="grid grid-cols-2 gap-3">
      <Select label="Target" bind:value={targetLang}>
        {#each TARGET_LANGS as l (l.value)}<option value={l.value}>{l.label}</option>{/each}
      </Select>
      <Select label="Style" bind:value={style}>
        {#each TRANSLATE_STYLES as s (s.value)}<option value={s.value}>{s.label}</option>{/each}
      </Select>
    </div>

    <div class="grid grid-cols-2 gap-3">
      <Select
        label="Mosaic"
        bind:value={mosaicQuality}
        hint={mosaicQuality === "low" ? "±1MB, 75 quality" : "±2MB, 85 quality"}
      >
        <option value="low">low</option>
        <option value="high">high</option>
      </Select>
      <!-- Checkbox disejajarkan dengan control Mosaic (h-9) supaya tepinya
           rata — label::before pseudo-elemen memberi tinggi yang sama. -->
      <label
        class="flex cursor-pointer items-center gap-2.5 self-end rounded-md border border-line
               bg-surface-2 px-3 text-[13px] text-ink-2 transition-colors hover:border-line-strong"
      >
        <input type="checkbox" class="size-4 shrink-0 accent-accent" bind:checked={skipSfx} />
        Skip SFX
      </label>
    </div>

    <div class="flex flex-col gap-1.5">
      <label
        for="glossary-override"
        class="text-[11px] font-medium tracking-wide text-ink-2 uppercase"
      >
        Glossary override
      </label>
      <textarea
        id="glossary-override"
        class="w-full rounded-md border border-line bg-surface-2 px-3 py-2.5 font-mono text-[12px]
               leading-relaxed text-ink placeholder:text-ink-3 outline-none transition-colors
               focus:border-accent focus:ring-2 focus:ring-accent/25"
        rows="2"
        bind:value={glossary}
        placeholder="Auto dari Glossary DB — override manual di sini"
      ></textarea>
    </div>

    {#if glossaryPreview}
      <details class="group rounded-md border border-line bg-surface-2/60">
        <summary
          class="cursor-pointer list-none px-3 py-2 text-[11px] font-medium text-ink-2 select-none
                 transition-colors hover:text-ink"
        >
          Auto-context dari Glossary ({glossaryPreview.split("\n").length} baris)
        </summary>
        <pre
          class="max-h-40 overflow-auto whitespace-pre-wrap border-t border-line px-3 py-2.5
                 font-mono text-[11px] leading-relaxed text-ink-3"
          title="Auto-context glossary">{glossaryPreview}</pre>
      </details>
    {/if}

    <Button
      variant="primary"
      block
      class="pt-1"
      onclick={translate}
      disabled={busy || !providerId}
    >{busy ? "Translate…" : "Translate"}</Button>

    {#if info}
      <p role="status" class="text-[11px] leading-relaxed text-ink-3">{info}</p>
    {/if}
  </div>
</div>
