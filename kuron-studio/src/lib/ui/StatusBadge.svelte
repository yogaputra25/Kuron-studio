<script lang="ts">
  import { statusBadgeClass, statusLabel } from "../status";
  import type { PageStatus } from "../types";

  interface Props {
    status: PageStatus;
    /** Angka turunan (jumlah bubble) — dirender tabular, bukan wrap. */
    count?: number;
    class?: string;
  }

  let { status, count, class: cls = "" }: Props = $props();
</script>

<span
  class={[
    "inline-flex shrink-0 items-center gap-1 rounded px-1.5 py-0.5",
    "text-[10px] font-semibold tracking-wide uppercase",
    statusBadgeClass(status),
    cls,
  ]}
  title={`Status: ${statusLabel(status)}`}
>
  {statusLabel(status)}
  {#if count !== undefined}<span class="tnum opacity-80">{count}</span>{/if}
</span>
