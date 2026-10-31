<!-- Dua pane canvas (FE-4, task 3.2): Before + After via SATU objek `shared`
  (12 prop identik) + override minimal per pane (showTranslation/canDraw/
  onEditTranslation/onToolDone). Scroll-div + gate mode ikut pindah agar
  tak ada duplikasi markup; refs stage/scroll naik via bindable. -->
<script lang="ts">
  import CanvasEditor from "./CanvasEditor.svelte";
  import type { BubbleBox, BubbleTranslation, ReadingDirection, Tool } from "../contracts/types";
  import type { PaneMode } from "./EditorToolbar.svelte";

  /** API stage yang dipakai panel (subset export CanvasEditor). */
  export type CanvasApi = Pick<CanvasEditor, "commitEditor" | "redraw" | "deleteSelected">;

  /** 12 prop yang identik di kedua pane (sumber duplikasi 15 prop ganda). */
  export interface PaneShared {
    imageUrl: string;
    fallbackUrl: string;
    imgW: number;
    imgH: number;
    initial: BubbleBox[];
    readingDir: ReadingDirection;
    tool: Tool;
    translations: BubbleTranslation[];
    externalSelected: number | null;
    onSelectForward: (i: number) => void;
    onDrawHint: (m: string) => void;
    onChange: (nb: BubbleBox[]) => void;
  }

  interface Props {
    mode: PaneMode;
    columnMode: boolean;
    shared: PaneShared;
    beforeApi?: CanvasApi | null;
    afterApi?: CanvasApi | null;
    beforeScrollEl?: HTMLDivElement | null;
    afterScrollEl?: HTMLDivElement | null;
    onScroll: (from: HTMLDivElement | null, to: HTMLDivElement | null) => void;
    onToolDone: () => void;
    onEditTranslation: (i: number, value: string) => void;
  }

  let {
    mode,
    columnMode,
    shared,
    beforeApi = $bindable(null),
    afterApi = $bindable(null),
    beforeScrollEl = $bindable(null),
    afterScrollEl = $bindable(null),
    onScroll,
    onToolDone,
    onEditTranslation,
  }: Props = $props();
</script>

{#if mode === "before" || mode === "split"}
  <div
    class="min-h-0 min-w-0 flex-1 overflow-auto p-4"
    bind:this={beforeScrollEl}
    onscroll={() => onScroll(beforeScrollEl, afterScrollEl)}
  >
    <CanvasEditor
      bind:this={beforeApi}
      {...shared}
      showTranslation={false}
      editable={true}
      canDraw={true}
      onToolDone={onToolDone}
    />
  </div>
{/if}
{#if mode === "after" || mode === "split"}
  <div
    class="min-h-0 min-w-0 flex-1 overflow-auto p-4"
    class:border-l={mode === "split" && !columnMode}
    class:border-t={mode === "split" && columnMode}
    class:border-line={mode === "split"}
    bind:this={afterScrollEl}
    onscroll={() => onScroll(afterScrollEl, beforeScrollEl)}
  >
    <CanvasEditor
      bind:this={afterApi}
      {...shared}
      showTranslation={true}
      editable={true}
      canDraw={false}
      onEditTranslation={onEditTranslation}
    />
  </div>
{/if}
