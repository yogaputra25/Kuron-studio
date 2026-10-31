<script lang="ts">
  import { detectPercent, type DetectState } from "./detectProgress";
  import { t } from "../app/i18n";
  import Button from "../ui/Button.svelte";

  interface Props {
    state: DetectState;
    onClose: () => void;
  }

  let { state, onClose }: Props = $props();

  const pct = $derived(detectPercent(state));
  const failedCount = $derived(state.failed.length);
  const okCount = $derived(Math.max(0, state.done - failedCount));

  // Otomatis tutup bila selesai tanpa gagal; kalau ada gagal, tampilkan
  // ringkasan dan tunggu user menekan Tutup (D1: Tutup = dismiss, bukan cancel).
  $effect(() => {
    if (state.finished && failedCount === 0) {
      const timer = setTimeout(onClose, 2000);
      return () => clearTimeout(timer);
    }
  });
</script>

<div class="pointer-events-none fixed inset-0 z-50 flex items-center justify-center p-4">
  <div
    role="status"
    aria-live="polite"
    class="pointer-events-auto w-80 rounded-card border border-line bg-surface p-5 shadow-lg"
  >
    <h2 class="font-display mb-3 text-sm font-semibold text-ink">{$t("detecting")}</h2>
    {#if !state.finished}
      <p class="tnum text-center text-2xl font-bold text-ink">{state.done} / {state.total}</p>
      <p class="tnum mt-0.5 text-center text-sm text-ink-2">{pct}%</p>
      <div
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={pct}
        aria-label={$t("detecting")}
        class="mt-2 h-1.5 overflow-hidden rounded-full bg-raised"
      >
        <div class="h-full rounded-full bg-accent transition-[width]" style:width={`${pct}%`}></div>
      </div>
      {#if state.file}
        <p class="mt-2 truncate text-center text-[11px] text-ink-3" title={state.file}>{state.file}…</p>
      {/if}
    {:else}
      <p class="tnum text-center text-sm text-ink">
        {$t("detectSummary").replace("{ok}", String(okCount)).replace("{failed}", String(failedCount))}
      </p>
      {#if failedCount > 0}
        <ul class="mt-2 max-h-32 space-y-1 overflow-y-auto text-left">
          {#each state.failed as f (f.pageId)}
            <li class="truncate rounded bg-danger-soft px-2 py-1 text-[11px] text-danger" title={`${f.file}: ${f.message}`}>
              {f.file}: {f.message}
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
    <div class="mt-4 flex justify-center">
      <Button variant="ghost" size="sm" onclick={onClose}>{$t("close")}</Button>
    </div>
  </div>
</div>
