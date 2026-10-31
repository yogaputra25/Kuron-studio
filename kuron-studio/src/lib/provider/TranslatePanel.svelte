<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../contracts/api";
  import { TARGET_LANGS, TRANSLATE_STYLES } from "../contracts/types";
  import type { AiModelOption, MosaicQuality, PageTranslation, ProviderView, ReadingDirection, TranslateStyle } from "../contracts/types";
  import { t } from "../app/i18n";
  import Button from "../ui/Button.svelte";
  import Lov from "../ui/Lov.svelte";

  import type { TranslateOpts } from "./translateOpts";
  import { toTranslateArgs } from "./translateOpts";

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
  let customPrompt = $state("");
  /** Model per-run: tidak disimpan ke DB, default = simpanan provider. */
  let model = $state("");
  let models = $state<AiModelOption[]>([]);
  let modelsMsg = $state("");
  let glossaryPreview = $state<string | null>(null);
  let busy = $state(false);
  let cancelled = $state(false);
  let info = $state("");

  // Angkat opts ke parent (Batch/Review) + preview glossary auto-context.
  $effect(() => {
    targetLang; style; skipSfx; mosaicQuality; customPrompt; model;
    onOptsChange({ targetLang, style, skipSfx, mosaicQuality, customPrompt: customPrompt.trim(), model });
  });

  /** Daftar model sesi-saja: ganti provider → list ulang, default = simpanan. */
  async function loadModelsFor(pid: string) {
    models = []; modelsMsg = "";
    if (!pid) { model = ""; return; }
    try {
      models = await api.listModels(pid);
      const saved = providers.find((p) => p.id === pid)?.model ?? "";
      model = saved || (models.find((m) => m.vision) ?? models[0])?.id || "";
      modelsMsg = models.length > 0 ? `${models.length} model.` : "Provider tidak mengembalikan model.";
    } catch (e) {
      modelsMsg = String(e);
    }
  }

  $effect(() => {
    void loadModelsFor(providerId);
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
    // Provider mungkin sudah terpilih sebelum daftar tiba: pastikan default.
    if (providerId) await loadModelsFor(providerId);
  });

  async function translate() {
    if (!providerId) { onError("Pilih provider dulu (tambah via Providers)."); return; }
    busy = true; cancelled = false; info = "";
    const t0 = performance.now();
    try {
      const t = await api.translatePage(toTranslateArgs(
        { targetLang, style, skipSfx, mosaicQuality, customPrompt, model },
        { pageId, providerId, readingDirection: readingDir, glossary },
      ));
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

<div class="rounded-lg border border-line bg-surface-2/60 p-4">
  <h3 class="font-display mb-4 flex items-baseline gap-1.5 text-sm font-semibold text-ink">
    Translate
    <span class="tnum text-xs font-normal text-ink-3">· {bubblesCount} bubble</span>
  </h3>

  <div class="space-y-4">
    <Lov
      label="Provider"
      value={providerId}
      onchange={(v) => onProviderChange(v)}
      hint={providerId ? undefined : "Wajib pilih provider sebelum translate."}
      options={[{ value: "", label: "— pilih —" }, ...providers.map((p) => ({ value: p.id, label: `${p.name} (${p.model})` }))]}
    />

    {#if providerId}
      <Lov
        label="Model"
        value={model}
        onchange={(v) => (model = v)}
        hint={modelsMsg}
        options={models.map((m) => ({ value: m.id, label: `${m.label}${m.vision ? " · vision" : ""}` }))}
      />
    {/if}

    <div class="grid grid-cols-2 gap-3">
      <Lov label="Target" value={targetLang} onchange={(v) => (targetLang = v)} options={TARGET_LANGS} />
      <Lov label="Style" value={style} onchange={(v) => (style = v as TranslateStyle)} options={TRANSLATE_STYLES} />
    </div>

    <details class="group rounded-md border border-line bg-surface-2/60">
      <summary
        class="cursor-pointer list-none px-3 py-2 text-[11px] font-medium text-ink-2 select-none
               transition-colors hover:text-ink"
      >
        Opsi lanjutan
      </summary>
      <div class="space-y-4 border-t border-line px-3 py-3">
        <div class="grid grid-cols-2 gap-3">
          <Lov
            label="Mosaic"
            value={mosaicQuality}
            onchange={(v) => (mosaicQuality = v as MosaicQuality)}
            hint={mosaicQuality === "low" ? "±1MB, 75 quality" : "±2MB, 85 quality"}
            options={[{ value: "low", label: "low" }, { value: "high", label: "high" }]}
          />
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

        <div class="flex flex-col gap-1.5">
          <label
            for="custom-prompt"
            class="text-[11px] font-medium tracking-wide text-ink-2 uppercase"
          >
            {$t("customPrompt")}
          </label>
          <textarea
            id="custom-prompt"
            class="w-full rounded-md border border-line bg-surface-2 px-3 py-2.5 text-[12px]
                   leading-relaxed text-ink placeholder:text-ink-3 outline-none transition-colors
                   focus:border-accent focus:ring-2 focus:ring-accent/25"
            rows="2"
            maxlength="2000"
            bind:value={customPrompt}
            placeholder={$t("customPromptPh")}
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
      </div>
    </details>

    <div class="sticky bottom-0 -mx-4 -mb-4 border-t border-line bg-surface-2 px-4 py-3">
      <Button
        variant="primary"
        block
        onclick={busy ? cancel : translate}
        disabled={!busy && !providerId}
      >{busy ? "Batal" : "Translate"}</Button>

      {#if info}
        <p role="status" class="mt-2 text-[11px] leading-relaxed text-ink-3">{info}</p>
      {/if}
    </div>
  </div>
</div>
