<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { isMobile } from "./lib/device";
  import { api } from "./lib/api";
  import UpdateButton from "./lib/UpdateButton.svelte";
  import BatchPanel from "./lib/BatchPanel.svelte";
  import EditorPanel from "./lib/EditorPanel.svelte";
  import GlossaryPanel from "./lib/GlossaryPanel.svelte";
  import ProviderSettings from "./lib/ProviderSettings.svelte";
  import ReviewGrid from "./lib/ReviewGrid.svelte";
  import { sortPagesByName } from "./lib/status";
  import { lang, t, theme } from "./lib/i18n";
  import type { Lang, Theme } from "./lib/i18n";
  import type { BatchOpts, Page, Project, ReadingDirection } from "./lib/types";
  import { ARCHIVE_EXTS, IMAGE_EXTS, PDF_EXTS } from "./lib/types";
  import { allPages, folderStats, groupKanban, projectStats } from "./lib/library";
  import Button from "./lib/ui/Button.svelte";
  import StatusBadge from "./lib/ui/StatusBadge.svelte";
  import DiagnosticsPanel from "./lib/DiagnosticsPanel.svelte";
  import { onInvoke } from "./lib/log";

  let projects = $state<Project[]>([]);
  let currentId = $state<string | null>(null);
  let name = $state("");
  let folderId = $state<string | "all">("all");
  let kanban = $state<"all" | "todo" | "doing" | "done" | "failed">("all");
  let folderName = $state("");
  let showProjMenu = $state(false);
  let projQuery = $state("");
  let menuName = $state("");
  let thumbs = $state<Record<string, string>>({});
  let fullUrls = $state<Record<string, string>>({});
  let selectedId = $state<string | null>(null);
  let readingDir = $state<ReadingDirection>("rtl");
  let error = $state("");
  let info = $state("");
  /** pageId → alasan preview gagal (file hilang / tidak bisa di-decode). */
  let broken = $state<Record<string, string>>({});
  let openError = $state("");
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
  const scopedPages = $derived(
    !current ? [] : currentFolder ? currentFolder.pages : allPages(current),
  );
  const kanbanGroups = $derived(groupKanban(scopedPages));
  const visiblePages = $derived(
    kanban === "all" ? [...kanbanGroups.todo, ...kanbanGroups.doing, ...kanbanGroups.done, ...kanbanGroups.failed] : kanbanGroups[kanban],
  );
  const selectedPage = $derived(allPages(current ?? { id: "", name: "", folders: [] } as Project).find((p) => p.id === selectedId) ?? null);
  /** Halaman yang preview-nya gagal → jumlah tombol "Bersihkan". */
  const brokenCount = $derived(Object.keys(broken).length);
  const fileName = (path: string) => path.split(/[/\\]/).pop() ?? path;
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
      return;
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

  // fix-preview-decode: `Promise.all` dulu — satu halaman gagal decode
  // bikin seluruh batch thumbnail ditolak dan tak ada yang tampil. Pakai
  // allSettled + skor per halaman supaya satu file rusak cuma jadi satu card
  // broken, dan sisanya tetap bisa dilihat.
  async function loadThumbs(projectId: string) {
    const proj = projects.find((p) => p.id === projectId);
    if (!proj) return;
    // Skip yang sudah broken: retry 179 file sidecar tiap ganti project
    // mahal. `cleanPages` yang membersihkan peta ini sebagai jalur recovery.
    const missing = sortPagesByName(allPages(proj)).filter((pg) => !thumbs[pg.id] && !broken[pg.id]);
    const settled = await Promise.allSettled(
      missing.map((pg) => api.getImagePreview(pg.path)),
    );
    const nextThumbs = { ...thumbs };
    const nextBroken = { ...broken };
    settled.forEach((r, i) => {
      const id = missing[i].id;
      if (r.status === "fulfilled") {
        nextThumbs[id] = r.value;
        delete nextBroken[id];
      } else {
        nextBroken[id] = String(r.reason);
      }
    });
    thumbs = nextThumbs;
    broken = nextBroken;
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
    if (!n) return;
    busy = true;
    error = "";
    try {
      const p = await api.createProject(n);
      name = "";
      menuName = "";
      showProjMenu = false;
      projQuery = "";
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

  async function createFolder() {
    if (!currentId) return;
    const n = folderName.trim();
    if (!n) return;
    busy = true;
    error = "";
    try {
      const f = await api.createFolder(currentId, n);
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
    if (!current) return;
    busy = true;
    error = "";
    try {
      await api.detectBatch(scopedPages.map((p) => p.id));
      const updated = await api.getProject(current.id);
      projects = projects.map((p) => (p.id === current.id ? updated : p));
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
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
  <header class="sticky top-0 z-30 flex flex-wrap items-center gap-2 border-b border-line bg-surface/80 px-4 py-2 backdrop-blur-md">
    <h1 class="font-display shrink-0 text-base font-semibold tracking-tight">Kuron Studio</h1>
    <div class="h-5 w-px shrink-0 bg-line" aria-hidden="true"></div>
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
            <div class="border-t border-line p-1">
              <form class="flex gap-1" onsubmit={(e) => { e.preventDefault(); void createProject(menuName); }}>
                <input
                  class="h-7 min-w-0 flex-1 rounded border border-line bg-surface px-2 text-xs text-ink
                         placeholder:text-ink-3 outline-none focus:border-accent"
                  placeholder={$t("projectName")}
                  aria-label={$t("projectName")}
                  bind:value={menuName}
                />
                <Button type="submit" variant="default" size="sm" disabled={busy || !menuName.trim()} title={$t("create")}>+</Button>
              </form>
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
      >{$t("review")}</Button>

      <div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>

      <Button
        variant="primary"
        size="sm"
        onclick={detectAll}
        disabled={!current || busy || scopedPages.length === 0}
      >{$t("detectAll")}</Button>
      {#if !isMobile}<Button variant="default" size="sm" onclick={importFolder} disabled={!current || busy}>{$t("importFolder")}</Button>{/if}
      <Button variant="default" size="sm" onclick={importFiles} disabled={!current || busy}>{$t("importFiles")}</Button>

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
  </header>

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
    <p role="status" class="border-b border-success-soft bg-success-soft px-5 py-2 text-sm text-success">{info}</p>
  {/if}

  {#if projects.length === 0}
    <section class="mx-auto mt-20 w-full max-w-sm rounded-card border border-line bg-surface p-6">
      <h2 class="font-display mb-1 text-sm font-semibold">{$t("newProject")}</h2>
      <p class="mb-4 text-xs text-ink-3">Satu project = satu judul. Halaman bisa diimpor kapan saja.</p>
      <form
        class="flex gap-2"
        onsubmit={(e) => { e.preventDefault(); void createProject(); }}
      >
        <input
          class="h-9 min-w-0 flex-1 rounded-md border border-line bg-surface-2 px-3 text-sm text-ink
                 placeholder:text-ink-3 outline-none transition-colors
                 focus:border-accent focus:ring-2 focus:ring-accent/25"
          placeholder={$t("projectName")}
          aria-label={$t("projectName")}
          bind:value={name}
        />
        <Button type="submit" variant="primary" disabled={busy || !name.trim()}>{$t("create")}</Button>
      </form>
    </section>
  {:else if current}
    {@const dashStats = projectStats(current)}
    {@const dashPct = Math.round(dashStats.progress * 100)}
    <section class="flex items-stretch gap-3 border-b border-line bg-surface/50 px-5 py-2.5" aria-label={$t("dashboard")}>
      <div class="w-60 shrink-0">
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
      <div class="w-px self-stretch bg-line" aria-hidden="true"></div>
      <div class="flex min-w-0 flex-1 items-stretch gap-1.5 overflow-x-auto">
        {#each projects as p (p.id)}
          {@const st = projectStats(p)}
          {@const pct = Math.round(st.progress * 100)}
          <button
            class="relative w-36 shrink-0 self-center overflow-hidden rounded-md border px-2.5 pt-1.5 pb-2.5 text-left transition-colors {p.id === currentId ? 'border-accent bg-accent/10' : 'border-line bg-surface hover:border-accent/60'}"
            onclick={() => openProject(p.id)}
            title="{p.name} · {st.done}/{st.total} · {st.failed} {$t('kanbanFailed')}"
          >
            <div class="truncate text-xs font-semibold">{p.name}</div>
            <div class="tnum mt-0.5 text-[10px] {st.failed > 0 ? 'text-danger' : 'text-ink-3'}">{st.done}/{st.total} · {pct}%{st.failed > 0 ? ` · ${st.failed}` : ""}</div>
            <div class="absolute inset-x-0 bottom-0 h-0.5 bg-raised">
              <div class="h-full bg-accent" style="width: {pct}%"></div>
            </div>
          </button>
        {/each}
      </div>
      <div class="w-px self-stretch bg-line" aria-hidden="true"></div>
      <form class="flex w-44 shrink-0 flex-col justify-center gap-1" onsubmit={(e) => { e.preventDefault(); void createProject(); }}>
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
      {#each current.folders as f (f.id)}
        {@const fs = folderStats(f)}
        <div class="flex items-center overflow-hidden rounded-md border {folderId === f.id ? 'border-accent bg-accent/10' : 'border-line'}">
          <button class="px-2.5 py-1 text-xs {folderId === f.id ? 'font-semibold' : ''}" onclick={() => (folderId = f.id)} title="{fs.done}/{fs.total}">
            {f.name} ({fs.done}/{fs.total}{fs.failed > 0 ? ` ⚠${fs.failed}` : ""})
          </button>
          {#if current.folders.length > 1}
            <button class="px-1.5 py-1 text-[11px] text-danger hover:bg-danger-soft" onclick={() => void deleteFolder(f.id)} title={$t("deleteFolder")} aria-label="{$t('deleteFolder')}: {f.name}">×</button>
          {/if}
        </div>
      {/each}
      <form class="ml-1 flex gap-1" onsubmit={(e) => { e.preventDefault(); void createFolder(); }}>
        <input
          class="h-7 w-32 rounded-md border border-line bg-surface-2 px-2 text-xs outline-none focus:border-accent"
          placeholder={$t("folderName")}
          aria-label={$t("folderName")}
          bind:value={folderName}
        />
        <Button type="submit" variant="default" size="sm" disabled={busy || !folderName.trim()}>{$t("newFolder")}</Button>
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
    </section>
    <section
      class="grid grid-cols-2 gap-3 p-5 sm:grid-cols-3 md:grid-cols-3 lg:grid-cols-6 xl:grid-cols-8"
      class:outline-2={dragging}
      class:outline-accent={dragging}
    >
      {#each sortPagesByName(visiblePages) as pg (pg.id)}
        <button
          class="group overflow-hidden rounded-lg border border-line bg-surface text-left transition-colors
                 hover:border-accent/60 focus-visible:border-accent
                 disabled:cursor-not-allowed disabled:opacity-60"
          onclick={() => openEditor(pg)}
          disabled={!!broken[pg.id]}
        >
          <div class="relative">
            {#if thumbs[pg.id]}
              <img
                src={thumbs[pg.id]}
                alt={fileName(pg.path)}
                class="aspect-[3/4] w-full bg-raised object-cover"
                loading="lazy"
              />
            {:else if broken[pg.id]}
              <div
                class="flex aspect-[3/4] w-full flex-col items-center justify-center gap-1 bg-danger-soft p-2 text-center"
                role="img"
                aria-label="Preview gagal"
              >
                <span class="text-base text-danger" aria-hidden="true">⚠</span>
                <span class="text-[10px] leading-tight text-danger" title={broken[pg.id]}>preview gagal</span>
              </div>
            {:else}
              <div class="aspect-[3/4] w-full animate-pulse bg-raised"></div>
            {/if}
            {#if (pg.bubbles?.length ?? 0) > 0}
              <span
                class="tnum absolute top-1.5 right-1.5 rounded bg-cyan-soft/90 px-1.5 py-0.5
                       text-[10px] font-semibold text-cyan backdrop-blur-sm"
                title="{pg.bubbles.length} bubble"
              >⬢{pg.bubbles.length}</span>
            {/if}
          </div>
          <div class="flex items-center gap-1.5 border-t border-line px-2 py-1.5">
            <span class="truncate text-[11px] text-ink-2 group-hover:text-ink" title={pg.path}>
              {fileName(pg.path)}
            </span>
            <StatusBadge status={pg.status} class="ml-auto" />
          </div>
        </button>
      {/each}
    </section>
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

  {#if showReview && current}
    {@const cur = current}
    <ReviewGrid
      project={cur}
      opts={batchOpts}
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
