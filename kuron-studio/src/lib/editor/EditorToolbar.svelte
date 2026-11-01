<!-- Toolbar editor (FE-4): 12 kontrol bar atas.
  Presentasional: mode/tool/status mengalir sebagai props, semua aksi naik
  via callback. Daftar TOOLS milik toolbar (satu-satunya pemakai). -->
<script lang="ts">
  import Button from "../ui/Button.svelte";
  import StatusBadge from "../ui/StatusBadge.svelte";
  import { t } from "../app/i18n";
  import type { PageStatus, Tool } from "../contracts/types";

  export type PaneMode = "before" | "split" | "after";

  const TOOLS: [Tool, string][] = [
    ["select", "Pilih"],
    ["rect", "Rect"],
    ["ellipse", "Elips"],
    ["freeform", "Bebas"],
    ["tail", "Ekor"],
  ];

  interface Props {
    fileName: string;
    filePath: string;
    status: PageStatus;
    detectEngine: string;
    bubbleCount: number;
    mode: PaneMode;
    tool: Tool;
    readOnlyMode: boolean;
    collapsed: boolean;
    detecting: boolean;
    dirty: boolean;
    saving: boolean;
    onClose: () => void;
    onSetMode: (m: PaneMode) => void;
    onSetTool: (t: Tool) => void;
    onToggleCollapse: () => void;
    onRunDetect: () => void;
    onSave: () => void;
  }

  let {
    fileName,
    filePath,
    status,
    detectEngine,
    bubbleCount,
    mode,
    tool,
    readOnlyMode,
    collapsed,
    detecting,
    dirty,
    saving,
    onClose,
    onSetMode,
    onSetTool,
    onToggleCollapse,
    onRunDetect,
    onSave,
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
    <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label="Mode pane">
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
    <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label="Tool gambar" title={readOnlyMode ? "Gambar hanya di Before" : undefined}>
      {#each TOOLS as [t, label] (t)}
        {@const drawTool = t !== "select"}
        {@const off = drawTool && readOnlyMode}
        <button
          class={[
            "h-8 border-l border-line px-3 text-xs font-medium transition-colors first:border-l-0",
            tool === t
              ? "bg-accent text-accent-fg"
              : "bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink",
            off ? "cursor-not-allowed opacity-40" : "",
          ]}
          aria-pressed={tool === t}
          disabled={off}
          title={off ? "Gambar hanya di Before" : label}
          onclick={() => onSetTool(t as Tool)}
        >{label}</button>
      {/each}
    </div>
    <Button
      variant="ghost"
      size="sm"
      onclick={onToggleCollapse}
      title={collapsed ? $t("expandSidebar") : $t("collapseSidebar")}
      aria-label={collapsed ? $t("expandSidebar") : $t("collapseSidebar")}
      aria-pressed={collapsed}
    >{collapsed ? "»" : "«"}</Button>
    <Button
      variant={bubbleCount === 0 && !detecting ? "primary" : "default"}
      size="sm"
      onclick={onRunDetect}
      disabled={detecting}
    >{detecting ? "Detect…" : "Detect"}</Button>
    <!-- "Tersimpan" bukan primary: state itu sudah selesai, bukan aksi.
         Warna coral dipakai hanya saat ada perubahan yang belum disimpan. -->
    <Button
      variant={dirty && !saving ? "primary" : "default"}
      size="sm"
      onclick={onSave}
      disabled={saving || !dirty}
    >
      {saving ? "Simpan…" : dirty ? "Simpan*" : "Tersimpan"}
    </Button>
  </div>
</div>
