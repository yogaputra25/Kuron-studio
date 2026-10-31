<script lang="ts">
  import { t } from "../app/i18n";
  import { isMobile } from "../app/device";
  import Button from "../ui/Button.svelte";
  import type { ExportFormat } from "../contracts/types";
  import type { ExportScope } from "./exportState.svelte";

  interface Props {
    busy: boolean;
    exportScope: ExportScope;
    onScopeChange: (s: ExportScope) => void;
    allCount: number;
    scopedCount: number;
    selectedCount: number;
    folderLabel: string | null;
    onExport: (f: ExportFormat) => void;
    onShare: () => void;
    onRefresh: () => void;
  }

  let {
    busy,
    exportScope,
    onScopeChange,
    allCount,
    scopedCount,
    selectedCount,
    folderLabel,
    onExport,
    onShare,
    onRefresh,
  }: Props = $props();

  // Guard mobile: PNG/PSD butuh folder picker yang tak ada di iOS.
  // String dijaga VERBATIM agar sinkron dengan factory (createExportState):
  // "Export PNG/PSD folder belum didukung di iOS — pakai CBZ atau JSON."
  function handleExport(format: ExportFormat) {
    if (isMobile && (format === "png" || format === "psd")) return;
    onExport(format);
  }
</script>

<!-- Export: blok primer di bawah progress, sebelum tab. -->
<div class="mb-4 rounded-lg border border-line bg-surface-2/60 p-3">
  <div class="mb-2 text-[11px] font-semibold tracking-wide text-ink-3 uppercase">{$t("export")}</div>
  <div class="mb-2 flex flex-wrap items-center gap-1.5" role="radiogroup" aria-label={$t("exportScope")}>
    <button
      class="rounded-full border px-2.5 py-1 text-xs transition-colors {exportScope === 'all' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
      role="radio" aria-checked={exportScope === "all"}
      onclick={() => onScopeChange("all")}
    >{$t("allFolders")} ({allCount})</button>
    <button
      class="rounded-full border px-2.5 py-1 text-xs transition-colors {exportScope === 'folder' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
      role="radio" aria-checked={exportScope === "folder"}
      onclick={() => onScopeChange("folder")}
      title={folderLabel ?? $t("allFolders")}
    >{$t("exportScopeFolder")}{folderLabel ? `: ${folderLabel}` : ""} ({scopedCount})</button>
    <button
      class="rounded-full border px-2.5 py-1 text-xs transition-colors {exportScope === 'selected' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'} disabled:cursor-not-allowed disabled:opacity-40"
      role="radio" aria-checked={exportScope === "selected"}
      disabled={selectedCount === 0}
      title={selectedCount === 0 ? $t("exportScopeEmpty") : undefined}
      onclick={() => onScopeChange("selected")}
    >{$t("exportScopeSelected")} ({selectedCount})</button>
  </div>
  <div class="flex flex-wrap items-center gap-1.5">
    <Button variant="default" size="sm" onclick={() => handleExport("json")} disabled={busy}>{$t("exportJson")}</Button>
    <Button variant="default" size="sm" onclick={() => handleExport("png")} disabled={busy || isMobile} title={isMobile ? "Butuh folder picker (tak ada di iOS)" : undefined}>{$t("exportPng")}</Button>
    <Button variant="default" size="sm" onclick={() => handleExport("cbz")} disabled={busy}>{$t("exportCbz")}</Button>
    <Button variant="default" size="sm" onclick={() => handleExport("psd")} disabled={busy || isMobile} title={isMobile ? "Butuh folder picker (tak ada di iOS)" : undefined}>PSD</Button>
    <Button variant="default" size="sm" onclick={onShare} disabled={busy}>ZIP</Button>
    <Button variant="ghost" size="sm" class="ml-auto" onclick={onRefresh}>{$t("refresh")}</Button>
  </div>
</div>
