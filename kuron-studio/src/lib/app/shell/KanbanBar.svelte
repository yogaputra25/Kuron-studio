<!-- Kanban + select + view + sort (FE-3): 4 kontrol toolbar digabung.
  Presentasional murni: state milik App/modul, aksi lewat callback.
  Tipe bersama (KanbanFilter/SortKey/PageView) dari library/library.ts. -->
<script lang="ts">
  import { fly } from "svelte/transition";
  import Button from "../../ui/Button.svelte";
  import Icon, { type IconName } from "../../ui/Icon.svelte";
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
  {#each [["all", counts.all, "layers", "text-ink-3"], ["todo", counts.todo, "circle", "text-warn"], ["doing", counts.doing, "clock", "text-info"], ["done", counts.done, "check-circle", "text-success"], ["failed", counts.failed, "x-circle", "text-danger"]] as [key, n, icon, cls] (key)}
    <button
      class="tnum flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs {kanban === key ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
      onclick={() => onKanban(key as KanbanFilter)}
      aria-pressed={kanban === key}
    >
      <span class={cls as string} aria-hidden="true"><Icon name={icon as IconName} /></span>
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
      class="flex h-7 items-center gap-1.5 px-3 text-xs font-medium transition-colors {pageView === 'grid' ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'}"
      role="tab"
      aria-selected={pageView === "grid"}
      title="{$t('gridView')} (G)"
      onclick={() => onSetPageView("grid")}
    ><span aria-hidden="true" class="contents"><Icon name="grid" /></span>{$t("gridView")}</button>
    <button
      class="flex h-7 items-center gap-1.5 border-l border-line px-3 text-xs font-medium transition-colors {pageView === 'list' ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'}"
      role="tab"
      aria-selected={pageView === "list"}
      title="{$t('listView')} (L)"
      onclick={() => onSetPageView("list")}
    ><span aria-hidden="true" class="contents"><Icon name="list" /></span>{$t("listView")}</button>
  </div>
  <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label={$t("sortBy")}>
    {#each [["name", $t("sortName")], ["status", $t("sortStatus")], ["bubbles", $t("sortBubbles")]] as [key, label]}
      <button
        class="flex h-7 items-center gap-1 border-l border-line px-2.5 text-xs font-medium transition-colors first:border-l-0 {sortKey === key ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'}"
        aria-pressed={sortKey === key}
        title={label}
        onclick={() => onSortKey(key as SortKey)}
      >{label}{#if sortKey === key}<span aria-hidden="true" class="contents"><Icon name={sortDir === 1 ? "chevron-up" : "chevron-down"} size="sm" /></span>{/if}</button>
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
