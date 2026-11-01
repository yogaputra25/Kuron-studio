<!-- FAB + palet tool editor (overlay kanan-bawah kanvas).
  Presentasional: buka/tutup + semua aksi lewat props; daftar TOOLS
  milik komponen ini. Tombol draw yang mati di After menjelaskan alasan
  (title) — kontrak draw-tools-feedback. -->
<script lang="ts">
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";
  import type { Tool } from "../contracts/types";

  const TOOLS: [Tool, string][] = [
    ["select", "Pilih"],
    ["rect", "Rect"],
    ["ellipse", "Elips"],
    ["freeform", "Bebas"],
    ["tail", "Ekor"],
  ];

  interface Props {
    open: boolean;
    tool: Tool;
    readOnlyMode: boolean;
    detecting: boolean;
    dirty: boolean;
    saving: boolean;
    bubbleCount: number;
    onToggle: () => void;
    onPickTool: (t: Tool) => void;
    onRunDetect: () => void;
    onSave: () => void;
  }

  let {
    open,
    tool,
    readOnlyMode,
    detecting,
    dirty,
    saving,
    bubbleCount,
    onToggle,
    onPickTool,
    onRunDetect,
    onSave,
  }: Props = $props();
</script>

<div class="pointer-events-none absolute inset-x-0 bottom-0 z-20 flex flex-col items-end gap-2 p-4">
  {#if open}
    <div
      class="pointer-events-auto flex w-56 flex-col gap-1 rounded-xl border border-line bg-surface p-2 shadow-2xl"
      role="menu"
      aria-label="Tool editor"
    >
      {#each TOOLS as [t, label] (t)}
        {@const drawTool = t !== "select"}
        {@const off = drawTool && readOnlyMode}
        <Button
          variant={tool === t ? "default" : "ghost"}
          class="w-full justify-start"
          disabled={off}
          title={off ? "Gambar hanya di Before" : label}
          onclick={() => onPickTool(t)}
        >{label}</Button>
      {/each}
      <div class="my-1 h-px bg-line" aria-hidden="true"></div>
      <Button
        variant={bubbleCount === 0 && !detecting ? "primary" : "default"}
        class="w-full"
        onclick={onRunDetect}
        disabled={detecting}
      >{detecting ? "Detect…" : "Detect"}</Button>
      <Button
        variant={dirty && !saving ? "primary" : "default"}
        class="w-full"
        onclick={onSave}
        disabled={saving || !dirty}
      >{saving ? "Simpan…" : dirty ? "Simpan*" : "Tersimpan"}</Button>
    </div>
  {/if}
  <button
    class="pointer-events-auto flex size-[52px] items-center justify-center rounded-full bg-accent text-accent-fg shadow-xl transition-transform hover:scale-105 active:scale-95"
    aria-expanded={open}
    aria-label={open ? "Tutup tool" : "Buka tool"}
    onclick={onToggle}
  >
    {#if open}
      <span aria-hidden="true" class="text-xl leading-none">×</span>
    {:else}
      <Icon name="pen" />
    {/if}
  </button>
</div>
