<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./api";
  import type { GlossaryEntry } from "./types";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let entries = $state<GlossaryEntry[]>([]);
  let error = $state("");
  let busy = $state(false);
  let fSource = $state("");
  let fTarget = $state("");
  let editId = $state<string | null>(null);
  let csvText = $state("");
  let csvMsg = $state("");
  let preview = $state<string | null>(null);

  async function load() {
    try {
      entries = await api.glossaryList();
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function refreshPreview() {
    try {
      preview = await api.glossaryContext([]);
    } catch {
      preview = null;
    }
  }

  async function save() {
    if (!fSource.trim() || !fTarget.trim()) { error = "Source + target wajib diisi."; return; }
    busy = true; error = "";
    try {
      if (editId) await api.glossaryUpdate(editId, fSource.trim(), fTarget.trim());
      else await api.glossaryAdd(fSource.trim(), fTarget.trim());
      fSource = ""; fTarget = ""; editId = null;
      await load(); await refreshPreview();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  function startEdit(e: GlossaryEntry) {
    editId = e.id; fSource = e.source; fTarget = e.target;
  }

  async function remove(id: string) {
    busy = true; error = "";
    try {
      await api.glossaryDelete(id);
      if (editId === id) { editId = null; fSource = ""; fTarget = ""; }
      await load(); await refreshPreview();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function importCsv() {
    if (!csvText.trim()) { csvMsg = "Tempel CSV dulu."; return; }
    busy = true; csvMsg = "";
    try {
      const n = await api.glossaryImportCsv(csvText);
      csvMsg = `${n} entri diimpor.`;
      csvText = "";
      await load(); await refreshPreview();
    } catch (e) {
      csvMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function exportCsv() {
    busy = true; csvMsg = "";
    try {
      csvText = await api.glossaryExportCsv();
      csvMsg = "CSV dimuat di bawah — salin manual.";
    } catch (e) {
      csvMsg = String(e);
    } finally {
      busy = false;
    }
  }

  onMount(async () => { await load(); await refreshPreview(); });
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" role="dialog" aria-modal="true">
  <div class="max-h-[90vh] w-full max-w-xl overflow-auto rounded-lg border border-zinc-800 bg-zinc-950 p-4 text-sm text-zinc-100">
    <div class="mb-3 flex items-center gap-2">
      <h2 class="font-bold">Glossary ({entries.length})</h2>
      <button class="ml-auto rounded bg-zinc-800 px-2 py-1 hover:bg-zinc-700" onclick={onClose}>Tutup</button>
    </div>

    {#if error}<p class="mb-2 rounded bg-amber-950 px-2 py-1 text-xs text-amber-200">{error}</p>{/if}

    <ul class="mb-3 max-h-56 space-y-1 overflow-auto">
      {#each entries as e (e.id)}
        <li class="flex items-center gap-2 rounded bg-zinc-900 px-2 py-1.5 text-xs">
          <span class="font-semibold">{e.source}</span>
          <span class="text-zinc-500">→</span>
          <span class="truncate text-zinc-300">{e.target}</span>
          <span class="ml-auto flex gap-1">
            <button class="rounded bg-zinc-800 px-2 py-0.5 hover:bg-zinc-700" onclick={() => startEdit(e)}>Edit</button>
            <button class="rounded bg-rose-900 px-2 py-0.5 text-rose-200 hover:bg-rose-800" onclick={() => remove(e.id)}>Hapus</button>
          </span>
        </li>
      {:else}
        <li class="text-xs text-zinc-500">Belum ada entri. Tambah di bawah atau import CSV.</li>
      {/each}
    </ul>

    <div class="mb-3 rounded border border-zinc-800 bg-zinc-900 p-3">
      <h3 class="mb-2 font-semibold">{editId ? "Edit entri" : "Entri baru"}</h3>
      <div class="grid grid-cols-2 gap-2">
        <label class="text-xs">Source (asli)
          <input class="mt-0.5 w-full rounded bg-zinc-800 px-2 py-1 outline-none focus:ring-1 focus:ring-emerald-500" bind:value={fSource} placeholder="Guild" />
        </label>
        <label class="text-xs">Target (terjemahan)
          <input class="mt-0.5 w-full rounded bg-zinc-800 px-2 py-1 outline-none focus:ring-1 focus:ring-emerald-500" bind:value={fTarget} placeholder="Serikat" />
        </label>
      </div>
      <div class="mt-2 flex gap-2">
        {#if editId}<button class="rounded bg-zinc-800 px-2 py-1 text-xs hover:bg-zinc-700" onclick={() => { editId = null; fSource = ""; fTarget = ""; }}>Batal</button>{/if}
        <button class="ml-auto rounded bg-emerald-600 px-3 py-1 text-xs font-semibold hover:bg-emerald-500 disabled:opacity-50" onclick={save} disabled={busy}>{busy ? "Simpan…" : "Simpan"}</button>
      </div>
    </div>

    <div class="mb-3 rounded border border-zinc-800 bg-zinc-900 p-3">
      <h3 class="mb-1 font-semibold">Import / Export CSV</h3>
      <textarea
        class="w-full rounded bg-zinc-800 px-2 py-1 font-mono text-[11px] outline-none focus:ring-1 focus:ring-emerald-500"
        rows="3" bind:value={csvText} placeholder={"source,target\nGuild,Serikat"}
      ></textarea>
      {#if csvMsg}<p class="mt-1 text-[11px] text-zinc-400">{csvMsg}</p>{/if}
      <div class="mt-2 flex gap-2">
        <button class="rounded bg-zinc-800 px-2 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={importCsv} disabled={busy}>Import</button>
        <button class="rounded bg-zinc-800 px-2 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={exportCsv} disabled={busy}>Export</button>
      </div>
    </div>

    <div class="rounded border border-zinc-800 bg-zinc-900 p-3">
      <h3 class="mb-1 font-semibold">Preview auto-context</h3>
      {#if preview}
        <pre class="whitespace-pre-wrap font-mono text-[11px] text-emerald-300">{preview}</pre>
      {:else}
        <p class="text-[11px] text-zinc-500">Kosong — tambah entri agar prompt translate otomatis diperkaya.</p>
      {/if}
    </div>
  </div>
</div>
