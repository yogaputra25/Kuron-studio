<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
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
  import { ARCHIVE_EXTS, IMAGE_EXTS } from "./lib/types";
  import Button from "./lib/ui/Button.svelte";
  import StatusBadge from "./lib/ui/StatusBadge.svelte";
  import DiagnosticsPanel from "./lib/DiagnosticsPanel.svelte";
  import { onInvoke } from "./lib/log";

  let projects = $state<Project[]>([]);
  let currentId = $state<string | null>(null);
  let name = $state("");
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
  const selectedPage = $derived(current?.pages.find((p) => p.id === selectedId) ?? null);
  /** Halaman yang preview-nya gagal → jumlah tombol "Bersihkan". */
  const brokenCount = $derived(Object.keys(broken).length);
  const fileName = (path: string) => path.split(/[/\\]/).pop() ?? path;

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
    const missing = sortPagesByName(proj.pages).filter((pg) => !thumbs[pg.id] && !broken[pg.id]);
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
      const res = await api.cleanPages(currentId);
      const updated = await api.getProject(currentId);
      projects = projects.map((p) => (p.id === currentId ? updated : p));
      // Halaman hilang dari store — buang juga thumbnail-nya.
      const live = new Set(updated.pages.map((pg) => pg.id));
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

  async function createProject() {
    const n = name.trim();
    if (!n) return;
    busy = true;
    try {
      const p = await api.createProject(n);
      name = "";
      projects = [...projects, p];
      currentId = p.id;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function doImport(paths: string[]) {
    if (!currentId || paths.length === 0) return;
    busy = true;
    try {
      const res = await api.importPages(currentId, paths);
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
        ? { ...p, pages: p.pages.map((pg) => (pg.id === updated.id ? updated : pg)) }
        : p,
    );
  }

  async function detectAll() {
    if (!current) return;
    busy = true;
    error = "";
    try {
      await api.detectBatch(current.pages.map((p) => p.id));
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
      filters: [
        { name: "Images", extensions: [...IMAGE_EXTS] },
        { name: "Archives", extensions: [...ARCHIVE_EXTS] },
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
  <header class="sticky top-0 z-30 flex flex-wrap items-center gap-3 border-b border-line bg-surface/80 px-5 py-2.5 backdrop-blur-md">
    <h1 class="font-display text-base font-semibold tracking-tight">Kuron Studio</h1>

    {#if projects.length > 0}
      <select
        class="tnum h-8 max-w-56 rounded-md border border-line bg-surface-2 px-2 text-sm text-ink
               outline-none transition-colors hover:border-line-strong focus:border-accent"
        bind:value={currentId}
        onchange={() => currentId && loadThumbs(currentId)}
      >
        {#each projects as p (p.id)}
          <option value={p.id}>{p.name} · {p.pages.length}</option>
        {/each}
      </select>
      <span class="tnum text-xs text-ink-3">{current?.pages.length ?? 0} halaman</span>
    {/if}

    <div class="ml-auto flex flex-wrap items-center gap-1.5">
      <Button variant="ghost" size="sm" class="font-mono" onclick={() => (readingDir = readingDir === "rtl" ? "ltr" : "rtl")} title="Urutan baca bubble">
        {readingDir === "rtl" ? "RTL→" : "←LTR"}
      </Button>
      <Button variant="ghost" size="icon" onclick={() => theme.update((v: Theme) => (v === "dark" ? "light" : "dark"))} title="Tema gelap/terang" aria-label="Tema gelap/terang">
        {$theme === "dark" ? "◐" : "◑"}
      </Button>
      <Button variant="ghost" size="icon" class="font-mono" onclick={cycleLang} title="Bahasa / Language / 语言" aria-label="Ganti bahasa">
        {$lang.toUpperCase()}
      </Button>

      <div class="mx-1 h-5 w-px bg-line"></div>

      <Button variant="ghost" size="sm" onclick={() => (showProviders = true)}>{$t("providers")}</Button>
      <UpdateButton />
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

      <div class="mx-1 h-5 w-px bg-line"></div>

      <Button
        variant="primary"
        size="sm"
        onclick={detectAll}
        disabled={!current || busy || (current?.pages.length ?? 0) === 0}
      >{$t("detectAll")}</Button>
      <Button variant="default" size="sm" onclick={importFolder} disabled={!current || busy}>{$t("importFolder")}</Button>
      <Button variant="default" size="sm" onclick={importFiles} disabled={!current || busy}>{$t("importFiles")}</Button>

      <div class="mx-1 h-5 w-px bg-line"></div>

      <Button

        variant="default"
        size="sm"
        onclick={cleanPages}
        disabled={!current || busy || brokenCount === 0}
        title={brokenCount > 0 ? `${brokenCount} halaman preview gagal` : $t("nothingToClean")}
      >
        {#if brokenCount > 0}<span class="tnum text-danger">{brokenCount}</span>{/if}
        {$t("cleanImages")}
      </Button>
      <Button
        variant="danger"
        size="sm"
        onclick={deleteProject}
        disabled={!current || busy}
        title={$t("confirmDeleteProject")}
      >{$t("deleteProject")}</Button>

      <div class="mx-1 h-5 w-px bg-line"></div>

      <Button
        variant={lastInvokeError ? "danger" : "ghost"}
        size="icon"
        onclick={() => (showDiagnostics = true)}
        title={lastInvokeError
          ? `${lastInvokeError.command}: ${lastInvokeError.error}`
          : "Diagnostics & log"}
        aria-label="Diagnostics"
      >⚙</Button>
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
    <section
      class="grid grid-cols-2 gap-3 p-5 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 xl:grid-cols-8"
      class:outline-2={dragging}
      class:outline-accent={dragging}
    >
      {#each sortPagesByName(current.pages) as pg (pg.id)}
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
    {#if current.pages.length === 0}
      <p class="mt-16 text-center text-sm text-ink-3">
        {#if dragging}Lepaskan file untuk import…{:else}{$t("emptyHint")}{/if}
      </p>
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
      pages={cur.pages}
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
