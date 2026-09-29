<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";
  import Button from "./Button.svelte";

  interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "class" | "title"> {
    title: string;
    /** Slot aksi di kanan header (biasanya tombol close). Default: tombol ×. */
    onClose?: () => void;
    closeLabel?: string;
    footer?: Snippet;
    children: Snippet;
    class?: string;
  }

  let {
    title,
    onClose,
    closeLabel = "Tutup",
    footer,
    children,
    class: cls = "",
  }: Props = $props();
</script>

<!-- role=dialog + aria-modal supaya screen reader tahu ini modal, bukan
     sekadar overlay. Backdrop gelap + blur biar fokus mata naik ke panel. -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/65 p-4 backdrop-blur-sm">
  <div
    role="dialog"
    aria-modal="true"
    aria-label={title}
    class={[
      "flex max-h-[90vh] w-full max-w-xl flex-col overflow-hidden",
      "rounded-card border border-line bg-surface shadow-2xl shadow-black/50",
      cls,
    ]}
  >
    <header class="flex shrink-0 items-center gap-3 border-b border-line px-5 py-3.5">
      <h2 class="font-display text-sm font-semibold tracking-tight text-ink">{title}</h2>
      {#if onClose}
        <Button
          variant="quiet"
          size="icon"
          class="ml-auto"
          aria-label={closeLabel}
          onclick={onClose}
        ><span aria-hidden="true">×</span></Button>
      {/if}
    </header>

    <div class="min-h-0 flex-1 overflow-auto px-5 py-5 text-sm leading-relaxed text-ink-2">
      {@render children()}
    </div>

    {#if footer}
      <footer class="flex shrink-0 items-center justify-end gap-2 border-t border-line bg-surface-2 px-5 py-3.5">
        {@render footer()}
      </footer>
    {/if}
  </div>
</div>
