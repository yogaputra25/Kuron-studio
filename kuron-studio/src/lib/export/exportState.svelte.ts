import { pickFolder, saveAs } from "../app/dialogs";
import { api } from "../contracts/api";
import { toBatchInput, toRetryBubbleInput } from "../provider/translateOpts";
import { allPages, subtreePages } from "../library/library";
import { isMobile } from "../app/device";
import type { BatchOpts, ExportFormat, Page, Project } from "../contracts/types";

export type ExportScope = "all" | "folder" | "selected";

export interface ExportStateArgs {
  getProject: () => Project;
  getOpts: () => BatchOpts | null;
  getSelectedIds: () => string[];
  onRefresh: () => void;
  getPageFilter: () => "todo" | "all";
  getFolderFilter: () => string | "all";
}

export interface ExportState {
  readonly busy: boolean;
  readonly error: string;
  readonly info: string;
  readonly exportScope: ExportScope;
  setExportScope(scope: ExportScope): void;
  readonly scopedPages: Page[];
  readonly visiblePages: Page[];
  readonly selectedLive: string[];
  exportAs(format: ExportFormat): Promise<void>;
  shareZip(): Promise<void>;
  retryPage(pageId: string): Promise<void>;
  retryBubble(pageId: string, bubbleIndex: number): Promise<void>;
}

export function createExportState(args: ExportStateArgs): ExportState {
  let busy = $state(false);
  let error = $state("");
  let info = $state("");
  let exportScope = $state<ExportScope>("all");

  const scopedPages = $derived(
    args.getFolderFilter() === "all"
      ? allPages(args.getProject())
      : subtreePages(args.getProject(), args.getFolderFilter() as string),
  );
  const visiblePages = $derived(
    args.getPageFilter() === "todo"
      ? scopedPages.filter((p) => p.status !== "translated")
      : scopedPages,
  );

  /** Id terpilih yang masih ada (buang basi diam-diam sebelum hitung/kirim). */
  const selectedLive = $derived.by(() => {
    const live = new Set(allPages(args.getProject()).map((p) => p.id));
    return args.getSelectedIds().filter((id) => live.has(id));
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
      const project = args.getProject();
      let path: string | null;
      if (format === "png" || format === "psd") {
        path = await pickFolder(format === "png" ? "Folder PNG overlay" : "Folder PSD layers");
      } else {
        const ext = format === "json" ? "json" : "cbz";
        path = await saveAs(`${project.name}.${ext}`, ext.toUpperCase(), [ext]);
      }
      if (!path) return;
      const folderFilter = args.getFolderFilter();
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
    const opts = args.getOpts();
    if (!opts?.providerId) { error = "Pilih provider dulu (buka editor satu halaman)."; return; }
    busy = true; error = "";
    try {
      await api.translateBatch(toBatchInput(opts, { pageIds: [pageId], providerId: opts.providerId, readingDirection: opts.readingDirection }));
      args.onRefresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function shareZip() {
    busy = true; error = ""; info = "";
    try {
      const project = args.getProject();
      const path = await saveAs(`${project.name}.zip`, "ZIP", ["zip"]);
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
    const opts = args.getOpts();
    if (!opts?.providerId) { error = "Pilih provider dulu (buka editor satu halaman)."; return; }
    busy = true; error = "";
    try {
      await api.retryBubble(toRetryBubbleInput(opts, { pageId, bubbleIndex, providerId: opts.providerId, readingDirection: opts.readingDirection }));
      args.onRefresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  return {
    get busy() { return busy; },
    get error() { return error; },
    get info() { return info; },
    get exportScope() { return exportScope; },
    setExportScope(scope: ExportScope) { exportScope = scope; },
    get scopedPages() { return scopedPages; },
    get visiblePages() { return visiblePages; },
    get selectedLive() { return selectedLive; },
    exportAs,
    shareZip,
    retryPage,
    retryBubble,
  };
}
