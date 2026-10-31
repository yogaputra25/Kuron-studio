<!-- Kanban + select + view + sort (FE-3): 4 kontrol toolbar digabung.
  Presentasional murni: state milik App/modul, aksi lewat callback.
  Tipe bersama (KanbanFilter/SortKey/PageView) dari library/library.ts. -->
<script lang="ts">
  import { fly } from "svelte/transition";
  import Button from "../../ui/Button.svelte";
  import { t } from "../../app/i18n";
  import type { KanbanFilter, PageView, SortKey } from "../../library/library";
  import type { Selection } from "../../library/selection.svelte.ts";
  import type { Page } from "../../contracts/types";

  interface Props {
    kanban: KanbanFilter;
    counts: { all: number; todo: number; doing: number; done: number; failed: number };
    sortKey: SortKey;
    sortDir: 1 | -1;
    pageView: PageView;
    visiblePages: Page[];
    selection: Selection;
    busy: boolean;
    onKanban: (k: KanbanFilter) => void;
    onSortKey: (k: SortKey) => void;
    onSetPageView: (v: PageView) => void;
    onMoveRequest: () => void;
  }

  let {
    kanban,
    counts,
    sortKey,
    sortDir,
    pageView,
    visiblePages,
    selection,
    busy,
    onKanban,
    onSortKey,
    onSetPageView,
    onMoveRequest,
  }: Props = $props();
</script>

<section class="flex flex-wrap items-center gap-1.5 px-5 pt-3" aria-label="kanban">
  {#each [["all", counts.all, "bg-ink-3"], ["todo", counts.todo, "bg-warn"], ["doing", counts.doing, "bg-info"], ["done", counts.done, "bg-success"], ["failed", counts.failed, "bg-danger"]] as [key, n, dot]}
    <button
      class="tnum flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs {kanban === key ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
      onclick={() => onKanban(key as KanbanFilter)}
      aria-pressed={kanban === key}
    >
      <span class="h-1.5 w-1.5 shrink-0 rounded-full {dot}" aria-hidden="true"></span>
      {key === "all" ? $t("allFolders") : $t(key === "todo" ? "kanbanTodo" : key === "doing" ? "kanbanDoing" : key === "done" ? "kanbanDone" : "kanbanFailed")} · {n}
    </button>
  {/each}
  {#if !selection.selectMode}
    <Button
      variant="default"
      size="sm"
      onclick={() => selection.setSelectMode(true)}
      disabled={busy || visiblePages.length === 0}
    >{$t("selectMode")}</Button>
  {:else}
    <span class="mx-1 h-4 w-px bg-line" aria-hidden="true"></span>
    <Button
      variant="default"
      size="sm"
      onclick={() => selection.selectAll(visiblePages.map((p) => p.id))}
      disabled={busy || visiblePages.length === 0 || visiblePages.every((p) => selection.selectedIds.includes(p.id))}
    >{$t("selectAll")} ({visiblePages.length})</Button>
    {#if selection.selectedIds.length === 0}
      <Button variant="ghost" size="sm" onclick={() => selection.setSelectMode(false)}>{$t("close")}</Button>
    {/if}
  {/if}
  <div class="ml-auto flex overflow-hidden rounded-md border border-line" role="tablist" aria-label="{$t('gridView')} / {$t('listView')}">
    <button
      class="h-7 px-3 text-xs font-medium transition-colors {pageView === 'grid' ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'}"
      role="tab"
      aria-selected={pageView === "grid"}
      title="{$t('gridView')} (G)"
      onclick={() => onSetPageView("grid")}
    >⬢ {$t("gridView")}</button>
    <button
      class="h-7 border-l border-line px-3 text-xs font-medium transition-colors {pageView === 'list' ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'}"
      role="tab"
      aria-selected={pageView === "list"}
      title="{$t('listView')} (L)"
      onclick={() => onSetPageView("list")}
    >☰ {$t("listView")}</button>
  </div>
  <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label={$t("sortBy")}>
    {#each [["name", $t("sortName")], ["status", $t("sortStatus")], ["bubbles", $t("sortBubbles")]] as [key, label]}
      <button
        class="h-7 px-2.5 text-xs font-medium transition-colors {sortKey === key ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'} {key !== 'name' ? 'border-l border-line' : ''}"
        aria-pressed={sortKey === key}
        title={label}
        onclick={() => onSortKey(key as SortKey)}
      >{label}{sortKey === key ? (sortDir === 1 ? " ▲" : " ▼") : ""}</button>
    {/each}
  </div>
</section>
{#if selection.selectedIds.length > 0}
  <div class="tnum mx-5 mt-3 flex items-center gap-2 rounded-md border border-line bg-surface-2 px-3 py-1.5 text-xs" role="status" transition:fly={{ y: -8, duration: 180 }}>
    <span class="font-semibold">{$t("selectedCount").replace("{n}", String(selection.selectedIds.length))}</span>
    <Button variant="default" size="sm" onclick={onMoveRequest} disabled={busy}>{$t("moveTo")}</Button>
    <Button variant="ghost" size="sm" onclick={() => selection.reset()}>{$t("clearSelection")}</Button>
  </div>
{/if}
