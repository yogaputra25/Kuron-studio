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
  import { sortPagesByName, statusBadgeClass } from "./lib/status";
  import { lang, t, theme } from "./lib/i18n";
  import type { Lang, Theme } from "./lib/i18n";
  import type { BatchOpts, Page, Project, ReadingDirection } from "./lib/types";

  let projects = $state<Project[]>([]);
  let currentId = $state<string | null>(null);
  let name = $state("");
  let thumbs = $state<Record<string, string>>({});
  let fullUrls = $state<Record<string, string>>({});
  let selectedId = $state<string | null>(null);
  let readingDir = $state<ReadingDirection>("rtl");
  let error = $state("");
  let openError = $state("");
  let dragging = $state(false);
  let busy = $state(false);
  let showProviders = $state(false);
  let showBatch = $state(false);
  let showGlossary = $state(false);
  let showReview = $state(false);
  // Opts terakhir dari editor (provider+translate opts) — dipakai Batch/Review.
  let batchOpts = $state<BatchOpts | null>(null);

  async function refreshProject(id: string) {
    const updated = await api.getProject(id);
    projects = projects.map((p) => (p.id === id ? updated : p));
  }

  const current = $derived(projects.find((p) => p.id === currentId) ?? null);
  const selectedPage = $derived(current?.pages.find((p) => p.id === selectedId) ?? null);
  const fileName = (path: string) => path.split(/[/\\]/).pop() ?? path;

  async function refresh() {
    try {
      projects = await api.listProjects();
      if (!currentId && projects.length > 0) currentId = projects[0].id;
      if (currentId) await loadThumbs(currentId);
    } catch (e) {
      error = `Backend unreachable (jalankan via 'pnpm tauri dev'): ${e}`;
    }
  }

  // M4-6: apply persisted theme before first paint of the shell.
  $effect(() => {
    document.documentElement.dataset.theme = $theme;
  });
  const cycleLang = () => {
    lang.update((v: Lang) => (v === "id" ? "en" : v === "en" ? "zh" : "id"));
  };

  async function loadThumbs(projectId: string) {
    const proj = projects.find((p) => p.id === projectId);
    if (!proj) return;
    const missing = sortPagesByName(proj.pages).filter((pg) => !thumbs[pg.id]);
    const entries = await Promise.all(
      missing.map(async (pg) => [pg.id, await api.getImagePreview(pg.path)] as const),
    );
    for (const [id, url] of entries) thumbs[id] = url;
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
        { name: "Images", extensions: ["jpg", "jpeg", "png", "webp"] },
        { name: "Archives", extensions: ["zip", "cbz"] },
      ],
    });
    if (Array.isArray(files)) await doImport(files);
    else if (typeof files === "string") await doImport([files]);
  }

  onMount(async () => {
    await refresh();
    await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") dragging = true;
      else if (event.payload.type === "leave") dragging = false;
      else if (event.payload.type === "drop") {
        dragging = false;
        void doImport(event.payload.paths);
      }
    });
  });
</script>

<main class="min-h-screen bg-zinc-950 text-zinc-100 dark:bg-zinc-950 dark:text-zinc-100" data-theme={$theme}>
  <header class="flex items-center gap-4 border-b border-zinc-800 px-6 py-3">
    <h1 class="text-lg font-bold">Kuron Studio</h1>
    {#if projects.length > 0}
      <select
        class="rounded bg-zinc-800 px-2 py-1 text-sm"
        bind:value={currentId}
        onchange={() => currentId && loadThumbs(currentId)}
      >
        {#each projects as p (p.id)}
          <option value={p.id}>{p.name} ({p.pages.length})</option>
        {/each}
      </select>
    {/if}
    <div class="ml-auto flex gap-2">
      <button class="rounded bg-zinc-800 px-2 py-1 text-sm hover:bg-zinc-700" onclick={() => (readingDir = readingDir === "rtl" ? "ltr" : "rtl")} title="Urutan baca chip">
        {readingDir === "rtl" ? "RTL→" : "←LTR"}
      </button>
      <button class="rounded bg-zinc-800 px-2 py-1 text-sm hover:bg-zinc-700" onclick={() => theme.update((v: Theme) => (v === "dark" ? "light" : "dark"))} title="Tema gelap/terang">{$theme === "dark" ? "🌙" : "☀️"}</button>
      <button class="rounded bg-zinc-800 px-2 py-1 text-sm hover:bg-zinc-700" onclick={cycleLang} title="Bahasa / Language / 语言">{$lang.toUpperCase()}</button>
      <button class="rounded bg-zinc-800 px-3 py-1 text-sm hover:bg-zinc-700" onclick={() => (showProviders = true)}>{$t("providers")}</button>
      <UpdateButton />
      <button
        class="rounded bg-zinc-800 px-3 py-1 text-sm hover:bg-zinc-700 disabled:opacity-50"
        onclick={() => (showBatch = true)}
        disabled={!current || !batchOpts?.providerId}
        title={batchOpts?.providerId ? "Batch translate" : "Buka satu halaman di editor + pilih provider dulu"}
      >Batch</button>
      <button class="rounded bg-zinc-800 px-3 py-1 text-sm hover:bg-zinc-700" onclick={() => (showGlossary = true)}>Glossary</button>
      <button
        class="rounded bg-zinc-800 px-3 py-1 text-sm hover:bg-zinc-700 disabled:opacity-50"
        onclick={() => (showReview = true)}
        disabled={!current}
      >Review</button>
      <button class="rounded bg-sky-800 px-3 py-1 text-sm hover:bg-sky-700 disabled:opacity-50" onclick={detectAll} disabled={!current || busy || (current?.pages.length ?? 0) === 0}>Detect semua</button>
      <button class="rounded bg-zinc-800 px-3 py-1 text-sm hover:bg-zinc-700 disabled:opacity-50" onclick={importFolder} disabled={!current || busy}>Import folder</button>
      <button class="rounded bg-zinc-800 px-3 py-1 text-sm hover:bg-zinc-700 disabled:opacity-50" onclick={importFiles} disabled={!current || busy}>Import files</button>
    </div>
  </header>

  {#if error}
    <p class="border-b border-amber-900 bg-amber-950 px-6 py-2 text-sm text-amber-200">{error}</p>
  {/if}

  {#if projects.length === 0}
    <section class="mx-auto mt-16 max-w-md rounded-lg border border-zinc-800 bg-zinc-900 p-6">
      <h2 class="mb-2 font-semibold">Buat project pertama</h2>
      <div class="flex gap-2">
        <input
          class="flex-1 rounded bg-zinc-800 px-3 py-1.5 text-sm outline-none focus:ring-1 focus:ring-emerald-500"
          placeholder="Nama project…"
          bind:value={name}
          onkeydown={(e) => e.key === "Enter" && createProject()}
        />
        <button class="rounded bg-emerald-600 px-4 py-1.5 text-sm font-semibold hover:bg-emerald-500 disabled:opacity-50" onclick={createProject} disabled={busy}>Buat</button>
      </div>
    </section>
  {:else if current}
    <section
      class="grid grid-cols-2 gap-4 p-6 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6"
      class:ring-2={dragging}
      class:ring-emerald-500={dragging}
    >
      {#each sortPagesByName(current.pages) as pg (pg.id)}
        <button
          class="overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900 text-left hover:border-emerald-600"
          onclick={() => openEditor(pg)}
        >
          {#if thumbs[pg.id]}
            <img src={thumbs[pg.id]} alt={fileName(pg.path)} class="aspect-[3/4] w-full object-cover" loading="lazy" />
          {:else}
            <div class="aspect-[3/4] w-full animate-pulse bg-zinc-800"></div>
          {/if}
          <div class="flex items-center gap-2 px-2 py-1.5 text-xs">
            <span class="truncate" title={pg.path}>{fileName(pg.path)}</span>
            <span class={`ml-auto shrink-0 rounded px-1.5 py-0.5 text-[10px] font-semibold text-white ${statusBadgeClass(pg.status)}`}>{pg.status}</span>
            {#if (pg.bubbles?.length ?? 0) > 0}<span class="shrink-0 rounded bg-sky-700 px-1.5 py-0.5 text-[10px]">⬢{pg.bubbles.length}</span>{/if}
          </div>
        </button>
      {/each}
    </section>
    {#if current.pages.length === 0}
      <p class="mt-16 text-center text-sm text-zinc-500">
        {#if dragging}Lepaskan file untuk import…{:else}Drag-drop folder/zip ke sini, atau pakai tombol Import. (M0: folder & files){/if}
      </p>
    {/if}
  {/if}

  {#if selectedPage && currentId}
    <div class="fixed inset-0 z-50 flex flex-col bg-zinc-950">
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
</main>
