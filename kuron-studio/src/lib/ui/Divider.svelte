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
    ? "w-2 shrink-0 cursor-col-resize touch-none border-l border-line bg-surface-2 hover:bg-hover"
    : handle
      ? "flex h-11 shrink-0 cursor-row-resize touch-none items-center justify-center border-t border-line bg-surface-2"
      : "h-2 shrink-0 cursor-row-resize touch-none border-t border-line bg-surface-2 hover:bg-hover"}
  onpointerdown={onDown}
  onpointermove={onMove}
  onpointerup={onUp}
  onpointercancel={onUp}
  onkeydown={onKey}
>{#if orientation === "horizontal" && handle}<span class="h-1 w-12 rounded-full bg-line"></span>{/if}</div>
