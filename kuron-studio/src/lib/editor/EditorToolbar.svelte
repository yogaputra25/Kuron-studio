<!-- Bar atas editor: identitas + (desktop) mode pane, Detect, Simpan.
  Mobile (slim): hanya identitas; aksi pindah ke EditorActionBar di bawahnya. -->
<script lang="ts">
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";
  import StatusBadge from "../ui/StatusBadge.svelte";
  import { t } from "../app/i18n";
  import type { PageStatus } from "../contracts/types";

  export type PaneMode = "before" | "split" | "after";

  interface Props {
    fileName: string;
    filePath: string;
    status: PageStatus;
    detectEngine: string;
    bubbleCount: number;
    mode: PaneMode;
    collapsed: boolean;
    detecting: boolean;
    dirty: boolean;
    saving: boolean;
    onClose: () => void;
    onSetMode: (m: PaneMode) => void;
    onToggleCollapse: () => void;
    onRunDetect: () => void;
    onSave: () => void;
    slim?: boolean;
  }

  let {
    fileName,
    filePath,
    status,
    detectEngine,
    bubbleCount,
    mode,
    collapsed,
    detecting,
    dirty,
    saving,
    onClose,
    onSetMode,
    onToggleCollapse,
    onRunDetect,
    onSave,
    slim = false,
  }: Props = $props();
</script>

<div class="flex items-center gap-x-3 gap-y-2 border-b border-line bg-surface px-4 py-2 sm:px-5">
  <Button variant="ghost" size="md" class="shrink-0" onclick={onClose}>← Grid</Button>

  <div class="flex min-w-0 items-center gap-2">
    <span class="font-display max-w-28 truncate text-sm font-semibold sm:max-w-48" title={filePath}>
      {fileName}
    </span>
    <span class="hidden min-[400px]:inline"><StatusBadge {status} /></span>
    <span class="tnum hidden text-xs text-ink-3 lg:inline">
      engine: {detectEngine} · {bubbleCount} bubble
    </span>
  </div>

  {#if !slim}
    <div class="ml-auto flex items-center gap-1.5 sm:gap-2">
      <div class="flex shrink-0 overflow-hidden rounded-md border border-line" role="group" aria-label="Mode pane">
        {#each [["before", "Before"], ["split", "Split"], ["after", "After"]] as [m, label] (m)}
          <button
            class={[
              "h-8 border-l border-line px-2 text-xs font-medium transition-colors first:border-l-0 sm:px-3",
              mode === m
                ? "bg-accent text-accent-fg"
                : "bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink",
            ]}
            aria-pressed={mode === m}
            onclick={() => onSetMode(m as PaneMode)}
          >{label}</button>
        {/each}
      </div>
      <Button
        variant="ghost"
        size="md"
        class="shrink-0"
        onclick={onToggleCollapse}
        title={collapsed ? $t("expandSidebar") : $t("collapseSidebar")}
        aria-label={collapsed ? $t("expandSidebar") : $t("collapseSidebar")}
        aria-pressed={collapsed}
      >{collapsed ? "»" : "«"}</Button>
      <Button
        variant={bubbleCount === 0 && !detecting ? "primary" : "default"}
        size="md"
        class="shrink-0"
        onclick={onRunDetect}
        disabled={detecting}
        title="Detect"
      >{#snippet leading()}<Icon name="search" />{/snippet}<span>Detect</span></Button>
      <!-- "Tersimpan" bukan primary: state itu sudah selesai, bukan aksi.
           Warna coral dipakai hanya saat ada perubahan yang belum disimpan. -->
      <Button
        variant={dirty && !saving ? "primary" : "default"}
        size="md"
        class="shrink-0"
        onclick={onSave}
        disabled={saving || !dirty}
        title="Simpan"
      >{#snippet leading()}<Icon name="save" />{/snippet}<span>{saving ? "Simpan…" : dirty ? "Simpan*" : "Tersimpan"}</span></Button>
    </div>
  {/if}
</div>
