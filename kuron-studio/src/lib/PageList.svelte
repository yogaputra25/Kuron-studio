<script lang="ts">
  import type { Action } from "svelte/action";
  import type { Page } from "./contracts/types";
  import { fileName, parentFolder } from "./status";
  import StatusBadge from "./ui/StatusBadge.svelte";
  import { t } from "./i18n";

  interface Props {
    pages: Page[];
    thumbs: Record<string, string>;
    broken: Record<string, string>;
    selectedIds: string[];
    selectMode: boolean;
    onCardClick: (e: MouseEvent, pg: Page) => void;
    onOpen: (pg: Page) => void;
    onToggleSelect: (id: string) => void;
    observeCard: Action<HTMLElement, string>;
  }

  let {
    pages,
    thumbs,
    broken,
    selectedIds,
    selectMode,
    onCardClick,
    onOpen,
    onToggleSelect,
    observeCard,
  }: Props = $props();

  let focusIdx = $state(0);
  let rowEls: (HTMLElement | null)[] = $state([]);

  // grid-sort: urutan datang dari App (orderedPages); fokus jepit saat daftar berubah.
  const sorted = $derived(pages);
  function moveFocus(ni: number) {
    if (sorted.length === 0) return;
    const next = Math.min(sorted.length - 1, Math.max(0, ni));
    focusIdx = next;
    rowEls[next]?.focus();
  }

  function onListKey(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || (e.target as HTMLElement)?.isContentEditable) return;
    if (e.key === "ArrowDown" || e.key === "j") {
      e.preventDefault();
      moveFocus(focusIdx + 1);
    } else if (e.key === "ArrowUp" || e.key === "k") {
      e.preventDefault();
      moveFocus(focusIdx - 1);
    } else if (e.key === "Home") {
      e.preventDefault();
      moveFocus(0);
    } else if (e.key === "End") {
      e.preventDefault();
      moveFocus(sorted.length - 1);
    } else if (e.key === " ") {
      const pg = sorted[focusIdx];
      if (!pg) return;
      e.preventDefault();
      onToggleSelect(pg.id);
    }
  }

  $effect(() => {
    // Daftar berubah (filter/ganti project): jepit fokus di rentang valid.
    void sorted.length;
    if (focusIdx >= sorted.length) focusIdx = Math.max(0, sorted.length - 1);
  });

</script>

<div role="listbox" tabindex={-1} aria-label={$t("listView")} aria-orientation="vertical" onkeydown={onListKey} class="px-5 py-3 outline-none">
  <div class="sticky top-0 z-10 flex items-center gap-3 border-b border-line bg-bg px-2 py-1.5 text-[11px] font-semibold text-ink-3">
    <span class="w-6 shrink-0"></span>
    <span class="min-w-0 flex-1 truncate text-left">{$t("sortName")}</span>
    <span class="hidden w-28 shrink-0 text-left sm:block">{$t("sortStatus")}</span>
    <span class="tnum w-16 shrink-0 text-right">{$t("sortBubbles")}</span>
    <span class="tnum hidden w-24 shrink-0 text-right font-mono text-[11px] text-ink-3 md:block">WxH</span>
  </div>
  {#each sorted as pg, i (pg.id)}
    {@const sel = selectedIds.includes(pg.id)}
    {@const folder = parentFolder(pg.path)}
    <div class="relative">
    <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_static_element_interactions -->
    <div
      bind:this={rowEls[i]}
      use:observeCard={pg.id}
      role="option"
      tabindex={i === focusIdx ? 0 : -1}
      aria-selected={sel}
      aria-current={i === focusIdx}
      aria-label={`${fileName(pg.path)}${folder ? `, ${folder}` : ""}`}
      class="flex min-h-11 w-full items-center gap-3 border-b border-line px-2 py-1 text-left
             transition-colors hover:bg-raised focus-visible:outline-2 focus-visible:outline-accent
             {i === focusIdx ? 'border-l-2 border-l-accent bg-raised' : 'border-l-2 border-l-transparent'} {sel ? 'bg-accent/5' : ''} sm:min-h-9"
      onclick={(e) => {
        focusIdx = i;
        onCardClick(e, pg);
      }}
      onkeydown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          focusIdx = i;
          onOpen(pg);
        }
      }}
      onfocus={() => {
        focusIdx = i;
      }}
    >
      <span class="w-6 shrink-0">
        {#if thumbs[pg.id]}
          <img src={thumbs[pg.id]} alt="" class="h-8 w-6 bg-raised object-cover" loading="lazy" />
        {:else if broken[pg.id]}
          <span class="flex h-8 w-6 items-center justify-center bg-danger-soft text-[10px] text-danger" title={broken[pg.id]}>⚠</span>
        {:else}
          <span class="block h-8 w-6 animate-pulse bg-raised"></span>
        {/if}
      </span>
      <span class="min-w-0 flex-1 truncate">
        <span class="font-mono text-xs text-ink">{fileName(pg.path)}</span>
        {#if folder}<span class="ml-1.5 truncate text-[10px] text-ink-3">{folder}</span>{/if}
      </span>
      <span class="hidden w-28 shrink-0 sm:block"><StatusBadge status={pg.status} /></span>
      <span class="tnum w-16 shrink-0 text-right text-[11px] text-cyan">{pg.bubbles.length > 0 ? `⬢${pg.bubbles.length}` : "—"}</span>
      <span class="tnum hidden w-24 shrink-0 text-right text-[11px] text-ink-3 md:block">{pg.width > 0 ? `${pg.width}×${pg.height}` : "—"}</span>
      {#if selectMode}
        <input
          type="checkbox"
          class="h-4 w-4 shrink-0 cursor-pointer accent-accent"
          checked={sel}
          tabindex={-1}
          aria-hidden="true"
          onclick={(e) => e.stopPropagation()}
          onchange={() => onToggleSelect(pg.id)}
        />
      {/if}
    </div>
    </div>
  {/each}
</div>
