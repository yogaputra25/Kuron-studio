<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { api } from "./api";
  import { t } from "./i18n";
  import type { BatchOpts, ExportFormat, Project, QaIssue, TmHit } from "./types";
  import Button from "./ui/Button.svelte";
  import Panel from "./ui/Panel.svelte";
  import StatusBadge from "./ui/StatusBadge.svelte";

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
  let qaLoaded = $state(false);
  /** Filter status di tab Halaman — default sembunyikan yang sudah beres. */
  let pageFilter = $state<"todo" | "all">("todo");

  /** Ringkasan progress: berapa halaman selesai vs belum, per status. */
  const stats = $derived.by(() => {
    const pages = project.pages;
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

  const visiblePages = $derived(
    pageFilter === "todo"
      ? project.pages.filter((p) => p.status !== "translated")
      : project.pages,
  );

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
      qaLoaded = true;
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

<Panel title={project.name} {onClose} class="max-w-5xl">
  <p class="-mt-1 mb-4 text-xs text-ink-3">
    {$t("review")} · <span class="tnum">{project.pages.length}</span> {$t("pages")}
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

  <!-- Tab di atas (lebih penting); export dipindah ke baris sekunder di
       bawah — sebelumnya 5 tombol export mendominasi layar sebelum user
       sempat melihat isi review. -->
  <div class="mb-4 flex overflow-hidden rounded-md border border-line" role="tablist" aria-label="Tab review">
    {#each [["pages", $t("pages")], ["qa", "QA"], ["tm", "TM"]] as [key, label] (key)}
      <button
        role="tab"
        aria-selected={tab === key}
        class={[
          "h-8 flex-1 border-l border-line text-xs font-medium transition-colors first:border-l-0",
          tab === key ? "bg-accent text-accent-fg" : "bg-surface-2 text-ink-2 hover:bg-hover hover:text-ink",
        ]}
        onclick={() => {
          tab = key as typeof tab;
          if (key === "qa") void loadQa();
        }}
      >{label}</button>
    {/each}
  </div>

  <!-- Export: baris sekunder di bawah tab + konten, bukan di atas. -->
  <div class="mb-4 flex flex-wrap items-center gap-1.5 border-t border-line pt-3">
    <span class="mr-1 text-[11px] tracking-wide text-ink-3 uppercase">{$t("export")}</span>
    <Button variant="default" size="sm" onclick={() => exportAs("json")} disabled={busy}>{$t("exportJson")}</Button>
    <Button variant="default" size="sm" onclick={() => exportAs("png")} disabled={busy}>{$t("exportPng")}</Button>
    <Button variant="default" size="sm" onclick={() => exportAs("cbz")} disabled={busy}>{$t("exportCbz")}</Button>
    <Button variant="default" size="sm" onclick={() => exportAs("psd")} disabled={busy}>PSD</Button>
    <Button variant="default" size="sm" onclick={shareZip} disabled={busy}>ZIP</Button>
    <Button variant="ghost" size="sm" class="ml-auto" onclick={onRefresh}>{$t("refresh")}</Button>
  </div>

  {#if tab === "qa"}
    <div class="mb-3 flex items-center gap-2">
      <Button variant="default" size="sm" onclick={loadQa} disabled={busy}>
        {qaLoaded ? "Cek ulang" : "Jalankan QA"}
      </Button>
    </div>
    {#if !qaLoaded}
      <p class="py-8 text-center text-xs text-ink-3">
        QA belum dijalankan. Cek untranslated, overflow, dan SFX leak per halaman.
      </p>
    {:else if issues.length === 0}
      <div class="rounded-lg border border-success-soft bg-success-soft px-4 py-8 text-center">
        <p class="text-sm font-medium text-success">QA bersih</p>
        <p class="mt-1 text-xs text-ink-3">Tidak ada masalah di {project.pages.length} halaman.</p>
      </div>
    {:else}
      <table class="w-full text-xs">
        <thead>
          <tr class="text-left text-ink-3">
            <th class="pb-1.5 pr-2 font-medium">File</th>
            <th class="pb-1.5 pr-2 font-medium">Bubble</th>
            <th class="pb-1.5 pr-2 font-medium">Jenis</th>
            <th class="pb-1.5 font-medium">Detail</th>
          </tr>
        </thead>
        <tbody>
          {#each issues as is (is.pageFile + is.bubbleIndex + is.kind)}
            <tr class="border-t border-line">
              <td class="max-w-48 truncate py-1.5 pr-2 text-ink-2">{is.pageFile}</td>
              <td class="tnum pr-2 text-ink-3">{is.bubbleIndex >= 0 ? `#${is.bubbleIndex + 1}` : "—"}</td>
              <td class="pr-2">
                <span class="rounded bg-warn-soft px-1.5 py-0.5 text-[10px] font-semibold text-warn">{is.kind}</span>
              </td>
              <td class="text-ink-3">{is.detail}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {:else if tab === "tm"}
    <form class="mb-2 flex gap-2" onsubmit={(e) => { e.preventDefault(); void searchTm(); }}>
      <input
        aria-label={$t("search")}
        class="h-8 min-w-0 flex-1 rounded-md border border-line bg-surface-2 px-3 text-xs text-ink
               placeholder:text-ink-3 outline-none transition-colors
               focus:border-accent focus:ring-2 focus:ring-accent/25"
        placeholder="Cari terjemahan sebelumnya… (Enter)"
        bind:value={tmQuery}
      />
      <Button type="submit" variant="default" size="sm" disabled={busy}>{$t("search")}</Button>
    </form>
    {#if tmHits.length === 0}
      <p class="py-6 text-center text-xs text-ink-3">Ketik lalu Enter — hasil lintas project muncul di sini.</p>
    {:else}
      <table class="w-full text-xs">
        <thead>
          <tr class="text-left text-ink-3">
            <th class="pb-1.5 pr-2 font-medium">Project</th>
            <th class="pb-1.5 pr-2 font-medium">File</th>
            <th class="pb-1.5 pr-2 font-medium">Asli</th>
            <th class="pb-1.5 font-medium">Terjemahan</th>
          </tr>
        </thead>
        <tbody>
          {#each tmHits as h (h.projectName + h.pageFile + h.bubbleIndex)}
            <tr class="border-t border-line">
              <td class="max-w-32 truncate py-1.5 pr-2 text-ink-2">{h.projectName}</td>
              <td class="max-w-32 truncate pr-2 text-ink-3">{h.pageFile}</td>
              <td class="max-w-48 truncate pr-2 text-ink-2" title={h.original}>{h.original}</td>
              <td class="max-w-48 truncate text-success" title={h.translated}>{h.translated}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  {:else}
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
          {#if project.pages.length === 0}Belum ada halaman{:else}Semua halaman selesai{/if}
        </p>
        <p class="mt-1 text-xs text-ink-3">
          {#if project.pages.length === 0}Import halaman dulu di grid utama.{:else}Coba tab QA untuk cek kualitas terjemahan.{/if}
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
  {/if}
</Panel>
