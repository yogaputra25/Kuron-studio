<script lang="ts">
  import { onMount } from "svelte";
  import { tick } from "svelte";
  import { fly, scale } from "svelte/transition";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { listen } from "@tauri-apps/api/event";
  import iconUrl from "../icon.svg";
  import { isMobile } from "./lib/app/device";
  import { api } from "./lib/contracts/api";
  import UpdateButton from "./lib/app/UpdateButton.svelte";
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
  import { allPages, childrenOf, folderPath, groupKanban, hasNesting, projectStats, rangeSelect, subtreePages, subtreeStats } from "./lib/library/library";
  import Button from "./lib/ui/Button.svelte";
  import { fileName, sortPagesByBubbles, sortPagesByName, sortPagesByStatus } from "./lib/library/status";
  import FolderChip from "./lib/library/FolderChip.svelte";
  import MoveDialog from "./lib/library/MoveDialog.svelte";
  import Onboarding from "./lib/app/Onboarding.svelte";
  import Snackbar from "./lib/ui/Snackbar.svelte";
  import DetectProgress from "./lib/detection/DetectProgress.svelte";
  import { initialDetectState, reduceDetectProgress, type DetectState } from "./lib/detection/detectProgress";
  import type { DetectProgress as DetectProgressEv } from "./lib/contracts/types";
  import DiagnosticsPanel from "./lib/app/DiagnosticsPanel.svelte";
  import { onInvoke } from "./lib/contracts/log";
  import { PREFETCH_MARGIN, PREVIEW_CONCURRENCY, pickNext } from "./lib/previews/previewQueue";
  import { summarizePreviews, type PreviewSample } from "./lib/previews/previewMetrics";

  let projects = $state<Project[]>([]);
  /** false sampai daftar project pertama selesai dimuat (cegah flash onboarding). */
  let loaded = $state(false);
  let currentId = $state<string | null>(null);
  let name = $state("");
  let folderId = $state<string | "all">("all");
  let kanban = $state<"all" | "todo" | "doing" | "done" | "failed">("all");
  let folderName = $state("");
  /** Halaman terpilih (multi-select untuk pindah folder). */
  let selectedIds = $state<string[]>([]);
  let selAnchor = $state<string | null>(null);
  /** Mode pilih manual: checkbox + "Pilih semua" baru muncul di mode ini. */
  let selectMode = $state(false);
  let holdTimer: ReturnType<typeof setTimeout> | null = null;
  let heldSelect = false;
  let showMove = $state(false);
  let showProjMenu = $state(false);
  // detect-ux-pack 3.1: satu tombol Import + menu pola switcher.
  let showImportMenu = $state(false);
  let projQuery = $state("");
  let thumbs = $state<Record<string, string>>({});
  let fullUrls = $state<Record<string, string>>({});
  let selectedId = $state<string | null>(null);
  let readingDir = $state<ReadingDirection>("rtl");
  let error = $state("");
  let info = $state("");
  /** pageId → alasan preview gagal (file hilang / tidak bisa di-decode). */
  let broken = $state<Record<string, string>>({});
  // grid-preview-perf D1-D3: antrean thumbnail berprioritas viewport.
  // `loadGen` guard generasi — ganti project membuang hasil basi.
  let loadGen = 0;
  let thumbPending: string[] = [];
  let thumbInflight = new Set<string>();
  let thumbActive = 0;
  let thumbByPath = new Map<string, string>();
  let visibleIds = new Set<string>();
  let thumbTotal = $state(0);
  let thumbDone = $state(0);
  let thumbFailed = $state(0);
  let thumbT0 = 0;
  let thumbFirstAt = 0;
  let thumbSamples: PreviewSample[] = [];
  // detect-ux-pack D1: overlay progres non-blocking + guard generasi.
  let detectState = $state<DetectState | null>(null);
  let detectHidden = $state(false);
  let detectGen = 0;
  // page-list-view D4/D5: mode Grid/List + pref + scroll per mode.
  function loadPageView(): "grid" | "list" {
    try {
      return localStorage.getItem("kuron-page-view") === "list" ? "list" : "grid";
    } catch {
      return "grid";
    }
  }
  let pageView = $state<"grid" | "list">(loadPageView());
  let scrollByView: Record<"grid" | "list", number> = { grid: 0, list: 0 };

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
  type SortKey = "name" | "status" | "bubbles";
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
    const updated = await api.getProject(id);
    projects = projects.map((p) => (p.id === id ? updated : p));
  }

  const current = $derived(projects.find((p) => p.id === currentId) ?? null);
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
  const brokenCount = $derived(Object.keys(broken).length);
  const filteredProjects = $derived(
    projQuery.trim()
      ? projects.filter((p) => p.name.toLowerCase().includes(projQuery.trim().toLowerCase()))
      : projects,
  );

  async function refresh() {
    try {
      projects = await api.listProjects();
      if (!currentId && projects.length > 0) currentId = projects[0].id;
    } catch (e) {
      // Hanya `list_projects` yang gagal di sini. Label "Backend
      // unreachable" hanya sah kalau invoke-nya sendiri yang tidak jalan.
      error = `Backend unreachable (jalankan via 'pnpm tauri dev'): ${e}`;
    } finally {
      loaded = true;
    }
    if (currentId) await loadThumbs(currentId);
  }

  // M4-6: apply persisted theme before first paint of the shell.
  $effect(() => {
    document.documentElement.dataset.theme = $theme;
  });
  const cycleLang = () => {
    lang.update((v: Lang) => (v === "id" ? "en" : v === "en" ? "zh" : "id"));
  };

  // grid-preview-perf D1-D3: antrean konkurensi-4 + prioritas viewport +
  // guard generasi. Ganti `Promise.allSettled` semua-halaman (isu #11: N
  // decode paralel + N payload IPC satu gelombang).
  function loadThumbs(projectId: string) {
    const proj = projects.find((p) => p.id === projectId);
    if (!proj) return;
    const myGen = ++loadGen;
    visibleIds = new Set<string>();
    thumbInflight = new Set<string>();
    thumbActive = 0;
    thumbByPath = new Map(allPages(proj).map((pg) => [pg.id, pg.path]));
    // Skip yang sudah broken: retry file sidecar tiap ganti project mahal.
    // `cleanPages`/retry yang membersihkan peta ini sebagai jalur recovery.
    thumbPending = sortPagesByName(allPages(proj))
      .filter((pg) => !thumbs[pg.id] && !broken[pg.id])
      .map((pg) => pg.id);
    thumbTotal = thumbPending.length;
    thumbDone = 0;
    thumbFailed = 0;
    thumbT0 = performance.now();
    thumbFirstAt = 0;
    thumbSamples = [];
    pumpThumbs(myGen);
  }

  /** Ambil id berikutnya: terlihat dulu, lalu prefetch awal antrean. */
  function nextThumbId(): string | null {
    const done = (id: string) => !!thumbs[id] || !!broken[id];
    return pickNext(thumbPending, visibleIds, thumbInflight, done, PREFETCH_MARGIN);
  }

  /** Pompa worker sampai batas konkurensi; hasil basi (gen beda) dibuang. */
  function pumpThumbs(myGen: number) {
    while (thumbActive < PREVIEW_CONCURRENCY && myGen === loadGen) {
      const id = nextThumbId();
      if (!id) return; // tunggu intersection berikutnya
      thumbInflight.add(id);
      thumbActive++;
      const path = thumbByPath.get(id) ?? "";
      const t = performance.now();
      void api
        .getImagePreview(path)
        .then((url) => {
          if (myGen !== loadGen) return;
          thumbSamples.push({ ms: performance.now() - t, bytes: url.length });
          thumbs = { ...thumbs, [id]: url };
          if (!thumbFirstAt) thumbFirstAt = performance.now();
          thumbDone++;
        })
        .catch((e) => {
          if (myGen !== loadGen) return;
          broken = { ...broken, [id]: String(e) };
          thumbFailed++;
          thumbDone++;
        })
        .finally(() => {
          thumbInflight.delete(id);
          thumbActive--;
          if (myGen !== loadGen) return;
          if (thumbTotal > 0 && thumbDone >= thumbTotal) {
            console.info(`[preview] ${JSON.stringify(summarizePreviews(thumbSamples, thumbFirstAt - thumbT0))}`);
          }
          pumpThumbs(myGen);
        });
    }
  }

  /** Coba lagi thumbnail yang gagal di project aktif. */
  function retryThumbs() {
    if (!currentId || !current) return;
    const live = new Set(allPages(current).map((pg) => pg.id));
    for (const id of Object.keys(broken)) if (live.has(id)) delete broken[id];
    broken = { ...broken };
    void loadThumbs(currentId);
  }

  /** Action kartu grid: laporkan keterlihatan ke antrean (D2). */
  function observeCard(node: HTMLElement, id: string) {
    const io = new IntersectionObserver(
      (es) => {
        for (const e of es) {
          if (e.isIntersecting) {
            visibleIds.add(id);
            pumpThumbs(loadGen);
          }
        }
      },
      { rootMargin: "400px 0px" },
    );
    io.observe(node);
    return {
      destroy() {
        io.disconnect();
      },
    };
  }

  async function cleanPages() {
    if (!currentId) return;
    if (!confirm($t("confirmClean"))) return;
    busy = true;
    error = "";
    try {
      const res = await api.cleanPages(currentId, folderId === "all" ? null : folderId);
      const updated = await api.getProject(currentId);
      projects = projects.map((p) => (p.id === currentId ? updated : p));
      // Halaman hilang dari store — buang juga thumbnail-nya.
      const live = new Set(allPages(updated).map((pg) => pg.id));
      for (const id of Object.keys(thumbs)) if (!live.has(id)) delete thumbs[id];
      for (const id of Object.keys(fullUrls)) if (!live.has(id)) delete fullUrls[id];
      for (const id of Object.keys(broken)) if (!live.has(id)) delete broken[id];
      selectedId = live.has(selectedId ?? "") ? selectedId : null;
      thumbs = { ...thumbs };
      fullUrls = { ...fullUrls };
      broken = { ...broken };
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
    if (!currentId || !current) return;
    if (!confirm($t("confirmDeleteProject"))) return;
    busy = true;
    error = "";
    const gone = currentId;
    try {
      await api.deleteProject(gone);
      thumbs = {};
      fullUrls = {};
      broken = {};
      selectedId = null;
      closeEditor();
      projects = projects.filter((p) => p.id !== gone);
      currentId = projects[0]?.id ?? null;
      if (currentId) await loadThumbs(currentId);
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
      projects = [...projects, p];
      currentId = p.id;
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
      projects = [...projects, p];
      openProject(p.id);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function pickProject(id: string) {
    showProjMenu = false;
    projQuery = "";
    openProject(id);
  }

  async function doImport(paths: string[]) {
    if (!currentId || paths.length === 0) return;
    busy = true;
    try {
      const target = folderId === "all" ? null : folderId;
      const res = await api.importPages(currentId, paths, target);
      const updated = await api.getProject(currentId);
      projects = projects.map((p) => (p.id === currentId ? updated : p));
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

  /** Seleksi ikut konteks: ganti project/folder = mulai dari kosong. */
  $effect(() => {
    const _scope = currentId + "|" + folderId;
    void _scope;
    selectedIds = [];
    selAnchor = null;
    selectMode = false;
  });

  function toggleSelect(id: string) {
    selectedIds = selectedIds.includes(id)
      ? selectedIds.filter((x) => x !== id)
      : [...selectedIds, id];
    selAnchor = id;
  }

  /** Pilih semua halaman yang tampil (ikut filter folder + kanban). */
  function selectAll() {
    selectedIds = visiblePages.map((p) => p.id);
    selAnchor = null;
  }

  function cancelHold() {
    if (holdTimer !== null) {
      clearTimeout(holdTimer);
      holdTimer = null;
    }
  }

  /** Tahan gambar ±0.5 dtk = masuk mode pilih + pilih kartu itu (pola mobile). */
  function onCardPress(pg: Page) {
    if (selectMode) return;
    heldSelect = false;
    cancelHold();
    holdTimer = setTimeout(() => {
      heldSelect = true;
      selectMode = true;
      if (!selectedIds.includes(pg.id)) {
        selectedIds = [...selectedIds, pg.id];
        selAnchor = pg.id;
      }
    }, 500);
  }

  function onCardClick(e: MouseEvent, pg: Page) {
    if (heldSelect) {
      // Lepas-tahan setelah long-press: jangan buka editor.
      heldSelect = false;
      return;
    }
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      selectMode = true;
      toggleSelect(pg.id);
    } else if (e.shiftKey && selAnchor) {
      e.preventDefault();
      selectMode = true;
      const order = sortPagesByName(visiblePages).map((p) => p.id);
      const range = rangeSelect(order, selAnchor, pg.id);
      selectedIds = [...new Set([...selectedIds, ...range])];
      selAnchor = pg.id;
    } else {
      openEditor(pg);
    }
  }

  function onSaved(updated: Page) {
    if (!currentId) return;
    projects = projects.map((p) =>
      p.id === currentId
        ? { ...p, folders: p.folders.map((f) => ({ ...f, pages: f.pages.map((pg) => (pg.id === updated.id ? updated : pg)) })) }
        : p,
    );
  }

  function openProject(id: string) {
    currentId = id;
    folderId = "all";
    kanban = "all";
    selectedId = null;
    void loadThumbs(id);
  }
  /** Buka Hasil & Ekspor untuk satu kartu dashboard (pintu-ekspor D2). */
  async function openResults(id: string) {
    if (busy) return;
    if (id !== currentId) {
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
      projects = projects.map((p) => (p.id === currentId ? updated : p));
      folderId = f.id;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function deleteFolder(id: string) {
    if (!currentId) return;
    if (!confirm($t("confirmDeleteFolder"))) return;
    busy = true;
    error = "";
    try {
      await api.deleteFolder(currentId, id);
      const updated = await api.getProject(currentId);
      projects = projects.map((p) => (p.id === currentId ? updated : p));
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
    const myGen = ++detectGen;
    const ids = scopedPages.map((p) => p.id);
    const names = new Map(scopedPages.map((p) => [p.id, fileName(p.path)]));
    detectState = initialDetectState(ids.length);
    detectHidden = false;
    busy = true;
    error = "";
    const unlisten = await listen<DetectProgressEv>("detect_progress", (e) => {
      if (myGen !== detectGen || !detectState) return;
      detectState = reduceDetectProgress(detectState, {
        ...e.payload,
        file: names.get(e.payload.pageId) ?? "",
      });
    });
    try {
      await api.detectBatch(ids);
      const updated = await api.getProject(current.id);
      if (myGen !== detectGen) return;
      projects = projects.map((p) => (p.id === current.id ? updated : p));
    } catch (e) {
      if (myGen === detectGen) error = String(e);
    } finally {
      unlisten();
      if (myGen === detectGen) busy = false;
    }
  }

  /** Tutup overlay: selesai → buang state; jalan → sembunyikan saja. */
  function closeDetectOverlay() {
    if (detectState?.finished) detectState = null;
    else detectHidden = true;
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
    return offInvoke;
  });
</script>

<main class="flex min-h-screen flex-col bg-bg text-ink" data-theme={$theme}>
  {#if loaded && projects.length > 0}
  <header class="sticky top-0 z-30 border-b border-line bg-surface/80 backdrop-blur-md">
    <div class="flex items-center gap-2 px-4 pt-2">
      <img src={iconUrl} alt="Kuron Studio" class="size-6 shrink-0" />
      <h1 class="font-display shrink-0 text-base font-semibold tracking-tight">Kuron Studio</h1>
      <p class="hidden text-[11px] text-ink-3 sm:block">{$t("appTagline")}</p>
    </div>
    <div class="flex flex-wrap items-center gap-2 px-4 py-2">
    {#if projects.length > 0}
      {@const cur = current}
      {@const curSt = cur ? projectStats(cur) : null}
      <div
        class="relative"
        onfocusout={(e) => { if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node | null)) showProjMenu = false; }}
      >
        <button
          class="flex h-8 max-w-64 items-center gap-2 rounded-md border border-line bg-surface-2 px-2.5 text-sm text-ink
                 outline-none transition-colors hover:border-line-strong focus:border-accent"
          aria-haspopup="listbox"
          aria-expanded={showProjMenu}
          aria-label={$t("selectProject")}
          onclick={() => (showProjMenu = !showProjMenu)}
          onkeydown={(e) => { if (e.key === "Escape") showProjMenu = false; }}
        >
          <span class="min-w-0 flex-1 truncate text-left">{cur?.name ?? "—"}</span>
          {#if curSt}
            <span class="tnum shrink-0 text-[11px] text-ink-3">{curSt.done}/{curSt.total}</span>
            <span class="h-1 w-12 shrink-0 overflow-hidden rounded bg-raised" aria-hidden="true">
              <span class="block h-full rounded bg-accent" style="width: {Math.round(curSt.progress * 100)}%"></span>
            </span>
          {/if}
          <span class="shrink-0 text-[10px] text-ink-3" aria-hidden="true">{showProjMenu ? "▴" : "▾"}</span>
        </button>
        {#if showProjMenu}
          <div
            class="absolute left-0 top-full z-50 mt-1 w-72 overflow-hidden rounded-md border border-line-strong bg-surface-2 py-1"
            transition:scale={{ duration: 140, start: 0.96 }}
            style="transform-origin: top left;"
            role="listbox"
            aria-label={$t("projectList")}
          >
            <div class="px-1 pb-1">
              <input
                class="h-8 w-full rounded border border-line bg-surface px-2 text-xs text-ink
                       placeholder:text-ink-3 outline-none focus:border-accent"
                placeholder={$t("searchProject")}
                aria-label={$t("searchProject")}
                bind:value={projQuery}
                onkeydown={(e) => { if (e.key === "Escape") showProjMenu = false; }}
              />
            </div>
            <div class="max-h-64 overflow-y-auto py-0.5">
              {#each filteredProjects as p (p.id)}
                {@const st = projectStats(p)}
                <button
                  class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs transition-colors hover:bg-raised {p.id === currentId ? 'bg-accent/10' : ''}"
                  role="option"
                  aria-selected={p.id === currentId}
                  onclick={() => pickProject(p.id)}
                  title="{p.name} · {st.done}/{st.total}"
                >
                  <span class="min-w-0 flex-1 truncate {p.id === currentId ? 'font-semibold' : ''}">{p.name}</span>
                  <span class="tnum shrink-0 text-[11px] {st.failed > 0 ? 'text-danger' : 'text-ink-3'}">{st.done}/{st.total}</span>
                </button>
              {:else}
                <p class="px-2.5 py-2 text-xs text-ink-3">{$t("noProjectMatch")}</p>
              {/each}
            </div>
          </div>
        {/if}
      </div>
      <span class="tnum text-xs text-ink-3">{curSt ? curSt.total : 0} {$t("pages")}</span>
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

  {#if lastInvokeError}
    <p
      role="alert"
      class="flex items-center gap-2 border-b border-danger-soft bg-danger-soft px-5 py-1.5 text-xs text-danger"
    >
      <span class="font-mono">{lastInvokeError.command}</span>
      <span class="min-w-0 flex-1 truncate" title={lastInvokeError.error}>
        {lastInvokeError.error}
      </span>
      <Button variant="default" size="sm" onclick={() => (showDiagnostics = true)}>
        Detail
      </Button>
    </p>
  {/if}

  {#if error}
    <p role="alert" class="border-b border-warn-soft bg-warn-soft px-5 py-2 text-sm text-warn">{error}</p>
  {/if}
  {#if info}
    {#key info}
      <Snackbar message={info} onClose={() => (info = "")} />
    {/key}
  {/if}

  {#if !loaded}
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
          {@const st = projectStats(p)}
          {@const pct = Math.round(st.progress * 100)}
        <div class="relative w-36 shrink-0 self-center">
          <button
            class="block w-full overflow-hidden rounded-md border px-2.5 pt-1.5 pb-2.5 text-left transition-colors {p.id === currentId ? 'border-accent bg-accent/10' : 'border-line bg-surface hover:border-accent/60'}"
            onclick={() => openProject(p.id)}
            title="{p.name} · {st.done}/{st.total} · {st.failed} {$t('kanbanFailed')}"
          >
            <div class="truncate pr-5 text-xs font-semibold">{p.name}</div>
            <div class="tnum mt-0.5 text-[10px] {st.failed > 0 ? 'text-danger' : 'text-ink-3'}">{st.done}/{st.total} · {pct}%{st.failed > 0 ? ` · ${st.failed}` : ""}</div>
            <div class="absolute inset-x-0 bottom-0 h-0.5 bg-raised">
              <div class="h-full bg-accent" style="width: {pct}%"></div>
            </div>
          </button>
          <button
            class="absolute top-1 right-1 rounded bg-surface-2/90 px-1 py-0.5 text-[11px] leading-none text-ink-2 opacity-70 backdrop-blur-sm transition-opacity hover:bg-hover hover:text-ink hover:opacity-100 focus-visible:opacity-100"
            title={$t("resultsExport")}
            aria-label="{$t('resultsExport')}: {p.name}"
            onclick={(e) => { e.stopPropagation(); void openResults(p.id); }}
          >⧉</button>
        </div>
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
    <section class="flex flex-wrap items-center gap-1.5 border-b border-line px-5 py-2" aria-label="folders">
      <button
        class="rounded-md border px-2.5 py-1 text-xs {folderId === 'all' ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
        onclick={() => (folderId = "all")}
      >{$t("allFolders")} ({allPages(current).length})</button>
      {#if nested}
        {#each crumb as f, i (f.id)}
          <span class="text-ink-3" aria-hidden="true">›</span>
          <FolderChip
            projectId={current.id}
            folders={current.folders}
            folder={f}
            stats={subtreeStats(current, f.id)}
            active={i === crumb.length - 1}
            canDelete={current.folders.length > 1}
            {busy}
            onSelect={(id) => (folderId = id)}
            onDelete={(id) => void deleteFolder(id)}
            onRenamed={() => currentId && void refreshProject(currentId)}
            onError={(msg) => (error = msg)}
          />
        {/each}
        {#if childFolders.length > 0}
          <span class="mx-1 h-4 w-px bg-line" aria-hidden="true"></span>
          {#each childFolders as f (f.id)}
            <FolderChip
              projectId={current.id}
              folders={current.folders}
              folder={f}
              stats={subtreeStats(current, f.id)}
              active={false}
              dashed
              canDelete={false}
              {busy}
              onSelect={(id) => (folderId = id)}
              onDelete={(id) => void deleteFolder(id)}
              onRenamed={() => currentId && void refreshProject(currentId)}
              onError={(msg) => (error = msg)}
            />
          {/each}
        {/if}
      {:else}
        {#each current.folders as f (f.id)}
          <FolderChip
            projectId={current.id}
            folders={current.folders}
            folder={f}
            stats={subtreeStats(current, f.id)}
            active={folderId === f.id}
            canDelete={current.folders.length > 1}
            {busy}
            onSelect={(id) => (folderId = id)}
            onDelete={(id) => void deleteFolder(id)}
            onRenamed={() => currentId && void refreshProject(currentId)}
            onError={(msg) => (error = msg)}
          />
        {/each}
      {/if}
      <form class="ml-1 flex gap-1" onsubmit={(e) => { e.preventDefault(); void createFolder(); }}>
        <input
          class="h-7 w-32 rounded-md border border-line bg-surface-2 px-2 text-xs outline-none focus:border-accent"
          placeholder={$t("folderName")}
          aria-label={$t("folderName")}
          bind:value={folderName}
        />
        <Button type="submit" variant="default" size="sm" disabled={busy || !folderName.trim()} title={folderId === "all" ? $t("newFolder") : `${$t("newFolder")} → ${currentFolder?.name ?? ""}`}>{$t("newFolder")}</Button>
      </form>
    </section>
    <section class="flex flex-wrap items-center gap-1.5 px-5 pt-3" aria-label="kanban">
      {#each [["all", allPages(current).length, "bg-ink-3"], ["todo", kanbanGroups.todo.length, "bg-warn"], ["doing", kanbanGroups.doing.length, "bg-info"], ["done", kanbanGroups.done.length, "bg-success"], ["failed", kanbanGroups.failed.length, "bg-danger"]] as [key, n, dot]}
        <button
          class="tnum flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs {kanban === key ? 'border-accent bg-accent/10 font-semibold' : 'border-line hover:border-accent/60'}"
          onclick={() => (kanban = key as typeof kanban)}
          aria-pressed={kanban === key}
        >
          <span class="h-1.5 w-1.5 shrink-0 rounded-full {dot}" aria-hidden="true"></span>
          {key === "all" ? $t("allFolders") : $t(key === "todo" ? "kanbanTodo" : key === "doing" ? "kanbanDoing" : key === "done" ? "kanbanDone" : "kanbanFailed")} · {n}
        </button>
      {/each}
      {#if !selectMode}
        <Button
          variant="default"
          size="sm"
          onclick={() => (selectMode = true)}
          disabled={busy || visiblePages.length === 0}
        >{$t("selectMode")}</Button>
      {:else}
        <span class="mx-1 h-4 w-px bg-line" aria-hidden="true"></span>
        <Button
          variant="default"
          size="sm"
          onclick={selectAll}
          disabled={busy || visiblePages.length === 0 || visiblePages.every((p) => selectedIds.includes(p.id))}
        >{$t("selectAll")} ({visiblePages.length})</Button>
        {#if selectedIds.length === 0}
          <Button variant="ghost" size="sm" onclick={() => (selectMode = false)}>{$t("close")}</Button>
        {/if}
      {/if}
      <div class="ml-auto flex overflow-hidden rounded-md border border-line" role="tablist" aria-label="{$t('gridView')} / {$t('listView')}">
        <button
          class="h-7 px-3 text-xs font-medium transition-colors {pageView === 'grid' ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'}"
          role="tab"
          aria-selected={pageView === "grid"}
          title="{$t('gridView')} (G)"
          onclick={() => void setPageView("grid")}
        >⬢ {$t("gridView")}</button>
        <button
          class="h-7 border-l border-line px-3 text-xs font-medium transition-colors {pageView === 'list' ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'}"
          role="tab"
          aria-selected={pageView === "list"}
          title="{$t('listView')} (L)"
          onclick={() => void setPageView("list")}
        >☰ {$t("listView")}</button>
      </div>
      <div class="flex overflow-hidden rounded-md border border-line" role="group" aria-label={$t("sortBy")}>
        {#each [["name", $t("sortName")], ["status", $t("sortStatus")], ["bubbles", $t("sortBubbles")]] as [key, label]}
          <button
            class="h-7 px-2.5 text-xs font-medium transition-colors {sortKey === key ? 'bg-accent text-accent-fg' : 'bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink'} {key !== 'name' ? 'border-l border-line' : ''}"
            aria-pressed={sortKey === key}
          title={label}
            onclick={() => setSortKey(key as SortKey)}
          >{label}{sortKey === key ? (sortDir === 1 ? " ▲" : " ▼") : ""}</button>
        {/each}
      </div>
    </section>
    {#if selectedIds.length > 0}
      <div class="tnum mx-5 mt-3 flex items-center gap-2 rounded-md border border-line bg-surface-2 px-3 py-1.5 text-xs" role="status" transition:fly={{ y: -8, duration: 180 }}>
        <span class="font-semibold">{$t("selectedCount").replace("{n}", String(selectedIds.length))}</span>
        <Button variant="default" size="sm" onclick={() => (showMove = true)} disabled={busy}>{$t("moveTo")}</Button>
        <Button variant="ghost" size="sm" onclick={() => { selectedIds = []; selAnchor = null; selectMode = false; }}>{$t("clearSelection")}</Button>
      </div>
    {/if}
    {#if thumbTotal > 0 && thumbDone < thumbTotal}
      <p role="status" class="tnum border-b border-line bg-surface-2 px-5 py-1.5 text-xs text-ink-2">
        {$t("loadingPreviews").replace("{done}", String(thumbDone)).replace("{total}", String(thumbTotal))}
      </p>
    {:else if thumbFailed > 0}
      <p role="status" class="flex items-center gap-2 border-b border-line bg-surface-2 px-5 py-1.5 text-xs text-ink-2">
        <span class="tnum">{$t("loadingPreviews").replace("{done}", String(thumbDone)).replace("{total}", String(thumbTotal))} · {$t("previewsFailed").replace("{n}", String(thumbFailed))}</span>
        <Button variant="ghost" size="sm" onclick={retryThumbs}>{$t("retryThumbs")}</Button>
      </p>
    {/if}
    {#if pageView === "grid"}
      <PageGrid
        pages={orderedPages}
        {thumbs}
        {broken}
        {selectedIds}
        {selectMode}
        {dragging}
        onCardClick={(e, pg) => onCardClick(e, pg)}
        onCardPress={(pg) => onCardPress(pg)}
        onRelease={cancelHold}
        onToggleSelect={(id) => toggleSelect(id)}
        observeCard={observeCard}
      />
    {:else}
      <PageList
        pages={orderedPages}
        {thumbs}
        {broken}
        {selectedIds}
        {selectMode}
        onCardClick={(e, pg) => onCardClick(e, pg)}
        onOpen={(pg) => openEditor(pg)}
        onToggleSelect={(id) => toggleSelect(id)}
        observeCard={observeCard}
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

  {#if selectedPage && currentId}
    <div class="fixed inset-0 z-50 flex flex-col bg-bg">
      {#key selectedPage.id}
        <EditorPanel
          projectId={currentId}
          page={selectedPage}
          fullImageUrl={fullUrls[selectedPage.id] ?? thumbs[selectedPage.id] ?? ""}
          fallbackUrl={thumbs[selectedPage.id] ?? ""}
          openError={openError}
          {readingDir}
          onClose={closeEditor}
          onSaved={onSaved}
          onBatchOpts={(o) => (batchOpts = { providerId: o.providerId, ...o.opts, readingDirection: readingDir }) }
        />
      {/key}
    </div>
  {/if}
  {#if detectState && !detectHidden}
    <DetectProgress state={detectState} onClose={closeDetectOverlay} />
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

  {#if showMove && current && currentId}
    {@const cur = current}
    {@const cid = currentId}
    <MoveDialog
      project={cur}
      fromFolderId={folderId}
      pageIds={selectedIds}
      {busy}
      onClose={() => (showMove = false)}
      onError={(msg) => (error = msg)}
      onMoved={(res, destName) => {
        // Dialog tetap terbuka: tampilkan layar sukses, user yang menutup.
        void (async () => {
          const updated = await api.getProject(cid);
          projects = projects.map((p) => (p.id === cid ? updated : p));
          // Seleksi + thumbnail basi dibuang: halaman sudah pindah folder.
          const live = new Set(allPages(updated).map((pg) => pg.id));
          for (const id of Object.keys(thumbs)) if (!live.has(id)) delete thumbs[id];
          for (const id of Object.keys(fullUrls)) if (!live.has(id)) delete fullUrls[id];
          for (const id of Object.keys(broken)) if (!live.has(id)) delete broken[id];
          selectedIds = [];
          selAnchor = null;
          selectMode = false;
          thumbs = { ...thumbs };
          fullUrls = { ...fullUrls };
          broken = { ...broken };
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
      selectedIds={selectedIds}
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
