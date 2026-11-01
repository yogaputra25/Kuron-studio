<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { tick } from "svelte";
  import { onDragDrop, onShowAbout } from "./lib/app/events";
  import { matchShortcut } from "./lib/app/shortcuts";
  import { pickFiles, pickFolder } from "./lib/app/dialogs";

  import { isMobile, isNarrow, watchViewport } from "./lib/app/device";
  import { read, STORAGE_KEYS, write } from "./lib/app/storage";
  import { api } from "./lib/contracts/api";
  import AlertBanners from "./lib/app/shell/AlertBanners.svelte";
  import AppHeader from "./lib/app/shell/AppHeader.svelte";
  import OverlayHost from "./lib/app/shell/OverlayHost.svelte";
  import PageGrid from "./lib/library/PageGrid.svelte";
  import PageList from "./lib/library/PageList.svelte";
  import { t, theme } from "./lib/app/i18n";
  import type { Page, Project, ReadingDirection } from "./lib/contracts/types";
  import type { TranslateOpts } from "./lib/provider/translateOpts";
  import { allPages, childrenOf, folderPath, groupKanban, hasNesting, projectStats, subtreePages } from "./lib/library/library";
  import type { KanbanFilter, PageView, SortKey } from "./lib/library/library";
  import Button from "./lib/ui/Button.svelte";
  import type { SnackbarTone } from "./lib/ui/Snackbar.svelte";
  import { fileName, sortPagesByBubbles, sortPagesByName, sortPagesByStatus } from "./lib/library/status";
  import Onboarding from "./lib/app/Onboarding.svelte";
  import FolderBar from "./lib/app/shell/FolderBar.svelte";
  import KanbanBar from "./lib/app/shell/KanbanBar.svelte";
  import ProjectCard from "./lib/app/shell/ProjectCard.svelte";
  import { onInvoke } from "./lib/contracts/log";
  import { createProjectStore } from "./lib/library/projectStore.svelte.ts";

  import { createSelection, orderIds } from "./lib/library/selection.svelte.ts";
  import { createThumbnailQueue } from "./lib/previews/thumbnailQueue.svelte.ts";
  import { createDetectRun } from "./lib/detection/detectRun.svelte.ts";

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
  // detect-ux-pack 3.1: satu tombol Import + menu pola switcher (state di AppHeader).
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
  // Persist via app/storage (FE-8): string mentah, guard grid|list.
  function loadPageView(): "grid" | "list" {
    const v = read(STORAGE_KEYS.pageView, "grid");
    return v === "grid" || v === "list" ? v : "grid";
  }
  let pageView = $state<PageView>(loadPageView());
  let scrollByView: Record<PageView, number> = { grid: 0, list: 0 };

  /** Ganti mode: simpan scroll lama, pulihkan scroll mode tujuan. */
  async function setPageView(v: "grid" | "list") {
    if (v === pageView) return;
    scrollByView[pageView] = window.scrollY;
    pageView = v;
    write(STORAGE_KEYS.pageView, v);
    await tick();
    window.scrollTo(0, scrollByView[v] ?? 0);
  }

  /** G/L global + shortcut desktop Cmd/Ctrl (abaikan saat editor/input fokus). */
  function onViewKey(e: KeyboardEvent) {
    if (selectedPage) return;
    const t = e.target as HTMLElement | null;
    const tag = t?.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || !!t?.isContentEditable) return;
    // Cmd (macOS) / Ctrl + tombol: provider, batch, hasil, detect, import.
    const act = matchShortcut(e);
    if (act) {
      e.preventDefault();
      if (act === "providers") showProviders = true;
      else if (act === "batch" && current) showBatchDialog = true;
      else if (act === "results" && current) showReview = true;
      else if (act === "detect" && !busy) void detectAll();
      else if (act === "import-files" && current && !busy) void importFiles();
      return;
    }
    if (e.ctrlKey || e.metaKey || e.altKey) return;
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
  /** Dialog opsi batch (dari list): null = ikut scopedPages saat submit. */
  let batchPageIds = $state<string[] | null>(null);
  let showBatchDialog = $state(false);
  let showGlossary = $state(false);
  let showReview = $state(false);
  /** Viewport sempit (<768px): header ringkas + drawer (#14). */
  let narrow = $state(false);
  let showAbout = $state(false);
  let showDiagnostics = $state(false);
  /** Error invoke terakhir — banner merah di header, selalu terlihat. */
  let lastInvokeError = $state<{ command: string; error: string } | null>(null);
  /** Hasil cek update (toast global) — dirender di AlertBanners, bukan inline. */
  let updateToast = $state<{ message: string; tone: SnackbarTone } | null>(null);
  // Opts terakhir dari editor (provider + translate opts) — referensi langsung
  // milik EditorPanel, BUKAN salinan merge (FE-6): tak ada snapshot basi.
  let batchReq = $state<{ providerId: string; opts: TranslateOpts } | null>(null);

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

  /** Batch selesai: refresh project + tutup. Dipindah utuh dari markup. */
  function handleBatchDone() {
    const cur = projectStore.current;
    if (cur) void refreshProject(cur.id);
    showBatch = false;
  }

  /** Pindah folder selesai: refresh project + buang seleksi/thumb basi. */
  async function handleMoved(res: { moved: number; skipped: number }, destName: string, cid: string) {
    const updated = await api.getProject(cid);
    projectStore.replaceProject(cid, updated);
    const live = new Set(allPages(updated).map((pg) => pg.id));
    queue.pruneThumbs(live);
    for (const id of Object.keys(fullUrls)) if (!live.has(id)) delete fullUrls[id];
    selection.reset();
    fullUrls = { ...fullUrls };
    info = $t("moveDone").replace("{n}", String(res.moved)).replace("{d}", destName)
      + (res.skipped > 0 ? ` (${$t("skippedCount").replace("{n}", String(res.skipped))})` : "");
  }

  async function importFolder() {
    const dir = await pickFolder();
    if (dir) await doImport([dir]);
  }

  async function importFiles() {
    const files = await pickFiles();
    if (files.length > 0) await doImport(files);
  }

  onMount(() => {
    // Error invoke tidak boleh hilang di panel: tampilkan command yang gagal
    // supaya user bisa buka Diagnostics dan lihat detailnya.
    const offInvoke = onInvoke((m) => {
      lastInvokeError = m.ok ? null : { command: m.command, error: m.error ?? "?" };
    });
    void refresh();
    const offDragDrop = onDragDrop((event) => {
      if (event.payload.type === "over") dragging = true;
      else if (event.payload.type === "leave") dragging = false;
      else if (event.payload.type === "drop") {
        dragging = false;
        void doImport(event.payload.paths);
      }
    });
    // onMount tidak boleh async di Svelte 5 — return cleanup langsung.
    narrow = isNarrow();
    const unwatchViewport = watchViewport(() => (narrow = isNarrow()));
    const offShowAbout = onShowAbout(() => { showAbout = true; });
    return () => {
      offInvoke();
      offDragDrop();
      unwatchViewport();
      offShowAbout();
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
    <AppHeader
      {current}
      {projects}
      {busy}
      {readingDir}
      {lastInvokeError}
      brokenCount={queue.failed}
      canDetect={!!current && scopedPages.length > 0}
      {narrow}
      onToggleReadingDir={() => (readingDir = readingDir === "rtl" ? "ltr" : "rtl")}
      onProviders={() => (showProviders = true)}
      onBatch={() => (showBatchDialog = true)}
      onGlossary={() => (showGlossary = true)}
      onResults={() => (showReview = true)}
      onDetect={detectAll}
      onImportFolder={() => void importFolder()}
      onImportFiles={() => void importFiles()}
      onClean={() => void cleanPages()}
      onDelete={() => void deleteProject()}
      onOpenDiagnostics={() => (showDiagnostics = true)}
      onAbout={() => (showAbout = true)}
      onPickProject={(id) => openProject(id)}
      onUpdateToast={(message, tone) => (updateToast = { message, tone })}
    />
  {/if}

  <AlertBanners
    {lastInvokeError}
    {error}
    {info}
    {updateToast}
    onOpenDiagnostics={() => (showDiagnostics = true)}
    onClearInfo={() => (info = "")}
    onClearUpdateToast={() => (updateToast = null)}
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

  <OverlayHost
    {selection}
    {queue}
    {detectRun}
    {selectedPage}
    {current}
    currentId={projectStore.currentId}
    {fullUrls}
    {openError}
    {readingDir}
    {showBatch}
    {batchReq}
    {showBatchDialog}
    {batchPageIds}
    {showGlossary}
    {showMove}
    {folderId}
    {busy}
    {showReview}
    {showProviders}
    {showDiagnostics}
    {showAbout}
    scopedPages={visiblePages}
    onCloseEditor={closeEditor}
    {onSaved}
    onBatchOpts={(o) => (batchReq = o)}
    onBatchDialogClose={() => (showBatchDialog = false)}
    onBatchDialogSubmit={(req, pageIds) => {
      batchReq = req;
      batchPageIds = pageIds;
      showBatchDialog = false;
      showBatch = true;
    }}
    onDetectClose={closeDetectOverlay}
    onBatchDone={handleBatchDone}
    onBatchClose={() => (showBatch = false)}
    onGlossaryClose={() => (showGlossary = false)}
    onMoveClose={() => (showMove = false)}
    onMoveError={(msg) => (error = msg)}
    onMoved={(res, destName, cid) => void handleMoved(res, destName, cid)}
    onReviewClose={() => (showReview = false)}
    onRefreshReview={() => { const c = projectStore.current; if (c) void refreshProject(c.id); }}
    onProvidersClose={() => (showProviders = false)}
    onDiagnosticsClose={() => (showDiagnostics = false)}
    onAboutClose={() => (showAbout = false)}
  />

</main>

<svelte:window onkeydown={onViewKey} />
