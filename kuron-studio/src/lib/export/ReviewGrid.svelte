<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { api } from "../contracts/api";
  import { t } from "../app/i18n";
  import type { BatchOpts, ExportFormat, Project } from "../contracts/types";
  import { allPages, folderPath, subtreePages } from "../library/library";
  import { isMobile } from "../app/device";
  import Button from "../ui/Button.svelte";
  import Panel from "../ui/Panel.svelte";
  import StatusBadge from "../ui/StatusBadge.svelte";

  interface Props {
    project: Project;
    opts: BatchOpts | null;
    selectedIds: string[];
    onRefresh: () => void;
    onClose: () => void;
  }

  let { project, opts, selectedIds, onRefresh, onClose }: Props = $props();

  let busy = $state(false);
  let error = $state("");
  let info = $state("");
  // M5 note: tab QA/TM dihapus dari panel ini (export-scope) — backend utuh.
  /** Filter status di tab Halaman — default tampilkan semua (hasil jadi langsung terlihat). */
  let pageFilter = $state<"todo" | "all">("all");
  /** Filter folder di tab Halaman — "all" = semua folder (scope default ekspor). */
  let folderFilter = $state<string | "all">("all");

  /** Ringkasan progress: berapa halaman selesai vs belum, per status. */
  const stats = $derived.by(() => {
    const pages = allPages(project);
    const done = pages.filter((p) => p.status === "translated").length;
    const failed = pages.filter((p) => p.status === "failed").length;
    const pending = pages.length - done - failed;
    const totalBubbles = pages.reduce((n, p) => n + p.bubbles.length, 0);
    const translatedBubbles = pages.reduce(
      (n, p) => n + (p.translation?.bubbles.filter((b) => b.translated).length ?? 0),
      0,
    );
    return {
      done,
      failed,
      pending,
      total: pages.length,
      pct: pages.length ? Math.round((done / pages.length) * 100) : 0,
      bubblePct: totalBubbles ? Math.round((translatedBubbles / totalBubbles) * 100) : 0,
    };
  });

  const scopedPages = $derived(
    folderFilter === "all" ? allPages(project) : subtreePages(project, folderFilter),
  );
  const visiblePages = $derived(
    pageFilter === "todo" ? scopedPages.filter((p) => p.status !== "translated") : scopedPages,
  );

  // export-scope D4: satu seleksi (dari dashboard), scope eksplisit saat export.
  type ExportScope = "all" | "folder" | "selected";
  let exportScope = $state<ExportScope>("all");
  /** Id terpilih yang masih ada (buang basi diam-diam sebelum hitung/kirim). */
  const selectedLive = $derived.by(() => {
    const live = new Set(allPages(project).map((p) => p.id));
    return selectedIds.filter((id) => live.has(id));
  });

  async function exportAs(format: ExportFormat) {
    // iOS: folder picker tidak ada; save dialog iOS hanya bisa pindah file
    // yang sudah ada di sandbox. PNG/PSD multi-file tak didukung di mobile.
    if (isMobile && (format === "png" || format === "psd")) {
      error = "Export PNG/PSD folder belum didukung di iOS — pakai CBZ atau JSON.";
      return;
    }
    busy = true; error = ""; info = "";
    try {
      let path: string | null;
      if (format === "png" || format === "psd") {
        path = await open({ directory: true, title: format === "png" ? "Folder PNG overlay" : "Folder PSD layers" });
      } else {
        const ext = format === "json" ? "json" : "cbz";
        path = await save({
          defaultPath: `${project.name}.${ext}`,
          filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
        });
      }
      if (!path) return;
      const scopeFolder = exportScope === "folder" && folderFilter !== "all" ? folderFilter : null;
      const scopeIds = exportScope === "selected" ? selectedLive : undefined;
      const out = await api.exportProject(project.id, format, path, scopeFolder, scopeIds);
      info = `Tersimpan: ${out}`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function retryPage(pageId: string) {
    if (!opts?.providerId) { error = "Pilih provider dulu (buka editor satu halaman)."; return; }
    busy = true; error = "";
    try {
      await api.translateBatch({
        pageIds: [pageId],
        providerId: opts.providerId,
        targetLang: opts.targetLang,
        style: opts.style,
        skipSfx: opts.skipSfx,
        mosaicQuality: opts.mosaicQuality,
        readingDirection: opts.readingDirection,
        ...(opts.customPrompt ? { customPrompt: opts.customPrompt } : {}),
        ...(opts.model ? { model: opts.model } : {}),
      });
      onRefresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }


  async function shareZip() {
    busy = true; error = ""; info = "";
    try {
      const path = await save({
        defaultPath: `${project.name}.zip`,
        filters: [{ name: "ZIP", extensions: ["zip"] }],
      });
      if (!path) return;
      const out = await api.shareProject(project.id, path);
      info = `Tersimpan: ${out}`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function retryBubble(pageId: string, bubbleIndex: number) {
    if (!opts?.providerId) { error = "Pilih provider dulu (buka editor satu halaman)."; return; }
    busy = true; error = "";
    try {
      await api.retryBubble({
        pageId,
        bubbleIndex,
        providerId: opts.providerId,
        targetLang: opts.targetLang,
        style: opts.style,
        skipSfx: opts.skipSfx,
        mosaicQuality: opts.mosaicQuality,
        readingDirection: opts.readingDirection,
        ...(opts.customPrompt ? { customPrompt: opts.customPrompt } : {}),
        ...(opts.model ? { model: opts.model } : {}),
      });
      onRefresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<Panel title={project.name} {onClose} class="max-w-5xl">
  <p class="-mt-1 mb-4 text-xs text-ink-3">
    {$t("resultsExport")} · <span class="tnum">{allPages(project).length}</span> {$t("pages")}
  </p>

  <!-- Progress: satu blok visual, bukan tabel angka. Tongkolnya
       diturunkan langsung dari --ks-* supaya ikut light/dark. -->
  <div class="mb-4 rounded-lg border border-line bg-surface-2/60 p-4">
    <div class="mb-3 flex flex-wrap items-baseline gap-x-3 gap-y-1">
      <span class="font-display text-2xl font-semibold tabular-nums text-ink">{stats.pct}%</span>
      <span class="text-xs text-ink-3">
        <span class="tnum">{stats.done}</span>/<span class="tnum">{stats.total}</span> {$t("pages")}
        · bubble <span class="tnum">{stats.bubblePct}%</span>
      </span>
      <span class="ml-auto flex gap-2">
        {#if stats.done > 0}
          <span class="tnum rounded bg-success-soft px-2 py-0.5 text-[11px] text-success">
            {stats.done} {$t("done")}
          </span>
        {/if}
        {#if stats.pending > 0}
          <span class="tnum rounded bg-raised px-2 py-0.5 text-[11px] text-ink-2">
            {stats.pending} pending
          </span>
        {/if}
        {#if stats.failed > 0}
          <span class="tnum rounded bg-danger-soft px-2 py-0.5 text-[11px] text-danger">
            {stats.failed} {$t("failed")}
          </span>
        {/if}
      </span>
    </div>

    <div
      class="flex h-2 gap-0.5 overflow-hidden rounded-full bg-raised"
      role="progressbar"
      aria-valuenow={stats.pct}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-label="Progres terjemahan"
    >
      {#if stats.done > 0}
        <div class="bg-success" style={`width: ${(stats.done / Math.max(1, stats.total)) * 100}%`}></div>
      {/if}
      {#if stats.failed > 0}
        <div class="bg-danger" style={`width: ${(stats.failed / Math.max(1, stats.total)) * 100}%`}></div>
      {/if}
    </div>
  </div>

  <!-- Export: blok primer di bawah progress, sebelum tab. -->
  <div class="mb-4 rounded-lg border border-line bg-surface-2/60 p-3">
    <div class="mb-2 text-[11px] font-semibold tracking-wide text-ink-3 uppercase">{$t("export")}</div>
    <div class="mb-2 flex flex-wrap items-center gap-1.5" role="radiogroup" aria-label={$t("exportScope")}>
      <button
        class="rounded-full border px-2.5 py-1 text-xs transition-colors {exportScope === 'all' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
        role="radio" aria-checked={exportScope === "all"}
        onclick={() => (exportScope = "all")}
      >{$t("allFolders")} ({allPages(project).length})</button>
      <button
        class="rounded-full border px-2.5 py-1 text-xs transition-colors {exportScope === 'folder' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
        role="radio" aria-checked={exportScope === "folder"}
        onclick={() => (exportScope = "folder")}
        title={folderFilter === "all" ? $t("allFolders") : undefined}
      >{$t("exportScopeFolder")}{folderFilter === "all" ? "" : `: ${folderPath(project, folderFilter).map((x) => x.name).join(" / ")}`} ({scopedPages.length})</button>
      <button
        class="rounded-full border px-2.5 py-1 text-xs transition-colors {exportScope === 'selected' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'} disabled:cursor-not-allowed disabled:opacity-40"
        role="radio" aria-checked={exportScope === "selected"}
        disabled={selectedLive.length === 0}
        title={selectedLive.length === 0 ? $t("exportScopeEmpty") : undefined}
        onclick={() => (exportScope = "selected")}
      >{$t("exportScopeSelected")} ({selectedLive.length})</button>
    </div>
    <div class="flex flex-wrap items-center gap-1.5">
      <Button variant="default" size="sm" onclick={() => exportAs("json")} disabled={busy}>{$t("exportJson")}</Button>
      <Button variant="default" size="sm" onclick={() => exportAs("png")} disabled={busy || isMobile} title={isMobile ? "Butuh folder picker (tak ada di iOS)" : undefined}>{$t("exportPng")}</Button>
      <Button variant="default" size="sm" onclick={() => exportAs("cbz")} disabled={busy}>{$t("exportCbz")}</Button>
      <Button variant="default" size="sm" onclick={() => exportAs("psd")} disabled={busy || isMobile} title={isMobile ? "Butuh folder picker (tak ada di iOS)" : undefined}>PSD</Button>
      <Button variant="default" size="sm" onclick={shareZip} disabled={busy}>ZIP</Button>
      <Button variant="ghost" size="sm" class="ml-auto" onclick={onRefresh}>{$t("refresh")}</Button>
    </div>
  </div>

  {#if error}
    <p role="alert" class="mb-3 rounded-md border border-warn-soft bg-warn-soft px-2.5 py-1.5 text-xs text-warn">
      {error}
    </p>
  {/if}
  {#if info}
    <p role="status" class="mb-3 rounded-md border border-success-soft bg-success-soft px-2.5 py-1.5 text-xs text-success">
      {info}
    </p>
  {/if}
    {#if project.folders.length > 1}
      <div class="mb-2 flex items-center gap-1.5 overflow-x-auto pb-1" role="group" aria-label="Filter folder">
        <button
          class="shrink-0 rounded-full border px-3 py-1 text-xs transition-colors {folderFilter === 'all' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
          aria-pressed={folderFilter === "all"}
          onclick={() => (folderFilter = "all")}
        >{$t("allFolders")} ({allPages(project).length})</button>
        {#each project.folders as f (f.id)}
          {@const n = subtreePages(project, f.id).length}
          {@const crumbs = folderPath(project, f.id).map((x) => x.name).join(" / ")}
          <button
            class="shrink-0 truncate rounded-full border px-3 py-1 text-xs transition-colors {folderFilter === f.id ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
            aria-pressed={folderFilter === f.id}
            title={crumbs}
            onclick={() => (folderFilter = f.id)}
          >{crumbs} ({n})</button>
        {/each}
      </div>
    {/if}
    <div class="mb-3 flex items-center gap-2">
      <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label="Filter halaman">
        {#each [["todo", "Perlu dikerjakan"], ["all", "Semua"]] as [key, label] (key)}
          <button
            aria-pressed={pageFilter === key}
            class={[
              "h-7 border-l border-line px-3 text-[11px] font-medium transition-colors first:border-l-0",
              pageFilter === key
                ? "bg-accent text-accent-fg"
                : "bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink",
            ]}
            onclick={() => (pageFilter = key as typeof pageFilter)}
          >{label}</button>
        {/each}
      </div>
      <span class="tnum ml-auto text-[11px] text-ink-3">
        {visiblePages.length} {pageFilter === "todo" ? "pending" : "total"}
      </span>
    </div>

    {#if visiblePages.length === 0}
      <div class="rounded-lg border border-success-soft bg-success-soft px-4 py-8 text-center">
        <p class="text-sm font-medium text-success">
          {#if allPages(project).length === 0}Belum ada halaman{:else}Semua halaman selesai{/if}
        </p>
        <p class="mt-1 text-xs text-ink-3">
          {#if allPages(project).length === 0}Import halaman dulu di grid utama.{:else}Coba ubah filter di atas.{/if}
        </p>
      </div>
    {:else}
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
        {#each visiblePages as pg (pg.id)}
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
              <Button variant="default" size="sm" onclick={() => retryPage(pg.id)} disabled={busy}>
                Retry page
              </Button>
            </td>
          </tr>
          {#if failed && tr.length > 0}
            {#each tr.filter((b) => !b.translated) as b (b.index)}
              <tr class="border-t border-danger/20 bg-danger-soft/15 text-[11px]">
                <td class="tnum py-1.5 pl-6 pr-2 text-danger">bubble #{b.index + 1} kosong</td>
                <td class="pr-2" colspan="2"></td>
                <td class="max-w-56 py-1.5 pr-2 text-ink-3">
              <!-- Asli ditampilkan penuh (2 baris): ini teks yang bikin page
                   ditandai gagal, jadi memotongnya menutup alasan retry. -->
              <span class="line-clamp-2" title={b.original}>{b.original || "—"}</span>
            </td>
                <td class="py-1 text-right">
                  <Button variant="default" size="sm" onclick={() => retryBubble(pg.id, b.index)} disabled={busy}>
                    Retry bubble
                  </Button>
                </td>
              </tr>
            {/each}
          {/if}
        {/each}
      </tbody>
    </table>
    {/if}
</Panel>
