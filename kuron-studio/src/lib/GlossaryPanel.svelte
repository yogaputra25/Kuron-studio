<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./api";
  import { t } from "./i18n";
  import type { GlossaryEntry } from "./types";
  import Button from "./ui/Button.svelte";
  import Field from "./ui/Field.svelte";
  import Panel from "./ui/Panel.svelte";

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

<Panel title={$t("glossary")} {onClose} class="max-w-3xl">
  {#if error}
    <p role="alert" class="mb-4 rounded-md border border-warn-soft bg-warn-soft px-3 py-2 text-xs text-warn">
      {error}
    </p>
  {/if}

  <!-- Konten utama dan import CSV jadi 2 kolom: daftar entri butuh
       lebar untuk membaca source→target, sementara CSV cukup sempit. -->
  <div class="grid gap-6 lg:grid-cols-[1fr_18rem]">
    <div class="min-w-0">
      <div class="mb-2.5 flex items-baseline gap-2">
        <h3 class="font-display text-sm font-semibold text-ink">Entri</h3>
        <span class="tnum text-xs text-ink-3">{entries.length}</span>
      </div>

      {#if entries.length === 0}
        <div class="rounded-lg border border-dashed border-line px-4 py-10 text-center">
          <p class="text-sm text-ink-2">Belum ada entri.</p>
          <p class="mx-auto mt-1.5 max-w-xs text-xs leading-relaxed text-ink-3">
            Tambah pasangan source → target di bawah, atau import CSV untuk isi banyak sekaligus.
          </p>
        </div>
      {:else}
        <ul class="max-h-72 space-y-1.5 overflow-y-auto pr-1">
          {#each entries as e (e.id)}
            <li
              class="group flex items-center gap-3 rounded-md border border-line bg-surface-2 px-3 py-2
                     transition-colors hover:border-line-strong"
            >
              <span class="max-w-40 shrink-0 truncate text-sm font-medium text-ink" title={e.source}>
                {e.source}
              </span>
              <span class="text-ink-3" aria-hidden="true">→</span>
              <span class="min-w-0 flex-1 truncate text-sm text-ink-2" title={e.target}>
                {e.target}
              </span>
              <span class="flex shrink-0 gap-1 opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100">
                <Button
                  variant="default"
                  size="sm"
                  onclick={() => startEdit(e)}
                  aria-label="{$t('edit')} entri {e.source}"
                >{$t("edit")}</Button>
                <Button
                  variant="danger"
                  size="sm"
                  onclick={() => remove(e.id)}
                  aria-label="{$t('remove')} entri {e.source}"
                >{$t("remove")}</Button>
              </span>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    <div class="min-w-0 space-y-5">
      <div class="rounded-lg border border-line bg-surface-2/60 p-4">
        <h3 class="font-display mb-3 text-sm font-semibold text-ink">
          {editId ? $t("edit") : $t("add")}
        </h3>
        <div class="space-y-3">
          <Field label="Source (asli)" bind:value={fSource} placeholder="Guild" />
          <Field label="Target (terjemahan)" bind:value={fTarget} placeholder="Serikat" />
        </div>
        <div class="mt-4 flex gap-2">
          {#if editId}
            <Button
              variant="ghost"
              size="sm"
              onclick={() => { editId = null; fSource = ""; fTarget = ""; }}
            >Batal</Button>
          {/if}
          <Button
            variant="primary"
            class="ml-auto"
            onclick={save}
            disabled={busy || !fSource.trim() || !fTarget.trim()}
          >{busy ? "…" : $t("save")}</Button>
        </div>
      </div>

      <div class="rounded-lg border border-line bg-surface-2/60 p-4">
        <h3 class="font-display mb-3 text-sm font-semibold text-ink">CSV</h3>
        <textarea
          aria-label="CSV glossary"
          class="w-full rounded-md border border-line bg-surface px-3 py-2.5 font-mono text-[12px]
                 leading-relaxed text-ink placeholder:text-ink-3 outline-none transition-colors
                 focus:border-accent focus:ring-2 focus:ring-accent/25"
          rows="4"
          bind:value={csvText}
          placeholder={"source,target\nGuild,Serikat"}
        ></textarea>
        {#if csvMsg}
          <p role="status" class="mt-2 text-[11px] leading-relaxed text-ink-2">{csvMsg}</p>
        {/if}
        <div class="mt-3 flex gap-2">
          <Button variant="default" size="sm" onclick={importCsv} disabled={busy || !csvText.trim()}>
            Import
          </Button>
          <Button
            variant="default"
            size="sm"
            class="ml-auto"
            onclick={exportCsv}
            disabled={busy || entries.length === 0}
          >{$t("export")}</Button>
        </div>
      </div>
    </div>
  </div>

  <!-- Auto-context cuma relevan kalau ada entri — tanpa isi, block ini
       cuma menambah tinggi panel dengan teks kosong. -->
  {#if preview}
    <details class="mt-6 rounded-lg border border-line bg-surface-2/60">
      <summary
        class="cursor-pointer list-none px-4 py-3 text-xs font-medium text-ink-2 select-none
               transition-colors hover:text-ink"
      >
        Auto-context untuk prompt ({preview.split("\n").length} baris)
      </summary>
      <pre
        class="max-h-56 overflow-auto whitespace-pre-wrap border-t border-line px-4 py-3
               font-mono text-[12px] leading-relaxed text-ink-2"
      >{preview}</pre>
    </details>
  {/if}
</Panel>
