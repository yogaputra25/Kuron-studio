<!-- Kartu project (FE-3): satu markup untuk switcher dropdown (row)
  dan dashboard (card). Beda disengaja = varian prop eksplisit, bukan {#if}
  tersembunyi. Presentasional murni: statistik dihitung di sini dari project,
  aksi lewat callback. -->
<script lang="ts">
  import { projectStats } from "../../library/library";
  import { t } from "../../app/i18n";
  import type { Project } from "../../contracts/types";

  interface Props {
    project: Project;
    selected: boolean;
    variant: "row" | "card";
    onOpen: (id: string) => void;
    /** Bila diisi, tampil tombol Hasil & Ekspor (varian card saja). */
    onResults?: (id: string) => void;
  }

  let { project, selected, variant, onOpen, onResults }: Props = $props();
  const st = $derived(projectStats(project));
  const pct = $derived(Math.round(st.progress * 100));
</script>

{#if variant === "row"}
  <button
    class="flex w-full items-center gap-2 px-2.5 py-1 text-left text-xs transition-colors hover:bg-raised {selected
      ? 'bg-accent/10'
      : ''}"
    role="option"
    aria-selected={selected}
    onclick={() => onOpen(project.id)}
    title="{project.name} · {st.done}/{st.total}"
  >
    <span class="min-w-0 flex-1 truncate {selected ? 'font-semibold' : ''}">{project.name}</span>
    <span class="tnum shrink-0 text-[11px] {st.failed > 0 ? 'text-danger' : 'text-ink-3'}">{st.done}/{st.total}</span>
  </button>
{:else}
  <div class="relative w-36 shrink-0 self-center">
    <button
      class="block w-full overflow-hidden rounded-md border px-2.5 pt-1.5 pb-2.5 text-left transition-colors {selected
        ? 'border-accent bg-accent/10'
        : 'border-line bg-surface hover:border-accent/60'}"
      onclick={() => onOpen(project.id)}
      title="{project.name} · {st.done}/{st.total} · {st.failed} {$t('kanbanFailed')}"
    >
      <div class="truncate pr-5 text-xs font-semibold">{project.name}</div>
      <div class="tnum mt-0.5 text-[10px] {st.failed > 0 ? 'text-danger' : 'text-ink-3'}">
        {st.done}/{st.total} · {pct}%{st.failed > 0 ? ` · ${st.failed}` : ""}
      </div>
      <div class="absolute inset-x-0 bottom-0 h-0.5 bg-raised">
        <div class="h-full bg-accent" style="width: {pct}%"></div>
      </div>
    </button>
    {#if onResults}
      <button
        class="absolute top-1 right-1 rounded bg-surface-2/90 px-1 py-0.5 text-[11px] leading-none text-ink-2 opacity-70 backdrop-blur-sm transition-opacity hover:bg-hover hover:text-ink hover:opacity-100 focus-visible:opacity-100"
        title={$t("resultsExport")}
        aria-label="{$t('resultsExport')}: {project.name}"
        onclick={(e) => {
          e.stopPropagation();
          onResults(project.id);
        }}
      >⧉</button>
    {/if}
  </div>
{/if}
