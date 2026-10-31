<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { tick } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";

  import iconUrl from "../icon.svg";
  import { isMobile } from "./lib/app/device";
  import { api } from "./lib/contracts/api";
  import UpdateButton from "./lib/app/UpdateButton.svelte";
  import AlertBanners from "./lib/app/shell/AlertBanners.svelte";
  import BatchPanel from "./lib/provider/BatchPanel.svelte";
  import EditorPanel from "./lib/editor/EditorPanel.svelte";
  import GlossaryPanel from "./lib/provider/GlossaryPanel.svelte";
  import ProviderSettings from "./lib/provider/ProviderSettings.svelte";
  import ReviewGrid from "./lib/export/ReviewGrid.svelte";
  import PageGrid from "./lib/library/PageGrid.svelte";
  import PageList from "./lib/library/PageList.svelte";
  import { lang, t, theme } from "./lib/app/i18n";
  import type { Lang, Theme } from "./lib/app/i18n";
  import type { BatchOpts, Page, Project, ReadingDirection } from "./lib/contracts/types";
  import { ARCHIVE_EXTS, IMAGE_EXTS, PDF_EXTS } from "./lib/contracts/types";
  import { allPages, childrenOf, folderPath, groupKanban, hasNesting, projectStats, subtreePages } from "./lib/library/library";
  import type { KanbanFilter, PageView, SortKey } from "./lib/library/library";
  import Button from "./lib/ui/Button.svelte";
  import { fileName, sortPagesByBubbles, sortPagesByName, sortPagesByStatus } from "./lib/library/status";

  import MoveDialog from "./lib/library/MoveDialog.svelte";
  import Onboarding from "./lib/app/Onboarding.svelte";
  import FolderBar from "./lib/app/shell/FolderBar.svelte";
  import KanbanBar from "./lib/app/shell/KanbanBar.svelte";
  import ProjectCard from "./lib/app/shell/ProjectCard.svelte";
  import ProjectSwitcher from "./lib/app/shell/ProjectSwitcher.svelte";
  import DetectProgress from "./lib/detection/DetectProgress.svelte";
  import { createDetectRun } from "./lib/detection/detectRun.svelte.ts";
  import DiagnosticsPanel from "./lib/app/DiagnosticsPanel.svelte";
  import { onInvoke } from "./lib/contracts/log";
  import { createProjectStore } from "./lib/library/projectStore.svelte.ts";

  import { createSelection, orderIds } from "./lib/library/selection.svelte.ts";
  import { createThumbnailQueue } from "./lib/previews/thumbnailQueue.svelte.ts";

  const projectStore = createProjectStore();
  const projects = $derived(projectStore.projects);
  const current = $derived(projectStore.current);
  let name = $state("");
  /** false sampai daftar project pertama selesai dimuat (cegah flash onboarding). */
  let folderId = $state<string | "all">("all");
  let kanban = $state<KanbanFilter>("all");
  let folderName = $state("");
  /** Seleksi multi-halaman: state milik modul, App hanya merangkai. */
  const selection = createSelection();
  let showMove = $state(false);
  // detect-ux-pack 3.1: satu tombol Import + menu pola switcher.
  let showImportMenu = $state(false);
  let fullUrls = $state<Record<string, string>>({});
  /** Antrean thumbnail: state milik modul, App hanya merangkai. */
  const queue = createThumbnailQueue();
  let selectedId = $state<string | null>(null);
  let readingDir = $state<ReadingDirection>("rtl");
  let error = $state("");
  let info = $state("");
  // detect-ux-pack D1: overlay progres non-blocking + guard generasi.
  const detectRun = createDetectRun();
  let pendingDetectCancel: (() => void) | null = null;
  // page-list-view D4/D5: mode Grid/List + pref + scroll per mode.
  function loadPageView(): "grid" | "list" {
    try {
      return localStorage.getItem("kuron-page-view") === "list" ? "list" : "grid";
    } catch {
      return "grid";
    }
  }
  let pageView = $state<PageView>(loadPageView());
  let scrollByView: Record<PageView, number> = { grid: 0, list: 0 };

  /** Ganti mode: simpan scroll lama, pulihkan scroll mode tujuan. */
  async function setPageView(v: "grid" | "list") {
    if (v === pageView) return;
    scrollByView[pageView] = window.scrollY;
    pageView = v;
    try {
      localStorage.setItem("kuron-page-view", v);
    } catch {
      /* private mode: sesi ini saja */
    }
    await tick();
    window.scrollTo(0, scrollByView[v] ?? 0);
  }

  /** G/L global khusus grid (abaikan saat editor/menu/input fokus). */
  function onViewKey(e: KeyboardEvent) {
    if (selectedPage || e.ctrlKey || e.metaKey || e.altKey) return;
    const t = e.target as HTMLElement | null;
    const tag = t?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || !!t?.isContentEditable) return;
    if (e.key === "g" || e.key === "G") void setPageView("grid");
    else if (e.key === "l" || e.key === "L") void setPageView("list");
  }
  let openError = $state("");
  // grid-sort D1: satu state sortir untuk Grid + List.
  let sortKey = $state<SortKey>("name");
  let sortDir = $state<1 | -1>(1);

  /** Urutkan sekali di App; kedua view terima jadi (tanpa fetch ulang). */
  const orderedPages = $derived.by(() => {
    if (sortKey === "status") return sortPagesByStatus(visiblePages, sortDir);
    if (sortKey === "bubbles") return sortPagesByBubbles(visiblePages, sortDir);
    const list = sortPagesByName(visiblePages);
    return sortDir === 1 ? list : [...list].reverse();
  });

  function setSortKey(key: SortKey) {
    if (sortKey === key) sortDir = sortDir === 1 ? -1 : 1;
    else {
      sortKey = key;
      sortDir = 1;
    }
  }
  let dragging = $state(false);
  let busy = $state(false);
  let showProviders = $state(false);
  let showBatch = $state(false);
  let showGlossary = $state(false);
  let showReview = $state(false);
  let showDiagnostics = $state(false);
  /** Error invoke terakhir — banner merah di header, selalu terlihat. */
  let lastInvokeError = $state<{ command: string; error: string } | null>(null);
  // Opts terakhir dari editor (provider+translate opts) — dipakai Batch/Review.
  let batchOpts = $state<BatchOpts | null>(null);

  async function refreshProject(id: string) {
    await projectStore.refreshProject(id);
  }

  const currentFolder = $derived(
    current && folderId !== "all" ? (current.folders.find((f) => f.id === folderId) ?? null) : null,
  );
  const nested = $derived(current ? hasNesting(current) : false);
  const crumb = $derived(current && currentFolder ? folderPath(current, folderId as string) : []);
  const childFolders = $derived(
    !current ? [] : folderId === "all"
      ? current.folders.filter((f) => f.parentId == null || f.parentId === "")
      : childrenOf(current, folderId as string),
  );
  const scopedPages = $derived(
    !current ? [] : currentFolder ? subtreePages(current, folderId as string) : allPages(current),
  );
  const kanbanGroups = $derived(groupKanban(scopedPages));
  const visiblePages = $derived(
    kanban === "all" ? [...kanbanGroups.todo, ...kanbanGroups.doing, ...kanbanGroups.done, ...kanbanGroups.failed] : kanbanGroups[kanban],
  );
  const selectedPage = $derived(allPages(current ?? { id: "", name: "", folders: [] } as Project).find((p) => p.id === selectedId) ?? null);
  /** Halaman yang preview-nya gagal → jumlah tombol "Bersihkan". */
  const brokenCount = $derived(Object.keys(queue.broken).length);

  async function refresh() {
    try {
      await projectStore.refresh();
    } catch (e) {
      // Hanya `list_projects` yang gagal di sini. Label "Backend
      // unreachable" hanya sah kalau invoke-nya sendiri yang tidak jalan.
      error = `Backend unreachable (jalankan via 'pnpm tauri dev'): ${e}`;
    }
    if (projectStore.currentId) await loadThumbs(projectStore.currentId);
  }

  // M4-6: apply persisted theme before first paint of the shell.
  $effect(() => {
    document.documentElement.dataset.theme = $theme;
  });
  const cycleLang = () => {
    lang.update((v: Lang) => (v === "id" ? "en" : v === "en" ? "zh" : "id"));
  };

  // Antrean thumbnail milik modul previews/thumbnailQueue (FE-2).
  function loadThumbs(projectId: string) {
    const proj = projects.find((p) => p.id === projectId);
    if (!proj) return;
    queue.loadThumbs(allPages(proj));
  }

  /** Coba lagi thumbnail yang gagal di project aktif. */
  function retryThumbs() {
    if (!current) return;
    queue.retryThumbs(allPages(current));
  }

  async function cleanPages() {
    const currentId = projectStore.currentId;
    if (!currentId) return;
    busy = true;
    error = "";
    try {
      const res = await api.cleanPages(currentId, folderId === "all" ? null : folderId);
      const updated = await api.getProject(currentId);
      projectStore.replaceProject(currentId, updated);
      // Halaman hilang dari store — buang juga thumbnail-nya.
      const live = new Set(allPages(updated).map((pg) => pg.id));
      queue.pruneThumbs(live);
      for (const id of Object.keys(fullUrls)) if (!live.has(id)) delete fullUrls[id];
      selectedId = live.has(selectedId ?? "") ? selectedId : null;
      fullUrls = { ...fullUrls };
      info =
        res.removed === 0
          ? $t("nothingToClean")
          : $t("cleanDone")
              .replace("{removed}", String(res.removed))
              .replace("{kept}", String(res.kept));
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function deleteProject() {
    const currentId = projectStore.currentId;
    if (!currentId || !current) return;
    if (!confirm($t("confirmDeleteProject"))) return;
    busy = true;
    error = "";
    const gone = currentId;
    try {
      await api.deleteProject(gone);
      queue.resetThumbs();
      fullUrls = {};
      selectedId = null;
      closeEditor();
      projectStore.removeProject(gone);
      if (projectStore.currentId) await loadThumbs(projectStore.currentId);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function createProject(explicit?: string) {
    const n = (explicit ?? name).trim();
    if (busy || !n) return;
    busy = true;
    error = "";
    try {
      const p = await api.createProject(n);
      name = "";
      projectStore.addProject(p);
      folderId = "all";
      kanban = "all";
      selectedId = null;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  /** Onboarding V4: project contoh siap-oprek (3 halaman + bubble seed). */
  async function createSample() {
    if (busy) return;
    busy = true;
    error = "";
    try {
      const p = await api.createSample($t("onboardSample"));
      projectStore.addProject(p);
      openProject(p.id);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function doImport(paths: string[]) {
    const currentId = projectStore.currentId;
    if (!currentId || paths.length === 0) return;
    busy = true;
    try {
      const target = folderId === "all" ? null : folderId;
      const res = await api.importPages(currentId, paths, target);
      const updated = await api.getProject(currentId);
      projectStore.replaceProject(currentId, updated);
      await loadThumbs(currentId);
      if (res.skipped > 0) error = `${res.skipped} file non-image dilewati.`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  // fix-preview-hang §2: mount editor LANGSUNG (thumb instan via
  // fullUrls ?? thumbs), fetch full 1600 paralel tanpa await-before-mount.
  // {#key selectedPage.id} stabil (id sama) → prop update tidak remount.
  // (ponytail: ganti base64 1600 dengan `asset:`/temp-file bila tetap berat.)
  function openEditor(pg: Page) {
    const id = pg.id;
    selectedId = id;
    openError = "";
    if (fullUrls[id]) return;
    void api
      .getImagePreview(pg.path, 1600)
      .then((url) => {
        if (selectedId !== id) return; // user sudah pindah halaman
        fullUrls[id] = url;
      })
      .catch((e) => {
        if (selectedId !== id) return;
        error = String(e);
        openError = String(e); // thumb tetap tampil — editor usable (lihat §2.2)
      });
  }

  function closeEditor() {
    selectedId = null;
    openError = "";
  }

  /** Seleksi ikut konteks: ganti project/folder = mulai dari kosong (D3). */
  $effect(() => {
    const _scope = projectStore.currentId + "|" + folderId;
    void _scope;
    selection.reset();
  });

  function onCardClick(e: MouseEvent, pg: Page) {
    if (!selection.click(e, pg, orderIds(visiblePages))) openEditor(pg);
  }

  function onSaved(updated: Page) {
    const currentId = projectStore.currentId;
    const cur = projectStore.current;
    if (!currentId || !cur) return;
    projectStore.replaceProject(currentId, {
      ...cur,
      folders: cur.folders.map((f) => ({ ...f, pages: f.pages.map((pg) => (pg.id === updated.id ? updated : pg)) })),
    });
  }

  function openProject(id: string) {
    projectStore.setCurrent(id);
    folderId = "all";
    kanban = "all";
    selectedId = null;
    void loadThumbs(id);
  }


  /** Buka Hasil & Ekspor untuk satu kartu dashboard (pintu-ekspor D2). */
  async function openResults(id: string) {
    if (busy) return;
    if (id !== projectStore.currentId) {
      openProject(id);
      try {
        await refreshProject(id);
      } catch (e) {
        error = String(e);
        return;
      }
    }
    showReview = true;
  }

  async function createFolder() {
    const currentId = projectStore.currentId;
    if (!currentId) return;
    const n = folderName.trim();
    if (!n) return;
    busy = true;
    error = "";
    try {
      const parent = folderId === "all" ? null : folderId;
      const f = await api.createFolder(currentId, n, parent);
      folderName = "";
      const updated = await api.getProject(currentId);
      projectStore.replaceProject(currentId, updated);
      folderId = f.id;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function deleteFolder(id: string) {
    const currentId = projectStore.currentId;
    if (!currentId) return;
    if (!confirm($t("confirmDeleteFolder"))) return;
    busy = true;
    error = "";
    try {
      await api.deleteFolder(currentId, id);
      const updated = await api.getProject(currentId);
      projectStore.replaceProject(currentId, updated);
      if (folderId === id) folderId = "all";
      selectedId = null;
      await loadThumbs(currentId);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function detectAll() {
    if (!current || scopedPages.length === 0) return;
    busy = true;
    error = "";
    const names = new Map(scopedPages.map((p) => [p.id, fileName(p.path)]));
    const { done, cancel, gen } = detectRun.start(current.id, scopedPages.map((p) => p.id), names);
    pendingDetectCancel = cancel;
    try {
      const updated = await done;
      if (updated && current) projectStore.replaceProject(current.id, updated);
    } catch (e) {
      if (detectRun.isCurrent(gen)) error = String(e);
    } finally {
      if (detectRun.isCurrent(gen)) busy = false;
    }
  }

  /** Tutup overlay: selesai → buang state; jalan → sembunyikan saja. */
  function closeDetectOverlay() {
    detectRun.closeOverlay();
  }

  async function importFolder() {
    const dir = await open({ directory: true });
    if (typeof dir === "string") await doImport([dir]);
  }

  async function importFiles() {
    const files = await open({
      multiple: true,
      // iOS: salin pilihan ke sandbox app (default pun copy, ditulis eksplisit
      // supaya tidak tergantung default plugin). Folder picker tak ada di iOS.
      fileAccessMode: "copy",
      filters: [
        { name: "Images", extensions: [...IMAGE_EXTS] },
        { name: "Archives", extensions: [...ARCHIVE_EXTS] },
        { name: "PDF documents", extensions: [...PDF_EXTS] },
      ],
    });
    if (Array.isArray(files)) await doImport(files);
    else if (typeof files === "string") await doImport([files]);
  }

  onMount(() => {
    // Error invoke tidak boleh hilang di panel: tampilkan command yang gagal
    // supaya user bisa buka Diagnostics dan lihat detailnya.
    const offInvoke = onInvoke((m) => {
      lastInvokeError = m.ok ? null : { command: m.command, error: m.error ?? "?" };
    });
    void refresh();
    void getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") dragging = true;
      else if (event.payload.type === "leave") dragging = false;
      else if (event.payload.type === "drop") {
        dragging = false;
        void doImport(event.payload.paths);
      }
    });
    // onMount tidak boleh async di Svelte 5 — return cleanup langsung.
    return () => {
      offInvoke();
      selection.dispose();
    };
  });

  onDestroy(() => {
    pendingDetectCancel?.();
    pendingDetectCancel = null;
  });
</script>

<main class="flex min-h-screen flex-col bg-bg text-ink" data-theme={$theme}>
  {#if projectStore.loaded && projects.length > 0}
  <header class="sticky top-0 z-30 border-b border-line bg-surface/80 backdrop-blur-md">
    <div class="flex items-center gap-2 px-4 pt-2">
      <img src={iconUrl} alt="Kuron Studio" class="size-6 shrink-0" />
      <h1 class="font-display shrink-0 text-base font-semibold tracking-tight">Kuron Studio</h1>
      <p class="hidden text-[11px] text-ink-3 sm:block">{$t("appTagline")}</p>
    </div>
    <div class="flex flex-wrap items-center gap-2 px-4 py-2">
    {#if projects.length > 0}
      <ProjectSwitcher current={current} {projects} onPick={(id) => openProject(id)} />
    {/if}

    <div class="ml-auto flex min-w-0 flex-1 items-center gap-1.5 overflow-x-auto sm:flex-none sm:flex-wrap sm:overflow-visible">
      <Button variant="ghost" size="sm" class="font-mono" onclick={() => (readingDir = readingDir === "rtl" ? "ltr" : "rtl")} title="Urutan baca bubble">
        {readingDir === "rtl" ? "RTL→" : "←LTR"}
      </Button>
      <Button variant="ghost" size="icon" onclick={() => theme.update((v: Theme) => (v === "dark" ? "light" : "dark"))} title="Tema gelap/terang" aria-label="Tema gelap/terang">
        {$theme === "dark" ? "◐" : "◑"}
      </Button>
      <Button variant="ghost" size="icon" class="font-mono" onclick={cycleLang} title="Bahasa / Language / 语言" aria-label="Ganti bahasa">
        {$lang.toUpperCase()}
      </Button>

      {#if !isMobile}<div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>{/if}

      <Button variant="ghost" size="sm" onclick={() => (showProviders = true)}>{$t("providers")}</Button>
      {#if !isMobile}<UpdateButton />{/if}
      <Button
        variant="ghost"
        size="sm"
        onclick={() => (showBatch = true)}
        disabled={!current || !batchOpts?.providerId}
        title={batchOpts?.providerId ? "Batch translate" : "Buka satu halaman di editor + pilih provider dulu"}
      >{$t("batch")}</Button>
      <Button variant="ghost" size="sm" onclick={() => (showGlossary = true)}>{$t("glossary")}</Button>
      <Button
        variant="ghost"
        size="sm"
        onclick={() => (showReview = true)}
        disabled={!current}
      >{$t("resultsExport")}</Button>

      <div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>

      <Button
        variant="primary"
        size="sm"
        onclick={detectAll}
        disabled={!current || busy || scopedPages.length === 0}
      >{$t("detectAll")}</Button>
      <div
        class="relative"
        onfocusout={(e) => { if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node | null)) showImportMenu = false; }}
      >
        <Button
          variant="default"
          size="sm"
          onclick={() => (showImportMenu = !showImportMenu)}
          onkeydown={(e) => { if (e.key === "Escape") showImportMenu = false; }}
          disabled={!current || busy}
          aria-haspopup="menu"
          aria-expanded={showImportMenu}
        >⬆ {$t("importAction")} ▾</Button>
        {#if showImportMenu}
          <div
            class="absolute top-full z-50 mt-1 w-44 overflow-hidden rounded-md border border-line-strong bg-surface-2 py-1"
            role="menu"
            aria-label={$t("importAction")}
          >
            {#if !isMobile}
              <button
                class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs transition-colors hover:bg-raised"
                role="menuitem"
                onclick={() => { showImportMenu = false; void importFolder(); }}
                onkeydown={(e) => {
                  if (e.key === "Escape") showImportMenu = false;
                  if (e.key === "ArrowDown") (e.currentTarget.nextElementSibling as HTMLElement | null)?.focus();
                }}
              >📁 {$t("importFolder")}</button>
            {/if}
            <button
              class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs transition-colors hover:bg-raised"
              role="menuitem"
              onclick={() => { showImportMenu = false; void importFiles(); }}
              onkeydown={(e) => {
                if (e.key === "Escape") showImportMenu = false;
                // iOS: folder picker tidak ada (FolderPickerNotImplemented) —
                // opsi folder disembunyikan total, bukan disabled.
                if (e.key === "ArrowUp") (e.currentTarget.previousElementSibling as HTMLElement | null)?.focus();
              }}
            >🖼️ {$t("importFiles")}</button>
          </div>
        {/if}
      </div>

      {#if !isMobile}<div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>{/if}

      {#if !isMobile}<Button
        variant="default"
        size="sm"
        onclick={cleanPages}
        disabled={!current || busy || brokenCount === 0}
        title={brokenCount > 0 ? `${brokenCount} halaman preview gagal` : $t("nothingToClean")}
      >
        {#if brokenCount > 0}<span class="tnum text-danger">{brokenCount}</span>{/if}
        {$t("cleanImages")}
      </Button>{/if}
      <Button
        variant="danger"
        size="sm"
        onclick={deleteProject}
        disabled={!current || busy}
        title={$t("confirmDeleteProject")}
      >{$t("deleteProject")}</Button>

      {#if !isMobile}<div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>{/if}

      {#if !isMobile}<Button
        variant={lastInvokeError ? "danger" : "ghost"}
        size="icon"
        onclick={() => (showDiagnostics = true)}
        title={lastInvokeError
          ? `${lastInvokeError.command}: ${lastInvokeError.error}`
          : "Diagnostics & log"}
        aria-label="Diagnostics"
      >⚙</Button>{/if}
    </div>
    </div>
  </header>
  {/if}

  <AlertBanners
    {lastInvokeError}
    {error}
    {info}
    onOpenDiagnostics={() => (showDiagnostics = true)}
    onClearInfo={() => (info = "")}
  />

  {#if !projectStore.loaded}
    <div class="flex flex-1 items-center justify-center p-10" aria-hidden="true">
      <div class="h-10 w-10 animate-pulse rounded-full bg-raised"></div>
    </div>
  {:else if projects.length === 0}
    <Onboarding busy={busy} onCreate={(n) => void createProject(n)} onSample={() => void createSample()} />
  {:else if current}
    {@const dashStats = projectStats(current)}
    {@const dashPct = Math.round(dashStats.progress * 100)}
    <section class="flex flex-col gap-2 border-b border-line bg-surface/50 px-5 py-2.5 sm:flex-row sm:items-stretch sm:gap-3" aria-label={$t("dashboard")}>
      <div class="w-full sm:w-60 sm:shrink-0">
        <div class="truncate text-sm font-semibold" title={current.name}>{current.name}</div>
        <div class="tnum mt-0.5 text-[11px] text-ink-3">
          {dashStats.done}/{dashStats.total} · {dashPct}%{#if dashStats.failed > 0}<span class="text-danger"> · {dashStats.failed} {$t("kanbanFailed")}</span>{/if}
        </div>
        <div class="mt-1.5 h-1 overflow-hidden rounded bg-raised">
          <div class="h-full rounded bg-accent" style="width: {dashPct}%"></div>
        </div>
        <div class="tnum mt-1 truncate text-[10px] text-ink-3" title={$t("lastWorked")}>
          {dashStats.todo + dashStats.doing} {$t("remainingTodo")} · {current.updatedAt ? new Date((current.updatedAt ?? 0) * 1000).toLocaleDateString() : "—"}
        </div>
      </div>
      <div class="hidden w-px self-stretch bg-line sm:block" aria-hidden="true"></div>
      <div class="flex min-w-0 flex-1 items-stretch gap-1.5 overflow-x-auto">
        {#each projects as p (p.id)}
          <ProjectCard
            project={p}
            selected={p.id === projectStore.currentId}
            variant="card"
            onOpen={(id) => openProject(id)}
            onResults={(id) => void openResults(id)}
          />
        {/each}
      </div>
      <div class="hidden w-px self-stretch bg-line sm:block" aria-hidden="true"></div>
      <form class="flex w-full flex-col justify-center gap-1 sm:w-44 sm:shrink-0" onsubmit={(e) => { e.preventDefault(); void createProject(); }}>
        <input
          class="h-7 w-full rounded-md border border-line bg-surface-2 px-2 text-xs text-ink
                 placeholder:text-ink-3 outline-none transition-colors focus:border-accent"
          placeholder={$t("projectName")}
          aria-label={$t("projectName")}
          bind:value={name}
        />
        <Button type="submit" variant="default" size="sm" disabled={busy || !name.trim()}>{$t("create")}</Button>
      </form>
    </section>
    <FolderBar
      {current}
      {nested}
      {crumb}
      {childFolders}
      {folderId}
      currentFolderName={currentFolder?.name ?? ""}
      {folderName}
      {busy}
      onSelectFolder={(id) => (folderId = id)}
      onDeleteFolder={(id) => void deleteFolder(id)}
      onRenamed={() => projectStore.currentId && void refreshProject(projectStore.currentId)}
      onError={(msg) => (error = msg)}
      onFolderName={(v) => (folderName = v)}
      onCreateFolder={() => void createFolder()}
    />
    <KanbanBar
      {kanban}
      counts={{
        all: allPages(current).length,
        todo: kanbanGroups.todo.length,
        doing: kanbanGroups.doing.length,
        done: kanbanGroups.done.length,
        failed: kanbanGroups.failed.length,
      }}
      {sortKey}
      {sortDir}
      {pageView}
      {visiblePages}
      {selection}
      {busy}
      onKanban={(k) => (kanban = k)}
      onSortKey={(k) => setSortKey(k)}
      onSetPageView={(v) => void setPageView(v)}
      onMoveRequest={() => (showMove = true)}
    />
    {#if queue.total > 0 && queue.done < queue.total}
      <p role="status" class="tnum border-b border-line bg-surface-2 px-5 py-1.5 text-xs text-ink-2">
        {$t("loadingPreviews").replace("{done}", String(queue.done)).replace("{total}", String(queue.total))}
      </p>
    {:else if queue.failed > 0}
      <p role="status" class="flex items-center gap-2 border-b border-line bg-surface-2 px-5 py-1.5 text-xs text-ink-2">
        <span class="tnum">{$t("loadingPreviews").replace("{done}", String(queue.done)).replace("{total}", String(queue.total))} · {$t("previewsFailed").replace("{n}", String(queue.failed))}</span>
        <Button variant="ghost" size="sm" onclick={retryThumbs}>{$t("retryThumbs")}</Button>
      </p>
    {/if}
    {#if pageView === "grid"}
      <PageGrid
        pages={orderedPages}
        thumbs={queue.thumbs}
        broken={queue.broken}
        selectedIds={selection.selectedIds}
        selectMode={selection.selectMode}
        {dragging}
        onCardClick={(e, pg) => onCardClick(e, pg)}
        onCardPress={(pg) => selection.press(pg)}
        onRelease={() => selection.cancelHold()}
        onToggleSelect={(id) => selection.toggleSelect(id)}
        observeCard={queue.observeCard}
      />
    {:else}
      <PageList
        pages={orderedPages}
        thumbs={queue.thumbs}
        broken={queue.broken}
        selectedIds={selection.selectedIds}
        selectMode={selection.selectMode}
        onCardClick={(e, pg) => onCardClick(e, pg)}
        onOpen={(pg) => openEditor(pg)}
        onToggleSelect={(id) => selection.toggleSelect(id)}
        observeCard={queue.observeCard}
      />
    {/if}
    {#if scopedPages.length === 0}
      <p class="mt-16 px-8 text-center text-sm text-ink-3">
        {#if dragging}Lepaskan file untuk import…{:else if isMobile}{$t("emptyHintMobile")}{:else}{$t("emptyHint")}{/if}
      </p>
    {:else if visiblePages.length === 0}
      <p class="mt-8 px-8 text-center text-sm text-ink-3">— 0 —</p>
    {/if}
  {/if}

  {#if selectedPage && projectStore.currentId}
    <div class="fixed inset-0 z-50 flex flex-col bg-bg">
      {#key selectedPage.id}
        <EditorPanel
          projectId={projectStore.currentId}
          page={selectedPage}
        fullImageUrl={fullUrls[selectedPage.id] ?? queue.thumbs[selectedPage.id] ?? ""}
        fallbackUrl={queue.thumbs[selectedPage.id] ?? ""}
          openError={openError}
          {readingDir}
          onClose={closeEditor}
          onSaved={onSaved}
          onBatchOpts={(o) => (batchOpts = { providerId: o.providerId, ...o.opts, readingDirection: readingDir }) }
        />
      {/key}
    </div>
  {/if}
  {#if detectRun.state && !detectRun.hidden}
    <DetectProgress state={detectRun.state} onClose={closeDetectOverlay} />
  {/if}

  {#if showBatch && current && batchOpts}
    {@const cur = current}
    <BatchPanel
      pages={scopedPages}
      opts={batchOpts}
      onDone={() => { void refreshProject(cur.id); showBatch = false; }}
      onClose={() => (showBatch = false)}
    />
  {/if}

  {#if showGlossary}
    <GlossaryPanel onClose={() => (showGlossary = false)} />
  {/if}

  {#if showMove && current && projectStore.currentId}
    {@const cur = current}
    {@const cid = projectStore.currentId}
    <MoveDialog
      project={cur}
      fromFolderId={folderId}
      pageIds={selection.selectedIds}
      {busy}
      onClose={() => (showMove = false)}
      onError={(msg) => (error = msg)}
      onMoved={(res, destName) => {
        // Dialog tetap terbuka: tampilkan layar sukses, user yang menutup.
        void (async () => {
          const updated = await api.getProject(cid);
          projectStore.replaceProject(cid, updated);
          // Seleksi + thumbnail basi dibuang: halaman sudah pindah folder.
          const live = new Set(allPages(updated).map((pg) => pg.id));
          queue.pruneThumbs(live);
          for (const id of Object.keys(fullUrls)) if (!live.has(id)) delete fullUrls[id];
          selection.reset();
          fullUrls = { ...fullUrls };
          info = $t("moveDone").replace("{n}", String(res.moved)).replace("{d}", destName)
            + (res.skipped > 0 ? ` (${$t("skippedCount").replace("{n}", String(res.skipped))})` : "");
        })();
      }}
    />
  {/if}

  {#if showReview && current}
    {@const cur = current}
    <ReviewGrid
      project={cur}
      opts={batchOpts}
      selectedIds={selection.selectedIds}
      onRefresh={() => { void refreshProject(cur.id); }}
      onClose={() => (showReview = false)}
    />
  {/if}

  {#if showProviders}
    <ProviderSettings onClose={() => (showProviders = false)} />
  {/if}

  {#if showDiagnostics}
    <DiagnosticsPanel onClose={() => (showDiagnostics = false)} />
  {/if}
</main>

<svelte:window onkeydown={onViewKey} />
