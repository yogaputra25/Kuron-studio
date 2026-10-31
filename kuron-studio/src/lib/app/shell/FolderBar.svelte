<!-- Folder bar (FE-3): breadcrumb + satu {#each} FolderChip per daftar.
  Varian chip (active/dashed/canDelete) dihitung helper chipSpecs —
  tiga blok manual yang dulu beda-beda kini satu jalur.
  Presentasional murni: data + aksi lewat props/callback. -->
<script lang="ts">
  import FolderChip from "../../library/FolderChip.svelte";
  import Button from "../../ui/Button.svelte";
  import { allPages, chipSpecs, type FolderChipList, type FolderChipSpec } from "../../library/library";
  import { t } from "../../app/i18n";
  import type { ChapterFolder, Project } from "../../contracts/types";

  interface Props {
    current: Project | null;
    nested: boolean;
    crumb: ChapterFolder[];
    childFolders: ChapterFolder[];
    folderId: string | "all";
    currentFolderName: string;
    folderName: string;
    busy: boolean;
    onSelectFolder: (id: string) => void;
    onDeleteFolder: (id: string) => void;
    onRenamed: () => void;
    onError: (msg: string) => void;
    onFolderName: (v: string) => void;
    onCreateFolder: () => void;
  }

  let {
    current,
    nested,
    crumb,
    childFolders,
    folderId,
    currentFolderName,
    folderName,
    busy,
    onSelectFolder,
    onDeleteFolder,
    onRenamed,
    onError,
    onFolderName,
    onCreateFolder,
  }: Props = $props();

  function specs(kind: FolderChipList, folders: ChapterFolder[]): FolderChipSpec[] {
    if (!current) return [];
    return chipSpecs(current, kind, folders, folderId);
  }
</script>

{#if current}
  {@const specsCrumb = specs("crumb", crumb)}
  {@const specsChildren = specs("children", childFolders)}
  {@const specsFlat = specs("flat", current.folders)}
  <section class="flex flex-wrap items-center gap-1.5 border-b border-line px-5 py-2" aria-label="folders">
    <button
      class="rounded-md border px-2.5 py-1 text-xs {folderId === 'all'
        ? 'border-accent bg-accent/10 font-semibold'
        : 'border-line hover:border-accent/60'}"
      onclick={() => onSelectFolder("all")}>{$t("allFolders")} ({allPages(current).length})</button
    >
    {#if nested}
      {#each specsCrumb as c (c.folder.id)}
        <span class="text-ink-3" aria-hidden="true">›</span>
        <FolderChip
          projectId={current.id}
          folders={current.folders}
          folder={c.folder}
          stats={c.stats}
          active={c.active}
          canDelete={c.canDelete}
          {busy}
          onSelect={onSelectFolder}
          onDelete={onDeleteFolder}
          onRenamed={onRenamed}
          onError={onError}
        />
      {/each}
      {#if childFolders.length > 0}
        <span class="mx-1 h-4 w-px bg-line" aria-hidden="true"></span>
        {#each specsChildren as c (c.folder.id)}
          <FolderChip
            projectId={current.id}
            folders={current.folders}
            folder={c.folder}
            stats={c.stats}
            active={c.active}
            dashed
            canDelete={c.canDelete}
            {busy}
            onSelect={onSelectFolder}
            onDelete={onDeleteFolder}
            onRenamed={onRenamed}
            onError={onError}
          />
        {/each}
      {/if}
    {:else}
      {#each specsFlat as c (c.folder.id)}
        <FolderChip
          projectId={current.id}
          folders={current.folders}
          folder={c.folder}
          stats={c.stats}
          active={c.active}
          canDelete={c.canDelete}
          {busy}
          onSelect={onSelectFolder}
          onDelete={onDeleteFolder}
          onRenamed={onRenamed}
          onError={onError}
        />
      {/each}
    {/if}
    <form
      class="ml-1 flex gap-1"
      onsubmit={(e) => {
        e.preventDefault();
        onCreateFolder();
      }}
    >
      <input
        class="h-7 w-32 rounded-md border border-line bg-surface-2 px-2 text-xs outline-none focus:border-accent"
        placeholder={$t("folderName")}
        aria-label={$t("folderName")}
        value={folderName}
        oninput={(e) => onFolderName(e.currentTarget.value)}
      />
      <Button
        type="submit"
        variant="default"
        size="sm"
        disabled={busy || !folderName.trim()}
        title={folderId === "all" ? $t("newFolder") : `${$t("newFolder")} → ${currentFolderName}`}
        >{$t("newFolder")}</Button
      >
    </form>
  </section>
{/if}
