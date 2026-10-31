<!-- Host overlay (FE-3): editor, detect overlay, 5 modal + glossary/settings.
  Presentasional murni: semua kondisi tampil + data lewat props,
  logika buka/tutup dan aksi tetap milik App (diteruskan sebagai callback).
  Catatan jujur: ~28 prop eksplisit — tiap overlay butuh datanya sendiri;
  alternatif (satu objek raksasa) justru menyembunyikan kontrak. -->
<script lang="ts">
  import EditorPanel from "../../editor/EditorPanel.svelte";
  import DetectProgress from "../../detection/DetectProgress.svelte";
  import BatchPanel from "../../provider/BatchPanel.svelte";
  import GlossaryPanel from "../../provider/GlossaryPanel.svelte";
  import MoveDialog from "../../library/MoveDialog.svelte";
  import ReviewGrid from "../../export/ReviewGrid.svelte";
  import ProviderSettings from "../../provider/ProviderSettings.svelte";
  import DiagnosticsPanel from "../../app/DiagnosticsPanel.svelte";
  import type { Page } from "../../contracts/types";
  import type { Project } from "../../contracts/types";
  import type { BatchOpts } from "../../contracts/types";
  import type { ReadingDirection } from "../../contracts/types";
  import type { TranslateOpts } from "../../provider/TranslatePanel.svelte";
  import type { Selection } from "../../library/selection.svelte.ts";
  import type { ThumbnailQueue } from "../../previews/thumbnailQueue.svelte.ts";
  import type { DetectRun } from "../../detection/detectRun.svelte.ts";

  interface Props {
    selection: Selection;
    queue: ThumbnailQueue;
    detectRun: DetectRun;
    selectedPage: Page | null;
    current: Project | null;
    currentId: string | null;
    fullUrls: Record<string, string>;
    openError: string;
    readingDir: ReadingDirection;
    showBatch: boolean;
    batchOpts: BatchOpts | null;
    showGlossary: boolean;
    showMove: boolean;
    folderId: string | "all";
    busy: boolean;
    showReview: boolean;
    showProviders: boolean;
    showDiagnostics: boolean;
    scopedPages: Page[];
    onCloseEditor: () => void;
    onSaved: (updated: Page) => void;
    onBatchOpts: (o: { providerId: string; opts: TranslateOpts }) => void;
    onDetectClose: () => void;
    onBatchDone: () => void;
    onBatchClose: () => void;
    onGlossaryClose: () => void;
    onMoveClose: () => void;
    onMoveError: (msg: string) => void;
    onMoved: (
      res: { moved: number; skipped: number },
      destName: string,
      cid: string,
    ) => void;
    onReviewClose: () => void;
    onRefreshReview: () => void;
    onProvidersClose: () => void;
    onDiagnosticsClose: () => void;
  }

  let {
    selection,
    queue,
    detectRun,
    selectedPage,
    current,
    currentId,
    fullUrls,
    openError,
    readingDir,
    showBatch,
    batchOpts,
    showGlossary,
    showMove,
    folderId,
    busy,
    showReview,
    showProviders,
    showDiagnostics,
    scopedPages,
    onCloseEditor,
    onSaved,
    onBatchOpts,
    onDetectClose,
    onBatchDone,
    onBatchClose,
    onGlossaryClose,
    onMoveClose,
    onMoveError,
    onMoved,
    onReviewClose,
    onRefreshReview,
    onProvidersClose,
    onDiagnosticsClose,
  }: Props = $props();
</script>

{#if selectedPage && currentId}
  <div class="fixed inset-0 z-50 flex flex-col bg-bg">
    {#key selectedPage.id}
      <EditorPanel
        projectId={currentId}
        page={selectedPage}
        fullImageUrl={fullUrls[selectedPage.id] ?? queue.thumbs[selectedPage.id] ?? ""}
        fallbackUrl={queue.thumbs[selectedPage.id] ?? ""}
        {openError}
        {readingDir}
        onClose={onCloseEditor}
        {onSaved}
        {onBatchOpts}
      />
    {/key}
  </div>
{/if}
{#if detectRun.state && !detectRun.hidden}
  <DetectProgress state={detectRun.state} onClose={onDetectClose} />
{/if}

{#if showBatch && current && batchOpts}
  <BatchPanel
    pages={scopedPages}
    opts={batchOpts}
    onDone={onBatchDone}
    onClose={onBatchClose}
  />
{/if}

{#if showGlossary}
  <GlossaryPanel onClose={onGlossaryClose} />
{/if}

{#if showMove && current && currentId}
  {@const cur = current}
  {@const cid = currentId}
  <MoveDialog
    project={cur}
    fromFolderId={folderId}
    pageIds={selection.selectedIds}
    {busy}
    onClose={onMoveClose}
    onError={onMoveError}
    onMoved={(res, destName) => onMoved(res, destName, cid)}
  />
{/if}

{#if showReview && current}
  {@const cur = current}
  <ReviewGrid
    project={cur}
    opts={batchOpts}
    selectedIds={selection.selectedIds}
    onRefresh={onRefreshReview}
    onClose={onReviewClose}
  />
{/if}

{#if showProviders}
  <ProviderSettings onClose={onProvidersClose} />
{/if}

{#if showDiagnostics}
  <DiagnosticsPanel onClose={onDiagnosticsClose} />
{/if}
