<!-- Divider geser panel (FE-4, task 3.1): satu varian vertikal (row)
  + dua varian horizontal (kolom slim, tablet handle ≥44px).
  role=separator + tabindex = pola sash (VS Code). -->
<script lang="ts">
  import { t } from "../app/i18n";

  interface Props {
    orientation: "vertical" | "horizontal";
    /** Handle sheet besar khusus sentuh (tablet portrait). */
    handle?: boolean;
    onDown: (e: PointerEvent) => void;
    onMove: (e: PointerEvent) => void;
    onUp: () => void;
    onKey: (e: KeyboardEvent) => void;
  }

  let { orientation, handle = false, onDown, onMove, onUp, onKey }: Props = $props();
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  role="separator"
  aria-orientation={orientation}
  aria-label={$t("sheetHandle")}
  tabindex="0"
  class={orientation === "vertical"
    ? "flex w-2 shrink-0 cursor-col-resize touch-none select-none items-center justify-center border-l border-line bg-surface-2 hover:bg-hover"
    : handle
    ? "flex h-11 shrink-0 cursor-row-resize touch-none select-none items-center justify-center border-t border-line bg-surface-2"
    : "flex h-2 shrink-0 cursor-row-resize touch-none select-none items-center justify-center border-t border-line bg-surface-2 hover:bg-hover"}
  onpointerdown={onDown}
  onpointermove={onMove}
  onpointerup={onUp}
  onpointercancel={onUp}
  onkeydown={onKey}
>{#if orientation === "vertical"}
  <span class="flex flex-col items-center gap-1" aria-hidden="true">
    <i class="size-1 rounded-full bg-ink-3"></i>
    <i class="size-1 rounded-full bg-ink-3"></i>
    <i class="size-1 rounded-full bg-ink-3"></i>
  </span>
{:else}
  <span class="h-1.5 w-16 rounded-full bg-ink-3" aria-hidden="true"></span>
{/if}</div>
