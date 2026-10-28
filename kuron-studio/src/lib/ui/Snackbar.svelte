<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { t } from "../i18n";
  import Button from "./Button.svelte";

  interface Props {
    message: string;
    /** Auto-tutup (ms). Pesan sukses itu sementara — jangan geser layout. */
    duration?: number;
    onClose: () => void;
  }

  let { message, duration = 5000, onClose }: Props = $props();

  onMount(() => {
    const timer = setTimeout(onClose, duration);
    return () => clearTimeout(timer);
  });
</script>

<div class="pointer-events-none fixed inset-x-0 bottom-6 z-[100] flex justify-center px-4">
  <div
    transition:fly={{ y: 16, duration: 220 }}
    role="status"
    class="pointer-events-auto flex w-max max-w-full items-center gap-2.5 rounded-full border border-line bg-surface py-2 pr-2 pl-3 shadow-2xl shadow-black/30"
  >
    <span
      class="flex size-5 shrink-0 items-center justify-center rounded-full bg-accent text-[11px] text-accent-fg"
      aria-hidden="true">✓</span
    >
    <p class="min-w-0 flex-1 truncate text-xs font-medium text-ink" title={message}>{message}</p>
    <Button variant="quiet" size="icon" aria-label={$t("close")} onclick={onClose}>
      <span aria-hidden="true">×</span>
    </Button>
  </div>
</div>
