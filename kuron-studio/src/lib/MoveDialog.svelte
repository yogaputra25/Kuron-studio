<script lang="ts">
  import { onDestroy } from "svelte";
  import { scale } from "svelte/transition";
  import { api } from "./api";
  import { t } from "./i18n";
  import { childrenOf, folderById, folderPath, singleOwnerFolderId, subtreeStats } from "./library";
  import type { MoveResult } from "./types";
  import type { Project } from "./types";
  import Button from "./ui/Button.svelte";
  import Panel from "./ui/Panel.svelte";

  interface Group {
    key: string;
    label: string;
    ids: string[];
  }

  interface Props {
    project: Project;
    fromFolderId: string | "all";
    pageIds: string[];
    busy: boolean;
    onMoved: (res: MoveResult, destName: string) => void;
    onClose: () => void;
    onError: (msg: string) => void;
  }

  let { project, fromFolderId, pageIds, busy, onMoved, onClose, onError }: Props = $props();

  const from = $derived(fromFolderId === "all" ? null : folderById(project, fromFolderId));
  const ancestors = $derived(from ? folderPath(project, from.id).slice(0, -1).reverse() : []);
  const siblings = $derived(
    !from
      ? []
      : project.folders.filter(
          (f) => f.id !== from.id && (f.parentId ?? null) === (from.parentId ?? null),
        ),
  );
  const kids = $derived(from ? childrenOf(project, from.id) : []);
  const otherRoots = $derived.by(() => {
    const skip = new Set<string>();
    if (from) {
      skip.add(from.id);
      for (const a of folderPath(project, from.id)) skip.add(a.id);
    }
    return project.folders.filter(
      ((f) => (f.parentId == null || f.parentId === "") && !skip.has(f.id)),
    );
  });

  // Folder asal pilihan (semua halaman terpilih ada di dalamnya) — tampil disabled.
  const sourceId = $derived(singleOwnerFolderId(project.folders, pageIds));

  // Satu folder hanya muncul di grup pertama yang memuatnya
  // (mis. sibling yang juga root tidak diulang di "Root lain").
  const groups = $derived.by<Group[]>(() => {
    const seen = new Set<string>();
    const out: Group[] = [];
    const push = (key: string, label: string, ids: string[]) => {
      const fresh = ids.filter((id) => !seen.has(id));
      for (const id of fresh) seen.add(id);
      if (fresh.length > 0) out.push({ key, label, ids: fresh });
    };
    if (ancestors.length > 0) push("up", $t("moveGroupUp"), ancestors.map((f) => f.id));
    if (siblings.length + kids.length > 0 || (from && sourceId === from.id))
      push("near", $t("moveGroupNear"), [
        ...(from && sourceId === from.id ? [from.id] : []),
        ...siblings.map((f) => f.id),
        ...kids.map((f) => f.id),
      ]);
    // from null (semua folder) → otherRoots sudah = semua root,
    // jadi cabang khusus "all" tidak diperlukan lagi.
    if (otherRoots.length > 0) push("roots", $t("moveGroupRoots"), otherRoots.map((f) => f.id));
    if (sourceId && !seen.has(sourceId)) {
      // Asal bersarang yang tak tercover grup di atas → tampil disabled di Sekitar.
      const at = out.findIndex((g) => g.key === "up") >= 0 ? 1 : 0;
      const near: Group = { key: "near", label: $t("moveGroupNear"), ids: [sourceId] };
      out.splice(at, 0, near);
      seen.add(sourceId);
    }
    return out;
  });

  let destId = $state<string | null>(null);
  const destName = $derived(destId ? (folderById(project, destId)?.name ?? "") : "");
  let phase = $state<"pick" | "moving" | "moved">("pick");
  let movedCount = $state(0);
  let shown = $state(0);
  let creep = $state(0);
  let stage = $state(0);
  let tickTimer: ReturnType<typeof setInterval> | null = null;
  const total = $derived(pageIds.length);
  const pct = $derived(
    movedCount === 0 ? creep : Math.round((shown / Math.max(1, movedCount)) * 100),
  );
  const stageLabel = $derived(
    stage === 0
      ? $t("moveStagePack")
      : stage === 1
        ? $t("moveStageSort").replace("{d}", destName)
        : $t("moveStageSave"),
  );

  function stopTick() {
    if (tickTimer !== null) {
      clearInterval(tickTimer);
      tickTimer = null;
    }
  }
  onDestroy(stopTick);

  async function confirm() {
    if (!destId || phase !== "pick" || busy) return;
    phase = "moving";
    movedCount = 0;
    shown = 0;
    creep = 0;
    stage = 0;
    const t0 = Date.now();
    tickTimer = setInterval(() => {
      const el = Date.now() - t0;
      creep = Math.min(90, Math.round((el / 1400) * 100));
      stage = el < 500 ? 0 : el < 1100 ? 1 : 2;
    }, 120);
    try {
      const res = await api.movePages(project.id, pageIds, destId);
      movedCount = res.moved;
      // Count-up nyata 0 → moved dengan easing.
      stopTick();
      const t1 = Date.now();
      await new Promise<void>((resolve) => {
        tickTimer = setInterval(() => {
          const p = Math.min(1, (Date.now() - t1) / 600);
          shown = Math.round(res.moved * (1 - Math.pow(1 - p, 3)));
          if (p >= 1) {
            stopTick();
            resolve();
          }
        }, 30);
      });
      // Total ±1.4 dtk biar transisi kerasa, lalu layar sukses.
      const wait = 1400 - (Date.now() - t0);
      if (wait > 0) await new Promise((r) => setTimeout(r, wait));
      phase = "moved";
      onMoved(res, destName);
    } catch (e) {
      stopTick();
      phase = "pick";
      onError(String(e));
    }
  }
</script>

{#snippet footer()}
  {#if phase === "moved"}
    <Button variant="primary" size="sm" onclick={onClose}>{$t("close")}</Button>
  {:else if phase === "pick"}
    <Button variant="ghost" size="sm" onclick={onClose}>{$t("close")}</Button>
    <Button
      variant="primary"
      size="sm"
      onclick={confirm}
      disabled={!destId || busy}
    >{destId
      ? $t("movePagesTo").replace("{n}", String(pageIds.length)).replace("{d}", destName)
      : $t("moveTo")}</Button
    >
  {/if}
{/snippet}

<Panel title={`${$t("moveTitle")} (${pageIds.length})`} onClose={phase === "moving" ? undefined : onClose} {footer}>
  {#if phase === "pick"}
  {#if groups.length === 0}
    <p class="py-6 text-center text-xs text-ink-3">{$t("noProjectMatch")}</p>
  {:else}
    {#each groups as g (g.key + g.label)}
      <p class="mb-1 mt-3 text-[11px] font-semibold tracking-wide text-ink-3 uppercase first:mt-0">{g.label}</p>
      <div class="overflow-hidden rounded-md border border-line" role="listbox" aria-label={g.label}>
        {#each g.ids as id (id)}
          {@const f = folderById(project, id)}
          {@const st = subtreeStats(project, id)}
          {@const isSource = id === sourceId}
          {#if f}
            <button
              class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs transition-colors {isSource ? 'cursor-not-allowed opacity-50' : 'hover:bg-raised'} {destId === id ? 'bg-accent/10 font-semibold' : ''}"
              role="option"
              aria-selected={destId === id}
              aria-disabled={isSource}
              disabled={isSource}
              onclick={() => (destId = id)}
            >
              <span
                class="flex size-4 shrink-0 items-center justify-center rounded-full text-[10px] {destId === id ? 'bg-accent text-accent-fg' : 'border border-line-strong'}"
                aria-hidden="true">{destId === id ? "✓" : ""}</span>
              <span class="min-w-0 flex-1 truncate">{f.name}</span>
              {#if isSource}
                <span class="shrink-0 rounded bg-raised px-1.5 py-0.5 text-[10px] font-semibold text-ink-3">{$t("moveCurrent")}</span>
              {/if}
              <span class="tnum shrink-0 text-[11px] text-ink-3">{st.done}/{st.total}</span>
            </button>
          {/if}
        {/each}
      </div>
    {/each}
    {#if !destId}
      <p class="mt-3 text-[11px] text-ink-3">{$t("moveHint")}</p>
    {/if}
  {/if}
  {:else if phase === "moving"}
    <div class="flex flex-col items-center px-2 py-6 text-center" role="status">
      <div class="relative mb-5 h-24 w-48" aria-hidden="true">
        <div class="absolute top-2 left-2 h-14 w-10 animate-bounce rounded border border-line bg-surface-2"></div>
        <div class="absolute top-0 left-8 h-14 w-10 animate-bounce rounded border border-line bg-surface-2" style="animation-delay: 150ms"></div>
        <div class="absolute top-2 left-14 h-14 w-10 animate-bounce rounded border border-line bg-surface-2" style="animation-delay: 300ms"></div>
        <div class="absolute right-2 bottom-0 animate-pulse">
          <div class="ml-1 h-2 w-8 rounded-t bg-accent"></div>
          <div class="h-10 w-20 rounded-md rounded-tl-none border border-accent/40 bg-accent/10"></div>
        </div>
      </div>
      <p class="text-sm font-semibold">{$t("movingTo").replace("{n}", String(total)).replace("{d}", destName)}</p>
      <p class="tnum mt-2 text-3xl font-bold">{movedCount === 0 ? creep : shown}<span class="text-base font-normal text-ink-3"> / {movedCount === 0 ? total : movedCount}</span></p>
      <p class="tnum mt-0.5 text-xs font-semibold text-accent">{pct}%</p>
      <div class="mt-3 h-2 w-full overflow-hidden rounded-full bg-raised" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={pct}>
        <div class="h-full rounded-full bg-gradient-to-r from-accent to-accent-hover transition-all duration-150" style="width: {pct}%">
          <div class="h-full w-full animate-pulse bg-white/20"></div>
        </div>
      </div>
      <p class="mt-3 h-4 text-[11px] text-ink-3">{stageLabel}</p>
    </div>
  {:else}
    <div class="flex flex-col items-center px-2 py-8 text-center">
      <span class="flex size-12 items-center justify-center rounded-full bg-accent text-xl text-accent-fg" aria-hidden="true" in:scale={{ start: 0.5, duration: 250 }}>✓</span>
      <p class="mt-4 text-sm font-semibold">{$t("moveDone").replace("{n}", String(movedCount)).replace("{d}", destName)}</p>
    </div>
  {/if}
</Panel>
