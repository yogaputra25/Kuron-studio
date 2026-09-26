<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { api } from "./lib/api";
  import { sortPagesByName, statusBadgeClass } from "./lib/status";
  import type { Project } from "./lib/types";

  let projects = $state<Project[]>([]);
  let currentId = $state<string | null>(null);
  let name = $state("");
  let thumbs = $state<Record<string, string>>({});
  let error = $state("");
  let dragging = $state(false);
  let busy = $state(false);

  const current = $derived(projects.find((p) => p.id === currentId) ?? null);
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

<main class="min-h-screen bg-zinc-950 text-zinc-100">
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
        <figure class="overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900">
          {#if thumbs[pg.id]}
            <img src={thumbs[pg.id]} alt={fileName(pg.path)} class="aspect-[3/4] w-full object-cover" loading="lazy" />
          {:else}
            <div class="aspect-[3/4] w-full animate-pulse bg-zinc-800"></div>
          {/if}
          <figcaption class="flex items-center gap-2 px-2 py-1.5 text-xs">
            <span class="truncate" title={pg.path}>{fileName(pg.path)}</span>
            <span class={`ml-auto shrink-0 rounded px-1.5 py-0.5 text-[10px] font-semibold text-white ${statusBadgeClass(pg.status)}`}>{pg.status}</span>
          </figcaption>
        </figure>
      {/each}
    </section>
    {#if current.pages.length === 0}
      <p class="mt-16 text-center text-sm text-zinc-500">
        {#if dragging}Lepaskan file untuk import…{:else}Drag-drop folder/zip ke sini, atau pakai tombol Import. (M0: folder & files){/if}
      </p>
    {/if}
  {/if}
</main>
