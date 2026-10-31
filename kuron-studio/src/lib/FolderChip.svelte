<script lang="ts">
  import { api } from "./contracts/api";
  import { t } from "./i18n";
  import type { ChapterFolder } from "./contracts/types";
  import { isSiblingNameTaken, type FolderStats } from "./library";

  interface Props {
    projectId: string;
    folders: ChapterFolder[];
    folder: ChapterFolder;
    stats: FolderStats;
    active: boolean;
    dashed?: boolean;
    canDelete: boolean;
    busy: boolean;
    onSelect: (id: string) => void;
    onDelete: (id: string) => void;
    onRenamed: () => void;
    onError: (msg: string) => void;
  }

  let {
    projectId,
    folders,
    folder,
    stats,
    active,
    dashed = false,
    canDelete,
    busy,
    onSelect,
    onDelete,
    onRenamed,
    onError,
  }: Props = $props();

  let editing = $state(false);
  let draft = $state("");
  let saving = $state(false);
  let inputEl: HTMLInputElement | null = $state(null);

  $effect(() => {
    if (editing) {
      inputEl?.focus();
      inputEl?.select();
    }
  });

  function startEdit() {
    draft = folder.name;
    editing = true;
  }

  function cancel() {
    editing = false;
    draft = "";
  }

  async function commit() {
    const name = draft.trim();
    if (!name || name === folder.name) {
      cancel();
      return;
    }
    if (isSiblingNameTaken(folders, folder.id, name)) {
      onError(`folder sudah ada: ${name}`);
      return;
    }
    saving = true;
    try {
      await api.renameFolder(projectId, folder.id, name);
      editing = false;
      draft = "";
      onRenamed();
    } catch (e) {
      onError(String(e));
    } finally {
      saving = false;
    }
  }
</script>

<div
  class="flex items-center overflow-hidden rounded-md border {active
    ? 'border-accent bg-accent/10'
    : dashed
      ? 'border-dashed border-line'
      : 'border-line'}"
>
  {#if editing}
    <input
      class="h-7 w-28 min-w-0 flex-1 bg-surface px-2 py-1 text-xs text-ink outline-none"
      bind:this={inputEl}
      aria-label={$t("renameFolder")}
      bind:value={draft}
      disabled={saving || busy}
      onkeydown={(e) => {
        if (e.key === "Enter") void commit();
        else if (e.key === "Escape") cancel();
        e.stopPropagation();
      }}
      onblur={() => {
        if (editing && !saving) void commit();
      }}
    />
  {:else}
    <button
      class="px-2.5 py-1 text-xs {active ? 'font-semibold' : ''}"
      onclick={() => onSelect(folder.id)}
      ondblclick={startEdit}
      title="{folder.name} · {stats.done}/{stats.total} — {$t('renameFolder')}: klik 2×"
    >
      {folder.name} ({stats.done}/{stats.total}{stats.failed > 0 ? ` ⚠${stats.failed}` : ""})
    </button>
  {/if}
  {#if canDelete && !editing}
    <button
      class="px-1.5 py-1 text-[11px] text-danger hover:bg-danger-soft"
      onclick={() => onDelete(folder.id)}
      title={$t("deleteFolder")}
      aria-label="{$t('deleteFolder')}: {folder.name}">×</button
    >
  {/if}
</div>
