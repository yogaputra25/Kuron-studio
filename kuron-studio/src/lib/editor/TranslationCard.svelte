<!-- Kartu terjemahan bubble (FE-4): header + baris aksi + 1 kartu + geometri.
  Presentasional: semua state mengalir sebagai props, semua aksi naik via
  callback. Draft (cache/timer/IME) milik useDraftText di panel. -->
<script lang="ts">
  import Button from "../ui/Button.svelte";
  import StatusBadge from "../ui/StatusBadge.svelte";
  import GeometryControls from "./GeometryControls.svelte";
  import { t } from "../app/i18n";
  import type { BubbleBox, BubbleTranslation } from "../contracts/types";
  import type { DraftField } from "./useDraftText.svelte";

  export interface CardDraft {
    row: number;
    original: string;
    reading: string;
    translated: string;
  }

  interface Props {
    /** Null = belum ada terjemahan (tampilkan cabang buat-baru). */
    hasTranslation: boolean;
    card: { b: BubbleTranslation; i: number } | null;
    mode: "before" | "split" | "after";
    trgLabel: string;
    draft: CardDraft | null;
    dirtyDraft: boolean;
    glossMsg: string;
    savingTr: boolean;
    showMarkDone: boolean;
    hasBubbles: boolean;
    geomBox: BubbleBox | null;
    geomIdx: number;
    geomOutOfSync: boolean;
    onSelectBubble: (idx: number | null) => void;
    onResetBubble: (i: number) => void;
    onDraftInput: (field: DraftField, v: string) => void;
    onCompStart: () => void;
    onCompEnd: () => void;
    onCommit: () => void;
    onPressStart: (original: string, translated: string) => void;
    onPressEnd: () => void;
    onMarkDone: () => void;
    onSaveAll: () => void;
    onSetGeometry: (i: number, patch: Partial<Pick<BubbleBox, "x" | "y" | "w" | "h">>) => void;
    onScaleBubble: (f: number) => void;
    onSetFontSize: (v: number | null) => void;
    onCreateEmpty: () => void;
  }

  let {
    hasTranslation,
    card,
    mode,
    trgLabel,
    draft,
    dirtyDraft,
    glossMsg,
    savingTr,
    showMarkDone,
    hasBubbles,
    geomBox,
    geomIdx,
    geomOutOfSync,
    onSelectBubble,
    onResetBubble,
    onDraftInput,
    onCompStart,
    onCompEnd,
    onCommit,
    onPressStart,
    onPressEnd,
    onMarkDone,
    onSaveAll,
    onSetGeometry,
    onScaleBubble,
    onSetFontSize,
    onCreateEmpty,
  }: Props = $props();
</script>

{#if hasTranslation}
  <section class="border-t border-line pt-4">
    <div class="mb-3 flex items-center gap-2">
      <h3 class="text-[11px] font-semibold uppercase tracking-wider text-ink-3">Terjemahan</h3>
      <StatusBadge status="translated" class="ml-auto" />
    </div>

    <div class="mb-3 flex gap-2">
      {#if dirtyDraft}<span class="rounded bg-warn-soft px-1.5 py-0.5 text-[10px] text-warn">belum tersimpan</span>{/if}
      {#if showMarkDone}
        <Button variant="ghost" size="sm" onclick={() => onMarkDone()} disabled={savingTr} title={$t("markDone")}>{$t("markDone")}</Button>
      {/if}
      <Button
        variant="default"
        size="sm"
        class="ml-auto"
        onclick={onSaveAll}
        disabled={savingTr}
      >{savingTr ? "…" : dirtyDraft ? `${$t("save")}*` : $t("save")}</Button>
    </div>

    {#if mode !== "before"}
      <p class="mb-2 text-[11px] leading-relaxed text-ink-3">Double-klik bubble di After untuk edit langsung.</p>
    {/if}

    {#if card}
      {@const b = card.b}
      {@const i = card.i}

    <ol class="space-y-2">
      <li
        class="rounded-md border border-accent/40 bg-surface-2 p-2.5"
        onpointerdown={() => onPressStart(b.original, b.translated)}
        onpointerup={onPressEnd}
        onpointerleave={onPressEnd}
        title="Tahan 0.5 dtk untuk simpan ke Glossary"
      >
        <div class="mb-1.5 flex items-center gap-1.5 text-[10px] text-ink-3">
          <button
            class="tnum rounded px-1 font-medium hover:text-ink"
            onclick={() => onSelectBubble(b.index)}
            title="Pilih bubble ini"
          >#{b.index + 1}</button>
          {#if b.isUserEdited}
            <span class="rounded bg-warn-soft px-1.5 py-0.5 text-warn">edited</span>
          {/if}
          {#if b.isUserEdited && b.aiTranslated}
            <button
              class="ml-auto rounded px-1 hover:text-ink"
              onclick={() => onResetBubble(i)}
              title="Kembalikan ke hasil AI terakhir"
            >↺</button>
          {/if}
        </div>
        <p class="mt-0.5 block text-[10px] text-ink-3">Original</p>
        <textarea
          rows="2"
          class="mt-1 w-full rounded-md border border-line bg-surface px-2.5 py-2 text-[13px] text-ink
                 outline-none transition-colors placeholder:text-ink-3
                 focus:border-accent focus:ring-2 focus:ring-accent/25"
          value={draft && draft.row === i ? draft.original : b.original}
          oninput={(e) => onDraftInput("original", (e.target as HTMLTextAreaElement).value)}
          oncompositionstart={onCompStart}
          oncompositionend={onCompEnd}
          onblur={onCommit}
        ></textarea>
        <p class="mt-1.5 block text-[10px] text-ink-3">Reading</p>
        <textarea
          rows="1"
          class="mt-1 w-full rounded-md border border-line bg-surface px-2.5 py-2 text-[13px] text-ink
                 outline-none transition-colors placeholder:text-ink-3
                 focus:border-accent focus:ring-2 focus:ring-accent/25"
          value={draft && draft.row === i ? draft.reading : b.reading}
          oninput={(e) => onDraftInput("reading", (e.target as HTMLTextAreaElement).value)}
          oncompositionstart={onCompStart}
          oncompositionend={onCompEnd}
          onblur={onCommit}
        ></textarea>
        <p class="mt-1.5 block text-[10px] text-ink-3">Terjemahan ({trgLabel})</p>
        <textarea
          rows="2"
          class="mt-1 w-full rounded-md border border-line bg-surface px-2.5 py-2 text-[13px] text-ink
                 outline-none transition-colors placeholder:text-ink-3
                 focus:border-accent focus:ring-2 focus:ring-accent/25"
          value={draft && draft.row === i ? draft.translated : b.translated}
          oninput={(e) => onDraftInput("translated", (e.target as HTMLTextAreaElement).value)}
          oncompositionstart={onCompStart}
          oncompositionend={onCompEnd}
          onblur={onCommit}
        ></textarea>
        {#if glossMsg}<p class="mt-1.5 text-[10px] text-success" role="status">{glossMsg}</p>{/if}
        <GeometryControls
          box={geomBox}
          idx={geomIdx}
          outOfSync={geomOutOfSync}
          fontSize={b.fontSize ?? null}
          onSetGeometry={onSetGeometry}
          onScaleBubble={onScaleBubble}
          onSetFontSize={onSetFontSize}
        />
      </li>
    </ol>
    {:else}
      <p class="rounded-md border border-dashed border-line px-3 py-4 text-center text-xs leading-relaxed text-ink-3">
        Klik bubble di After untuk memilih.
      </p>
    {/if}
  </section>
{:else}
  <section class="border-t border-line pt-4">
    <Button
      variant="default"
      block
      onclick={onCreateEmpty}
      disabled={savingTr || !hasBubbles}
    >{savingTr ? "…" : "Isi manual"}</Button>
    <p class="mt-2 text-[11px] leading-relaxed text-ink-3">Buat translation kosong tanpa AI, lalu ketik per bubble.</p>
  </section>
{/if}
