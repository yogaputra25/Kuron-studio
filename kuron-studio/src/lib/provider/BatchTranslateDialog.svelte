<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../contracts/api";
  import { TARGET_LANGS, TRANSLATE_STYLES } from "../contracts/types";
  import type {
    AiModelOption,
    MosaicQuality,
    Page,
    Project,
    ProviderView,
    ReadingDirection,
    TranslateStyle,
  } from "../contracts/types";
  import { t } from "../app/i18n";
  import { read, write, STORAGE_KEYS } from "../app/storage";
  import Button from "../ui/Button.svelte";
  import Lov from "../ui/Lov.svelte";
  import Panel from "../ui/Panel.svelte";
  import { allPages, subtreePages } from "../library/library";
  import type { TranslateOpts } from "./translateOpts";

  interface SubmitReq {
    providerId: string;
    opts: TranslateOpts;
  }

  interface Props {
    project: Project;
    selectedIds: string[];
    folderId: string | "all";
    visiblePages: Page[];
    readingDirection: ReadingDirection;
    onSubmit: (req: SubmitReq, pageIds: string[]) => void;
    onClose: () => void;
  }
  // readingDirection merge tetap di OverlayHost (batchOpts); prop dipertahankan untuk kontrak.
  let { project, selectedIds, folderId, visiblePages, onSubmit, onClose }: Props = $props();

  const batchProviderKey = STORAGE_KEYS.batchProvider;
  const batchOptsKey = STORAGE_KEYS.batchOpts;
  const FALLBACK_OPTS: TranslateOpts = {
    targetLang: "id",
    style: "standard",
    skipSfx: true,
    mosaicQuality: "low",
    customPrompt: "",
    model: "",
  };

  const savedOpts = read(batchOptsKey, FALLBACK_OPTS);

  let providers = $state<ProviderView[]>([]);
  let providerId = $state(read(batchProviderKey, ""));
  let models = $state<AiModelOption[]>([]);
  let modelsMsg = $state("");
  let model = $state(savedOpts.model);
  let targetLang = $state(savedOpts.targetLang);
  let style = $state<TranslateStyle>(savedOpts.style as TranslateStyle);
  let skipSfx = $state(savedOpts.skipSfx);
  let mosaicQuality = $state<MosaicQuality>(savedOpts.mosaicQuality as MosaicQuality);
  let customPrompt = $state(savedOpts.customPrompt);
  let scope = $state<"selected" | "folder" | "visible">("selected");
  /** Lewati halaman berstatus translated. */
  let skipDone = $state(true);

  const basePages = $derived.by<Page[]>(() => {
    if (scope === "visible") return visiblePages;
    if (scope === "folder") return folderId === "all" ? allPages(project) : subtreePages(project, folderId);
    const all = allPages(project);
    const sel = new Set(selectedIds);
    return all.filter((p) => sel.has(p.id));
  });
  const pageIds = $derived(
    (skipDone ? basePages.filter((p) => p.status !== "translated") : basePages).map((p) => p.id),
  );

  /** Daftar model sesi-saja: ganti provider → list ulang, default = simpanan. */
  async function loadModelsFor(pid: string) {
    models = [];
    modelsMsg = "";
    if (!pid) {
      model = "";
      return;
    }
    try {
      models = await api.listModels(pid);
      const saved = providers.find((p) => p.id === pid)?.model ?? "";
      model = saved || (models.find((m) => m.vision) ?? models[0])?.id || "";
      modelsMsg = models.length > 0 ? `${models.length} model.` : "Provider tidak mengembalikan model.";
    } catch (e) {
      modelsMsg = String(e);
    }
  }

  function onProviderChange(v: string) {
    providerId = v;
    write(batchProviderKey, v);
    void loadModelsFor(v);
  }

  // Persist opts tiap berubah (batchOpts merge tetap di OverlayHost).
  $effect(() => {
    const opts: TranslateOpts = {
      targetLang,
      style,
      skipSfx,
      mosaicQuality,
      customPrompt: customPrompt.trim(),
      model,
    };
    write(batchOptsKey, opts);
  });

  onMount(async () => {
    try {
      providers = (await api.getProviders()) ?? [];
    } catch {
      providers = [];
    }
    if (providerId) await loadModelsFor(providerId);
  });

  function submit() {
    if (!providerId) return;
    const opts: TranslateOpts = {
      targetLang,
      style,
      skipSfx,
      mosaicQuality,
      customPrompt: customPrompt.trim(),
      model,
    };
    write(batchProviderKey, providerId);
    write(batchOptsKey, opts);
    onSubmit({ providerId, opts }, pageIds);
  }
</script>

<Panel title={$t("batchDlgTitle")} {onClose} class="max-w-xl">
  <div class="space-y-4">
    <Lov
      label={$t("batchDlgProvider")}
      value={providerId}
      onchange={(v) => onProviderChange(v)}
      hint={providerId ? undefined : $t("batchDlgProviderHint")}
      options={[{ value: "", label: $t("batchDlgProviderHint") }, ...providers.map((p) => ({ value: p.id, label: `${p.name} (${p.model})` }))]}
    />

    {#if providerId}
      <Lov
        label={$t("batchDlgModel")}
        value={model}
        onchange={(v) => (model = v)}
        hint={modelsMsg}
        options={models.map((m) => ({ value: m.id, label: `${m.label}${m.vision ? " · vision" : ""}` }))}
      />
    {/if}

    <div class="grid grid-cols-2 gap-3">
      <Lov label={$t("batchDlgTarget")} value={targetLang} onchange={(v) => (targetLang = v)} options={TARGET_LANGS} />
      <Lov label={$t("batchDlgStyle")} value={style} onchange={(v) => (style = v as TranslateStyle)} options={TRANSLATE_STYLES} />
    </div>

    <details class="group rounded-md border border-line bg-surface-2/60">
      <summary
        class="cursor-pointer list-none px-3 py-2 text-[11px] font-medium text-ink-2 select-none
               transition-colors hover:text-ink"
      >
        {$t("batchDlgAdvanced")}
      </summary>
      <div class="space-y-4 border-t border-line px-3 py-3">
        <div class="grid grid-cols-2 gap-3">
          <Lov
            label={$t("batchDlgMosaic")}
            value={mosaicQuality}
            onchange={(v) => (mosaicQuality = v as MosaicQuality)}
            options={[{ value: "low", label: "low" }, { value: "high", label: "high" }]}
          />
          <label
            class="flex cursor-pointer items-center gap-2.5 self-end rounded-md border border-line
                   bg-surface-2 px-3 text-[13px] text-ink-2 transition-colors hover:border-line-strong"
          >
            <input type="checkbox" class="size-4 shrink-0 accent-accent" bind:checked={skipSfx} />
            {$t("batchDlgSkipSfx")}
          </label>
        </div>

        <div class="flex flex-col gap-1.5">
          <label
            for="batch-custom-prompt"
            class="text-[11px] font-medium tracking-wide text-ink-2 uppercase"
          >
            {$t("batchDlgCustomPrompt")}
          </label>
          <textarea
            id="batch-custom-prompt"
            class="w-full rounded-md border border-line bg-surface-2 px-3 py-2.5 text-[12px]
                   leading-relaxed text-ink placeholder:text-ink-3 outline-none transition-colors
                   focus:border-accent focus:ring-2 focus:ring-accent/25"
            rows="2"
            maxlength="2000"
            bind:value={customPrompt}
          ></textarea>
        </div>
      </div>
    </details>

    <fieldset class="space-y-2">
      <legend class="text-[11px] font-medium tracking-wide text-ink-2 uppercase">{$t("batchDlgScope")}</legend>
      <label class="flex cursor-pointer items-center gap-2 text-[13px] text-ink-2">
        <input type="radio" name="batch-scope" value="selected" class="size-4 accent-accent" bind:group={scope} />
        {$t("batchDlgScopeSelected")} ({selectedIds.length})
      </label>
      <label class="flex cursor-pointer items-center gap-2 text-[13px] text-ink-2">
        <input type="radio" name="batch-scope" value="folder" class="size-4 accent-accent" bind:group={scope} />
        {$t("batchDlgScopeFolder")}
      </label>
      <label class="flex cursor-pointer items-center gap-2 text-[13px] text-ink-2">
        <input type="radio" name="batch-scope" value="visible" class="size-4 accent-accent" bind:group={scope} />
        {$t("batchDlgScopeVisible")} ({visiblePages.length})
      </label>
      <label class="flex cursor-pointer items-center gap-2 text-[13px] text-ink-2">
        <input type="checkbox" class="size-4 shrink-0 accent-accent" bind:checked={skipDone} />
        {$t("batchDlgSkipDone")}
      </label>
    </fieldset>

    <p role="status" class="text-[12px] leading-relaxed text-ink-2">
      {$t("batchDlgEstimate")}: {pageIds.length} × 1
    </p>
    <p class="text-[11px] leading-relaxed text-ink-3">{$t("batchDlgNoBubbleNote")}</p>

    <div class="flex items-center gap-2 border-t border-line pt-4">
      <Button variant="primary" onclick={submit} disabled={!providerId}>{$t("batchDlgStart")}</Button>
      <Button variant="default" onclick={onClose}>{$t("batchDlgCancel")}</Button>
      {#if !providerId}
        <p role="status" class="text-[11px] leading-relaxed text-ink-3">{$t("batchDlgNeedProvider")}</p>
      {/if}
    </div>
  </div>
</Panel>
