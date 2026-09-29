<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { api } from "./api";
  import { initialBatchState, reduceProgress } from "./batch";
  import { t } from "./i18n";
  import Button from "./ui/Button.svelte";
  import Panel from "./ui/Panel.svelte";
  import type { BatchOpts, Page, TranslateProgress } from "./types";

  interface Props {
    pages: Page[];
    opts: BatchOpts;
    onDone: () => void;
    onClose: () => void;
  }

  let { pages, opts, onDone, onClose }: Props = $props();

  let running = $state(false);
  // Nilai awal dihitung dari prop saat mount, sengaja bukan $derived:
  // `batch` di-mutate (reduceProgress) dan di-reset tiap run(), jadi harus beku.
  // eslint-disable-next-line svelte/no-state-referenced-locally
  // svelte-ignore state_referenced_locally
  let batch = $state(initialBatchState(pages.length));
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

<!-- snippet (bukan variabel) — Panel menerimanya sebagai prop `footer`. -->
{#snippet footer()}
  {#if batch.failed.length > 0}
    <Button variant="default" size="sm" onclick={() => run(batch.failed)} disabled={running}>
      {$t("retryFailed")} ({batch.failed.length})
    </Button>
  {/if}
  <Button
    variant="primary"
    onclick={() => run(targets)}
    disabled={running || targets.length === 0}
  >
    {running ? `${$t("running")} ${pct}%` : `${$t("batch")} ${targets.length} ${$t("pages")}`}
  </Button>
{/snippet}

<Panel title={$t("batchTitle")} {onClose} {footer}>
  <p class="tnum mb-2 text-xs text-ink-3">
    {targets.length} halaman · {batch.done}/{batch.total} {$t("done")} · {$t("failed")}: {batch.failed.length}
  </p>

  <div
    class="mb-3 h-1.5 overflow-hidden rounded-full bg-raised"
    role="progressbar"
    aria-valuenow={pct}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label="Progres batch"
  >
    <div
      class="h-full rounded-full bg-accent transition-[width] duration-200"
      style={`width: ${pct}%`}
    ></div>
  </div>

  {#if error}
    <p role="alert" class="mb-3 rounded-md border border-warn-soft bg-warn-soft px-2.5 py-1.5 text-xs text-warn">
      {error}
    </p>
  {/if}

  {#if batch.failed.length > 0}
    <ul class="mb-1 space-y-1">
      {#each batch.failed as id (id)}
        <li class="rounded-md border border-danger/30 bg-danger-soft px-2.5 py-1.5 text-xs text-danger">
          <span class="font-medium">{pages.find((p) => p.id === id)?.path.split(/[/\\]/).pop() ?? id}</span>
          <span class="text-danger/80"> — {batch.messages[id] ?? ""}</span>
        </li>
      {/each}
    </ul>
  {/if}
</Panel>
