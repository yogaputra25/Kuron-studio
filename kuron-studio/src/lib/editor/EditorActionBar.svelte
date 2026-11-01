<!-- Bar aksi bawah editor: mode pane + Detect + Simpan + collapse.
  Satu alur kiri-ke-kanan, selalu muat satu baris (label ciut di sempit). -->
<script lang="ts">
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";
  import { t } from "../app/i18n";
  import type { PaneMode } from "./EditorToolbar.svelte";

  interface Props {
    mode: PaneMode;
    collapsed: boolean;
    detecting: boolean;
    dirty: boolean;
    saving: boolean;
    bubbleCount: number;
    onSetMode: (m: PaneMode) => void;
    onToggleCollapse: () => void;
    onRunDetect: () => void;
    onSave: () => void;
  }

  let {
    mode,
    collapsed,
    detecting,
    dirty,
    saving,
    bubbleCount,
    onSetMode,
    onToggleCollapse,
    onRunDetect,
    onSave,
  }: Props = $props();
</script>

<div class="flex items-center gap-1.5 border-b border-line bg-surface px-3 py-2 shadow-md sm:gap-2 sm:px-4">
  <div class="flex min-w-0 flex-1 overflow-hidden rounded-md border border-line" role="group" aria-label="Mode pane">
    {#each [["before", "Before"], ["split", "Split"], ["after", "After"]] as [m, label] (m)}
      <button
        class={[
          "h-8 flex-1 border-l border-line px-1 text-xs font-medium transition-colors first:border-l-0 sm:px-3",
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
  >{#snippet leading()}<Icon name="search" />{/snippet}<span class="hidden sm:inline">Detect</span></Button>
  <!-- "Tersimpan" bukan primary: state itu sudah selesai, bukan aksi.
       Warna coral dipakai hanya saat ada perubahan yang belum disimpan. -->
  <Button
    variant={dirty && !saving ? "primary" : "default"}
    size="md"
    class="shrink-0"
    onclick={onSave}
    disabled={saving || !dirty}
    title="Simpan"
  >{#snippet leading()}<Icon name="save" />{/snippet}<span class="hidden sm:inline">{saving ? "Simpan…" : dirty ? "Simpan*" : "Tersimpan"}</span></Button>
</div>
