<!-- Toolbar editor (FE-4): bar atas ramping — kembali, file+status,
  mode pane, collapse. Tool gambar + Detect + Simpan tinggal di
  EditorToolsFab (overlay kanan-bawah kanvas). -->
<script lang="ts">
  import Button from "../ui/Button.svelte";
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
    onClose: () => void;
    onSetMode: (m: PaneMode) => void;
    onToggleCollapse: () => void;
  }

  let {
    fileName,
    filePath,
    status,
    detectEngine,
    bubbleCount,
    mode,
    collapsed,
    onClose,
    onSetMode,
    onToggleCollapse,
  }: Props = $props();
</script>

<div class="flex flex-wrap items-center gap-x-3 gap-y-2 border-b border-line bg-surface px-5 py-2.5">
  <Button variant="ghost" size="sm" onclick={onClose}>← Grid</Button>

  <div class="flex min-w-0 items-center gap-2.5">
    <span class="font-display max-w-48 truncate text-sm font-semibold" title={filePath}>
      {fileName}
    </span>
    <StatusBadge {status} />
    <span class="tnum hidden text-xs text-ink-3 lg:inline">
      engine: {detectEngine} · {bubbleCount} bubble
    </span>
  </div>

  <div class="ml-auto flex max-w-full items-center gap-2 overflow-x-auto">
    <div class="flex shrink-0 overflow-hidden rounded-md border border-line" role="group" aria-label="Mode pane">
      {#each [["before", "Before"], ["split", "Split"], ["after", "After"]] as [m, label] (m)}
        <button
          class={[
            "h-8 border-l border-line px-3 text-xs font-medium transition-colors first:border-l-0",
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
      size="sm"
      class="shrink-0"
      onclick={onToggleCollapse}
      title={collapsed ? $t("expandSidebar") : $t("collapseSidebar")}
      aria-label={collapsed ? $t("expandSidebar") : $t("collapseSidebar")}
      aria-pressed={collapsed}
    >{collapsed ? "»" : "«"}</Button>
  </div>
</div>
