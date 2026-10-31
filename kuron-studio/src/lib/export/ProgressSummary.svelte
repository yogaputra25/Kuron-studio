<script lang="ts">
  import { t } from "../app/i18n";

  interface Stats {
    done: number;
    failed: number;
    pending: number;
    total: number;
    pct: number;
    bubblePct: number;
  }

  interface Props {
    stats: Stats;
  }

  let { stats }: Props = $props();
</script>

<!-- Progress: satu blok visual, bukan tabel angka. Tongkolnya
     diturunkan langsung dari --ks-* supaya ikut light/dark. -->
<div class="mb-4 rounded-lg border border-line bg-surface-2/60 p-4">
  <div class="mb-3 flex flex-wrap items-baseline gap-x-3 gap-y-1">
    <span class="font-display text-2xl font-semibold tabular-nums text-ink">{stats.pct}%</span>
    <span class="text-xs text-ink-3">
      <span class="tnum">{stats.done}</span>/<span class="tnum">{stats.total}</span> {$t("pages")}
      · bubble <span class="tnum">{stats.bubblePct}%</span>
    </span>
    <span class="ml-auto flex gap-2">
      {#if stats.done > 0}
        <span class="tnum rounded bg-success-soft px-2 py-0.5 text-[11px] text-success">
          {stats.done} {$t("done")}
        </span>
      {/if}
      {#if stats.pending > 0}
        <span class="tnum rounded bg-raised px-2 py-0.5 text-[11px] text-ink-2">
          {stats.pending} pending
        </span>
      {/if}
      {#if stats.failed > 0}
        <span class="tnum rounded bg-danger-soft px-2 py-0.5 text-[11px] text-danger">
          {stats.failed} {$t("failed")}
        </span>
      {/if}
    </span>
  </div>

  <div
    class="flex h-2 gap-0.5 overflow-hidden rounded-full bg-raised"
    role="progressbar"
    aria-valuenow={stats.pct}
    aria-valuemin={0}
    aria-valuemax={100}
    aria-label="Progres terjemahan"
  >
    {#if stats.done > 0}
      <div class="bg-success" style={`width: ${(stats.done / Math.max(1, stats.total)) * 100}%`}></div>
    {/if}
    {#if stats.failed > 0}
      <div class="bg-danger" style={`width: ${(stats.failed / Math.max(1, stats.total)) * 100}%`}></div>
    {/if}
  </div>
</div>
