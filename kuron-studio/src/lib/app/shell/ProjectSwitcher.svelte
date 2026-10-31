<!-- Project switcher (FE-3): dropdown + kartu statistik + hitung halaman.
  Self-contained: query pencarian, menu terbuka, dan filter hidup di sini;
  App hanya menerima id terpilih lewat onPick. -->
<script lang="ts">
  import { scale } from "svelte/transition";
  import ProjectCard from "./ProjectCard.svelte";
  import { projectStats } from "../../library/library";
  import { t } from "../../app/i18n";
  import type { Project } from "../../contracts/types";

  interface Props {
    current: Project | null;
    projects: Project[];
    onPick: (id: string) => void;
  }

  let { current, projects, onPick }: Props = $props();
  let showMenu = $state(false);
  let query = $state("");

  const filtered = $derived(
    query.trim()
      ? projects.filter((p) => p.name.toLowerCase().includes(query.trim().toLowerCase()))
      : projects,
  );
  const curStats = $derived(current ? projectStats(current) : null);

  function pick(id: string) {
    showMenu = false;
    query = "";
    onPick(id);
  }
</script>

{#if projects.length > 0}
  {@const cur = current}
  <div
    class="relative"
    onfocusout={(e) => {
      if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node | null)) showMenu = false;
    }}
  >
    <button
      class="flex h-8 max-w-64 items-center gap-2 rounded-md border border-line bg-surface-2 px-2.5 text-sm text-ink outline-none transition-colors hover:border-line-strong focus:border-accent"
      aria-haspopup="listbox"
      aria-expanded={showMenu}
      aria-label={$t("selectProject")}
      onclick={() => (showMenu = !showMenu)}
      onkeydown={(e) => {
        if (e.key === "Escape") showMenu = false;
      }}
    >
      <span class="min-w-0 flex-1 truncate text-left">{cur?.name ?? "—"}</span>
      {#if curStats}
        <span class="tnum shrink-0 text-[11px] text-ink-3">{curStats.done}/{curStats.total}</span>
        <span class="h-1 w-12 shrink-0 overflow-hidden rounded bg-raised" aria-hidden="true">
          <span class="block h-full rounded bg-accent" style="width: {Math.round(curStats.progress * 100)}%"></span>
        </span>
      {/if}
      <span class="shrink-0 text-[10px] text-ink-3" aria-hidden="true">{showMenu ? "▴" : "▾"}</span>
    </button>
    {#if showMenu}
      <div
        class="absolute left-0 top-full z-50 mt-1 w-72 overflow-hidden rounded-md border border-line-strong bg-surface-2 py-1"
        transition:scale={{ duration: 140, start: 0.96 }}
        style="transform-origin: top left;"
        role="listbox"
        aria-label={$t("projectList")}
      >
        <div class="px-1 pb-1">
          <input
            class="h-8 w-full rounded border border-line bg-surface px-2 text-xs text-ink placeholder:text-ink-3 outline-none focus:border-accent"
            placeholder={$t("searchProject")}
            aria-label={$t("searchProject")}
            value={query}
            oninput={(e) => (query = e.currentTarget.value)}
            onkeydown={(e) => {
              if (e.key === "Escape") showMenu = false;
            }}
          />
        </div>
        <div class="max-h-64 overflow-y-auto py-0.5">
          {#each filtered as p (p.id)}
            <ProjectCard project={p} selected={p.id === current?.id} variant="row" onOpen={pick} />
          {:else}
            <p class="px-2.5 py-2 text-xs text-ink-3">{$t("noProjectMatch")}</p>
          {/each}
        </div>
      </div>
    {/if}
  </div>
  <span class="tnum text-xs text-ink-3">{curStats ? curStats.total : 0} {$t("pages")}</span>
{/if}
