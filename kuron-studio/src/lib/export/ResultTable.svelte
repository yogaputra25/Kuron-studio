<script lang="ts">
  import type { Page } from "../contracts/types";
  import Button from "../ui/Button.svelte";
  import StatusBadge from "../ui/StatusBadge.svelte";

  interface Props {
    pages: Page[];
    busy: boolean;
    onRetryPage: (id: string) => void;
    onRetryBubble: (pageId: string, index: number) => void;
  }

  let { pages, busy, onRetryPage, onRetryBubble }: Props = $props();
</script>

<table class="w-full text-xs">
  <thead>
    <tr class="text-left text-ink-3">
      <th class="pb-1.5 pr-2 font-medium">File</th>
      <th class="pb-1.5 pr-2 font-medium">Status</th>
      <th class="pb-1.5 pr-2 text-right font-medium">Bubble</th>
      <th class="pb-1.5 pr-2 font-medium">Translated</th>
      <th class="pb-1.5 text-right font-medium">Aksi</th>
    </tr>
  </thead>
  <tbody>
    {#each pages as pg (pg.id)}
      {@const tr = pg.translation?.bubbles ?? []}
      {@const trDone = tr.filter((b) => b.translated).length}
      {@const failed = pg.status === "failed" || tr.some((b) => !b.translated)}
      <tr class="border-t border-line transition-colors hover:bg-surface-2/50 {failed ? 'bg-danger-soft/25' : ''}">
        <td class="max-w-56 truncate py-2 pr-2 text-[13px] text-ink-2" title={pg.path}>
          {pg.path.split(/[/\\]/).pop()}
        </td>
        <td class="pr-2"><StatusBadge status={pg.status} /></td>
        <td class="tnum pr-2 text-right text-ink-3">{pg.bubbles.length}</td>
        <td class="pr-2">
          {#if tr.length === 0}
            <span class="tnum text-ink-3">—</span>
          {:else}
            <span class="flex items-center gap-2">
              <span class="h-1 w-16 overflow-hidden rounded-full bg-raised">
                <span
                  class="block h-full rounded-full"
                  class:bg-success={trDone === tr.length}
                  class:bg-warn={trDone < tr.length}
                  style={`width: ${(trDone / tr.length) * 100}%`}
                ></span>
              </span>
              <span class="tnum text-ink-3">{trDone}/{tr.length}</span>
            </span>
          {/if}
        </td>
        <td class="py-1.5 text-right">
          <Button variant="default" size="sm" onclick={() => onRetryPage(pg.id)} disabled={busy}>
            Retry page
          </Button>
        </td>
      </tr>
      {#if failed && tr.length > 0}
        {#each tr.filter((b) => !b.translated) as b (b.index)}
          <tr class="border-t border-danger/20 bg-danger-soft/15 text-[11px]">
            <td class="tnum py-1.5 pl-6 pr-2 text-danger">bubble #{b.index + 1} kosong</td>
            <td class="pr-2"></td>
            <td class="pr-2"></td>
            <td class="max-w-56 py-1.5 pr-2 text-ink-3">
          <!-- Asli ditampilkan penuh (2 baris): ini teks yang bikin page
               ditandai gagal, jadi memotongnya menutup alasan retry. -->
          <span class="line-clamp-2" title={b.original}>{b.original || "—"}</span>
        </td>
            <td class="py-1 text-right">
              <Button variant="default" size="sm" onclick={() => onRetryBubble(pg.id, b.index)} disabled={busy}>
                Retry bubble
              </Button>
            </td>
          </tr>
        {/each}
      {/if}
    {/each}
  </tbody>
</table>
