<script lang="ts">
  import { t } from "../app/i18n";
  import type { BatchOpts, ExportFormat, Project } from "../contracts/types";
  import { allPages, folderPath, subtreePages } from "../library/library";
  import { computeStats } from "./progressStats";
  import { createExportState } from "./exportState.svelte";
  import Panel from "../ui/Panel.svelte";
  import ProgressSummary from "./ProgressSummary.svelte";
  import ExportActions from "./ExportActions.svelte";
  import ResultTable from "./ResultTable.svelte";

  interface Props {
    project: Project;
    opts: BatchOpts | null;
    selectedIds: string[];
    onRefresh: () => void;
    onClose: () => void;
  }

  let { project, opts, selectedIds, onRefresh, onClose }: Props = $props();

  // M5 note: tab QA/TM dihapus dari panel ini (export-scope) — backend utuh.
  /** Filter status di tab Halaman — default tampilkan semua (hasil jadi langsung terlihat). */
  let pageFilter = $state<"todo" | "all">("all");
  /** Filter folder di tab Halaman — "all" = semua folder (scope default ekspor). */
  let folderFilter = $state<string | "all">("all");

  const stats = $derived(computeStats(project));
  const ex = createExportState({
    getProject: () => project,
    getOpts: () => opts,
    getSelectedIds: () => selectedIds,
    onRefresh: () => onRefresh(),
    getPageFilter: () => pageFilter,
    getFolderFilter: () => folderFilter,
  });

  const folderLabel = $derived(
    folderFilter === "all" ? null : folderPath(project, folderFilter).map((x) => x.name).join(" / "),
  );
</script>

<Panel title={project.name} {onClose} class="max-w-5xl">
  <p class="-mt-1 mb-4 text-xs text-ink-3">
    {$t("resultsExport")} · <span class="tnum">{allPages(project).length}</span> {$t("pages")}
  </p>

  <ProgressSummary {stats} />

  <ExportActions
    busy={ex.busy}
    exportScope={ex.exportScope}
    onScopeChange={(s) => ex.setExportScope(s)}
    allCount={allPages(project).length}
    scopedCount={ex.scopedPages.length}
    selectedCount={ex.selectedLive.length}
    {folderLabel}
    onExport={(f: ExportFormat) => ex.exportAs(f)}
    onShare={() => ex.shareZip()}
    {onRefresh}
  />

  {#if ex.error}
    <p role="alert" class="mb-3 rounded-md border border-warn-soft bg-warn-soft px-2.5 py-1.5 text-xs text-warn">
      {ex.error}
    </p>
  {/if}
  {#if ex.info}
    <p role="status" class="mb-3 rounded-md border border-success-soft bg-success-soft px-2.5 py-1.5 text-xs text-success">
      {ex.info}
    </p>
  {/if}
    {#if project.folders.length > 1}
      <div class="mb-2 flex items-center gap-1.5 overflow-x-auto pb-1" role="group" aria-label="Filter folder">
        <button
          class="shrink-0 rounded-full border px-3 py-1 text-xs transition-colors {folderFilter === 'all' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
          aria-pressed={folderFilter === "all"}
          onclick={() => (folderFilter = "all")}
        >{$t("allFolders")} ({allPages(project).length})</button>
        {#each project.folders as f (f.id)}
          {@const n = subtreePages(project, f.id).length}
          {@const crumbs = folderPath(project, f.id).map((x) => x.name).join(" / ")}
          <button
            class="shrink-0 truncate rounded-full border px-3 py-1 text-xs transition-colors {folderFilter === f.id ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
            aria-pressed={folderFilter === f.id}
            title={crumbs}
            onclick={() => (folderFilter = f.id)}
          >{crumbs} ({n})</button>
        {/each}
      </div>
    {/if}
    <div class="mb-3 flex items-center gap-2">
      <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label="Filter halaman">
        {#each [["todo", "Perlu dikerjakan"], ["all", "Semua"]] as [key, label] (key)}
          <button
            aria-pressed={pageFilter === key}
            class={[
              "h-7 border-l border-line px-3 text-[11px] font-medium transition-colors first:border-l-0",
              pageFilter === key
                ? "bg-accent text-accent-fg"
                : "bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink",
            ]}
            onclick={() => (pageFilter = key as typeof pageFilter)}
          >{label}</button>
        {/each}
      </div>
      <span class="tnum ml-auto text-[11px] text-ink-3">
        {ex.visiblePages.length} {pageFilter === "todo" ? "pending" : "total"}
      </span>
    </div>

    {#if ex.visiblePages.length === 0}
      <div class="rounded-lg border border-success-soft bg-success-soft px-4 py-8 text-center">
        <p class="text-sm font-medium text-success">
          {#if allPages(project).length === 0}Belum ada halaman{:else}Semua halaman selesai{/if}
        </p>
        <p class="mt-1 text-xs text-ink-3">
          {#if allPages(project).length === 0}Import halaman dulu di grid utama.{:else}Coba ubah filter di atas.{/if}
        </p>
      </div>
    {:else}
      <ResultTable
        pages={ex.visiblePages}
        busy={ex.busy}
        onRetryPage={(id) => ex.retryPage(id)}
        onRetryBubble={(pid, i) => ex.retryBubble(pid, i)}
      />
    {/if}
</Panel>
