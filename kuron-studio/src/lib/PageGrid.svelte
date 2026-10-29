<script lang="ts">
  import { fly } from "svelte/transition";
  import type { Action } from "svelte/action";
  import type { Page } from "./types";
  import { fileName, parentFolder } from "./status";
  import StatusBadge from "./ui/StatusBadge.svelte";
  import { t } from "./i18n";

  interface Props {
    pages: Page[];
    thumbs: Record<string, string>;
    broken: Record<string, string>;
    selectedIds: string[];
    selectMode: boolean;
    dragging: boolean;
    onCardClick: (e: MouseEvent, pg: Page) => void;
    onCardPress: (pg: Page) => void;
    onRelease: () => void;
    onToggleSelect: (id: string) => void;
    observeCard: Action<HTMLElement, string>;
  }

  let {
    pages,
    thumbs,
    broken,
    selectedIds,
    selectMode,
    dragging,
    onCardClick,
    onCardPress,
    onRelease,
    onToggleSelect,
    observeCard,
  }: Props = $props();

  // grid-sort: pages datang terurut dari App (orderedPages).
  const ordered = $derived(pages);
</script>

<section
  class="grid grid-cols-2 gap-3 p-5 sm:grid-cols-3 md:grid-cols-3 lg:grid-cols-6 xl:grid-cols-8"
  class:outline-2={dragging}
  class:outline-accent={dragging}
>
  {#each ordered as pg, idx (pg.id)}
    {@const sel = selectedIds.includes(pg.id)}
    {@const folder = parentFolder(pg.path)}
    <div class="relative">
    <button
      use:observeCard={pg.id}
      class="group block w-full overflow-hidden rounded-lg border bg-surface text-left transition-all duration-150
             hover:-translate-y-0.5 hover:border-accent/60 hover:shadow-lg focus-visible:border-accent
             disabled:cursor-not-allowed disabled:opacity-60 {sel ? 'border-accent' : 'border-line'}"
      in:fly={{ y: 14, duration: 220, delay: Math.min(idx * 15, 300) }}
      onclick={(e) => onCardClick(e, pg)}
      onpointerdown={() => onCardPress(pg)}
      onpointerup={onRelease}
      onpointerleave={onRelease}
      onpointercancel={onRelease}
      disabled={!!broken[pg.id]}
    >
      <div class="relative">
        {#if thumbs[pg.id]}
          <img
            src={thumbs[pg.id]}
            alt={fileName(pg.path)}
            class="aspect-[3/4] w-full bg-raised object-cover"
            loading="lazy"
          />
        {:else if broken[pg.id]}
          <div
            class="flex aspect-[3/4] w-full flex-col items-center justify-center gap-1 bg-danger-soft p-2 text-center"
            role="img"
            aria-label="Preview gagal"
          >
            <span class="text-base text-danger" aria-hidden="true">⚠</span>
            <span class="text-[10px] leading-tight text-danger" title={broken[pg.id]}>preview gagal</span>
          </div>
        {:else}
          <div class="aspect-[3/4] w-full animate-pulse bg-raised"></div>
        {/if}
        {#if (pg.bubbles?.length ?? 0) > 0}
          <span
            class="tnum absolute top-1.5 right-1.5 rounded bg-cyan-soft/90 px-1.5 py-0.5
                   text-[10px] font-semibold text-cyan backdrop-blur-sm"
            title="{pg.bubbles.length} bubble"
          >⬢{pg.bubbles.length}</span>
        {/if}
      </div>
      <div class="flex items-center gap-1.5 border-t border-line px-2 py-1.5">
        <span class="truncate text-[11px] text-ink-2 group-hover:text-ink" title={pg.path}>
          {#if folder}{folder}/{/if}{fileName(pg.path)}
        </span>
        <StatusBadge status={pg.status} class="ml-auto" />
      </div>
    </button>
    {#if selectMode}
      <input
        type="checkbox"
        class="absolute top-1.5 left-1.5 h-4 w-4 cursor-pointer accent-accent"
        checked={sel}
        aria-label="{$t('selectPage')}: {fileName(pg.path)}"
        onclick={(e) => e.stopPropagation()}
        onchange={() => onToggleSelect(pg.id)}
      />
    {/if}
    </div>
  {/each}
</section>
