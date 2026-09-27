<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { api } from "./api";
  import { statusBadgeClass } from "./status";
  import type { BatchOpts, ExportFormat, Project, QaIssue, TmHit } from "./types";

  interface Props {
    project: Project;
    opts: BatchOpts | null;
    onRefresh: () => void;
    onClose: () => void;
  }

  let { project, opts, onRefresh, onClose }: Props = $props();

  let busy = $state(false);
  let error = $state("");
  let info = $state("");
  // M5: tab tampilan + state QA/TM.
  let tab = $state<"pages" | "qa" | "tm">("pages");
  let issues = $state<QaIssue[]>([]);
  let tmQuery = $state("");
  let tmHits = $state<TmHit[]>([]);

  async function exportAs(format: ExportFormat) {
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
      const out = await api.exportProject(project.id, format, path);
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
      });
      onRefresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function loadQa() {
    busy = true; error = ""; info = "";
    try {
      issues = await api.qaCheck(project.id);
      info = issues.length === 0 ? "QA bersih — tidak ada masalah." : `${issues.length} masalah QA.`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function searchTm() {
    const q = tmQuery.trim();
    if (!q) { tmHits = []; return; }
    busy = true; error = "";
    try {
      tmHits = await api.tmSearch(q, 10);
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
      });
      onRefresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" role="dialog" aria-modal="true">
  <div class="max-h-[90vh] w-full max-w-3xl overflow-auto rounded-lg border border-zinc-800 bg-zinc-950 p-4 text-sm text-zinc-100">
    <div class="mb-3 flex items-center gap-2">
      <h2 class="font-bold">Review — {project.name} ({project.pages.length})</h2>
      <button class="ml-auto rounded bg-zinc-800 px-2 py-1 hover:bg-zinc-700" onclick={onClose}>Tutup</button>
    </div>

    {#if error}<p class="mb-2 rounded bg-amber-950 px-2 py-1 text-xs text-amber-200">{error}</p>{/if}
    {#if info}<p class="mb-2 rounded bg-emerald-950 px-2 py-1 text-xs text-emerald-200">{info}</p>{/if}

    <div class="mb-3 flex flex-wrap gap-2">
      <button class="rounded bg-zinc-800 px-3 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={() => exportAs("json")} disabled={busy}>Export JSON</button>
      <button class="rounded bg-zinc-800 px-3 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={() => exportAs("png")} disabled={busy}>Export PNG</button>
      <button class="rounded bg-zinc-800 px-3 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={() => exportAs("cbz")} disabled={busy}>Export CBZ</button>
      <button class="rounded bg-zinc-800 px-3 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={() => exportAs("psd")} disabled={busy}>Export PSD</button>
      <button class="rounded bg-zinc-800 px-3 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={shareZip} disabled={busy}>Share ZIP</button>
      <button class="ml-auto rounded bg-zinc-800 px-3 py-1 text-xs hover:bg-zinc-700" onclick={onRefresh}>Refresh</button>
    </div>

    <div class="mb-3 flex gap-1 text-xs">
      <button class={`rounded px-3 py-1 ${tab === "pages" ? "bg-emerald-700 text-white" : "bg-zinc-800 hover:bg-zinc-700"}`} onclick={() => (tab = "pages")}>Halaman</button>
      <button class={`rounded px-3 py-1 ${tab === "qa" ? "bg-emerald-700 text-white" : "bg-zinc-800 hover:bg-zinc-700"}`} onclick={() => { tab = "qa"; void loadQa(); }}>QA</button>
      <button class={`rounded px-3 py-1 ${tab === "tm" ? "bg-emerald-700 text-white" : "bg-zinc-800 hover:bg-zinc-700"}`} onclick={() => (tab = "tm")}>TM</button>
    </div>

    {#if tab === "qa"}
      <div class="mb-2 flex gap-2">
        <button class="rounded bg-zinc-800 px-3 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={loadQa} disabled={busy}>Cek ulang</button>
      </div>
      {#if issues.length === 0}
        <p class="py-4 text-center text-xs text-zinc-500">Belum ada hasil QA — klik “Cek ulang”.</p>
      {:else}
        <table class="w-full text-xs">
          <thead><tr class="text-left text-zinc-500"><th class="pb-1 pr-2">File</th><th class="pb-1 pr-2">Bubble</th><th class="pb-1 pr-2">Jenis</th><th class="pb-1">Detail</th></tr></thead>
          <tbody>
            {#each issues as is (is.pageFile + is.bubbleIndex + is.kind)}
              <tr class="border-t border-zinc-800">
                <td class="max-w-48 truncate py-1 pr-2">{is.pageFile}</td>
                <td class="pr-2 text-zinc-400">{is.bubbleIndex >= 0 ? `#${is.bubbleIndex + 1}` : "—"}</td>
                <td class="pr-2"><span class="rounded bg-amber-800 px-1.5 py-0.5 text-[10px] font-semibold text-amber-100">{is.kind}</span></td>
                <td class="text-zinc-400">{is.detail}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {:else if tab === "tm"}
      <div class="mb-2 flex gap-2">
        <input
          class="flex-1 rounded bg-zinc-800 px-3 py-1.5 text-xs outline-none focus:ring-1 focus:ring-emerald-500"
          placeholder="Cari terjemahan sebelumnya… (Enter)"
          bind:value={tmQuery}
          onkeydown={(e) => e.key === "Enter" && searchTm()}
        />
        <button class="rounded bg-zinc-800 px-3 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={searchTm} disabled={busy}>Cari</button>
      </div>
      {#if tmHits.length === 0}
        <p class="py-4 text-center text-xs text-zinc-500">Ketik lalu Enter — hasil lintas project muncul di sini.</p>
      {:else}
        <table class="w-full text-xs">
          <thead><tr class="text-left text-zinc-500"><th class="pb-1 pr-2">Project</th><th class="pb-1 pr-2">File</th><th class="pb-1 pr-2">Asli</th><th class="pb-1">Terjemahan</th></tr></thead>
          <tbody>
            {#each tmHits as h (h.projectName + h.pageFile + h.bubbleIndex)}
              <tr class="border-t border-zinc-800">
                <td class="max-w-32 truncate py-1 pr-2">{h.projectName}</td>
                <td class="max-w-32 truncate pr-2 text-zinc-400">{h.pageFile}</td>
                <td class="max-w-48 truncate pr-2" title={h.original}>{h.original}</td>
                <td class="max-w-48 truncate text-emerald-300" title={h.translated}>{h.translated}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {:else}

    <table class="w-full text-xs">
      <thead>
        <tr class="text-left text-zinc-500">
          <th class="pb-1 pr-2">File</th>
          <th class="pb-1 pr-2">Status</th>
          <th class="pb-1 pr-2">Bubble</th>
          <th class="pb-1 pr-2">Translated</th>
          <th class="pb-1">Aksi</th>
        </tr>
      </thead>
      <tbody>
        {#each project.pages as pg (pg.id)}
          {@const failed = pg.status === "failed" || (pg.translation && pg.translation.bubbles.some((b) => !b.translated))}
          <tr class={`border-t border-zinc-800 ${failed ? "bg-rose-950/40" : ""}`}>
            <td class="max-w-56 truncate py-1.5 pr-2" title={pg.path}>{pg.path.split(/[/\\]/).pop()}</td>
            <td class="pr-2"><span class={`rounded px-1.5 py-0.5 text-[10px] font-semibold text-white ${statusBadgeClass(pg.status)}`}>{pg.status}</span></td>
            <td class="pr-2 text-zinc-400">{pg.bubbles.length}</td>
            <td class="pr-2 text-zinc-400">{pg.translation?.bubbles.filter((b) => b.translated).length ?? 0}/{pg.translation?.bubbles.length ?? 0}</td>
            <td class="py-1.5">
              <button class="rounded bg-zinc-800 px-2 py-0.5 hover:bg-zinc-700 disabled:opacity-50" onclick={() => retryPage(pg.id)} disabled={busy}>Retry page</button>
            </td>
          </tr>
          {#if failed && pg.translation}
            {#each pg.translation.bubbles.filter((b) => !b.translated) as b (b.index)}
              <tr class="border-t border-rose-900/50 bg-rose-950/20 text-[11px]">
                <td class="py-1 pl-6 pr-2 text-rose-300">bubble #{b.index + 1}: kosong</td>
                <td class="pr-2" colspan="3"></td>
                <td class="py-1">
                  <button class="rounded bg-amber-800 px-2 py-0.5 text-amber-100 hover:bg-amber-700 disabled:opacity-50" onclick={() => retryBubble(pg.id, b.index)} disabled={busy}>Retry bubble</button>
                </td>
              </tr>
            {/each}
          {/if}
        {/each}
      </tbody>
    </table>
    {/if}
  </div>
</div>
