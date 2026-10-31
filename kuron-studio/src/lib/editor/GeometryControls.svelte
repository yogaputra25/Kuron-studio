<!-- Kontrol geometri bubble (FE-4): 4 input angka + skala ±10% + font-size + reset.
  Presentasional: box/flag masuk sebagai props, semua aksi naik via callback. -->
<script lang="ts">
  import Button from "../ui/Button.svelte";
  import type { BubbleBox } from "../contracts/types";

  interface Props {
    /** Box aktif; null = tak ada seleksi valid. */
    box: BubbleBox | null;
    idx: number;
    /** True bila jumlah bubbles ≠ translation (baru hapus/tambah tanpa save). */
    outOfSync: boolean;
    /** fontSize manual bubble (null = auto-fit). */
    fontSize: number | null;
    onSetGeometry: (i: number, patch: Partial<Pick<BubbleBox, "x" | "y" | "w" | "h">>) => void;
    onScaleBubble: (f: number) => void;
    onSetFontSize: (v: number | null) => void;
  }

  let {
    box,
    idx,
    outOfSync,
    fontSize,
    onSetGeometry,
    onScaleBubble,
    onSetFontSize,
  }: Props = $props();
</script>

{#if box}
  <p class="mt-2 block text-[10px] text-ink-3">Geometri (px asli)</p>
  <div class="mt-1 grid grid-cols-4 gap-1.5">
    {#each [["x"], ["y"], ["w"], ["h"]] as [k] (k)}
      <label class="flex flex-col gap-0.5 text-[10px] text-ink-3">{k}
        <input
          type="number"
          min="0"
          class="tnum w-full rounded-md border border-line bg-surface px-1.5 py-1 text-[12px] text-ink outline-none focus:border-accent"
          value={box[k as "x" | "y" | "w" | "h"]}
          onchange={(e) => onSetGeometry(idx, { [k]: Number((e.target as HTMLInputElement).value) } as Partial<Pick<BubbleBox, "x" | "y" | "w" | "h">>)}
        />
      </label>
    {/each}
  </div>
  <div class="mt-1.5 flex gap-1.5">
    <Button variant="quiet" size="sm" title="Kecilkan 10% dari pusat" onclick={() => onScaleBubble(0.9)}>−10%</Button>
    <Button variant="quiet" size="sm" title="Besarkan 10% dari pusat" onclick={() => onScaleBubble(1.1)}>+10%</Button>
  </div>
  <label class="ml-2 flex flex-col gap-0.5 text-[10px] text-ink-3">font (px)
    <input
      type="number"
      min="4"
      class="tnum w-16 rounded-md border border-line bg-surface px-1.5 py-1 text-[12px] text-ink outline-none focus:border-accent"
      value={fontSize ?? ""}
      placeholder="auto"
      onchange={(e) => onSetFontSize(Number((e.target as HTMLInputElement).value) || null)}
    />
  </label>
  {#if fontSize}
    <Button variant="quiet" size="sm" title="Kembali ke ukuran otomatis" onclick={() => onSetFontSize(null)}>auto</Button>
  {/if}
{:else if outOfSync}
  <p class="mt-2 text-[10px] leading-relaxed text-warn" role="status">Geometri belum sinkron (bubble baru dihapus/ditambah) — Simpan dulu agar kontrol angka menunjuk bubble yang benar.</p>
{/if}
