<script lang="ts">
  import { api, type DiagnosticsInfo } from "./api";
  import { onInvoke, type InvokeMeta } from "./log";
  import Button from "./ui/Button.svelte";
  import Field from "./ui/Field.svelte";
  import Panel from "./ui/Panel.svelte";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  /** Maksimum baris di panel — cukup untuk melihat polanya, tidak membanjiri. */
  const KEEP = 300;

  let entries = $state<InvokeMeta[]>([]);
  let info = $state<DiagnosticsInfo | null>(null);
  let logText = $state("");
  let query = $state("");
  let onlyErrors = $state(false);
  let loaded = $state(false);

  // Berlangganan saat panel dibuka; berhenti saat ditutup supaya tidak
  // menahan komponen tetap hidup.
  $effect(() => {
    const off = onInvoke((m) => {
      entries = [m, ...entries].slice(0, KEEP);
    });
    void refresh();
    return off;
  });

  const filtered = $derived(
    entries.filter((e) => {
      if (onlyErrors && e.ok) return false;
      if (!query.trim()) return true;
      const q = query.trim().toLowerCase();
      return (
        e.command.toLowerCase().includes(q) ||
        (e.error ?? "").toLowerCase().includes(q)
      );
    }),
  );

  const errorCount = $derived(entries.filter((e) => !e.ok).length);

  async function refresh() {
    try {
      info = await api.diagnostics();
      logText = await api.readLog();
    } catch (e) {
      // Panel ini adalah alat debugging; kegagalan di sini tidak boleh
      // menjatuhkan app. Tampilkan apa adanya.
      logText = `Gagal membaca log: ${e}`;
    } finally {
      loaded = true;
    }
  }

  async function copyLog() {
    const text =
      `# kuron-studio ${info?.appVersion ?? "?"} · ${info?.os ?? "?"}/${info?.arch ?? "?"}\n` +
      `# log: ${info?.logActive ? "file" : "stderr saja"}\n\n` +
      logText;
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch (e) {
      copyError = String(e);
    }
  }

  let copied = $state(false);
  let copyError = $state("");

  function fmtTime(ms: number): string {
    const d = new Date(ms);
    return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}:${String(d.getSeconds()).padStart(2, "0")}`;
  }
</script>

<Panel title="Diagnostics" {onClose} class="max-w-4xl">
  <p class="-mt-1 mb-4 text-xs leading-relaxed text-ink-3">
    Semua panggilan Rust tercatat sebagai JSON Lines. Tempelkan isi ini saat
    membuat bug report — API key sudah disensor sebelum ditulis.
  </p>

  {#if info}
    <dl class="mb-4 grid grid-cols-2 gap-x-4 gap-y-2 rounded-md border border-line bg-surface-2/60 p-3 text-xs sm:grid-cols-4">
      <div>
        <dt class="text-ink-3">Versi</dt>
        <dd class="tnum font-medium text-ink">{info.appVersion}</dd>
      </div>
      <div>
        <dt class="text-ink-3">OS</dt>
        <dd class="font-medium text-ink">{info.os}</dd>
      </div>
      <div>
        <dt class="text-ink-3">Arsitektur</dt>
        <dd class="font-medium text-ink">{info.arch}</dd>
      </div>
      <div class="col-span-2 sm:col-span-1">
        <dt class="text-ink-3">File log</dt>
        <dd
          class="truncate font-medium"
          class:text-success={info.logActive}
          class:text-warn={!info.logActive}
          title={info.logActive ? info.dataDir : "hanya ke stderr"}
        >
          {info.logActive ? "aktif" : "stderr saja"}
        </dd>
      </div>
    </dl>
  {/if}

  <div class="mb-3 flex flex-wrap items-center gap-2">
    <Field
      class="min-w-48 flex-1"
      placeholder="Filter command atau error…"
      bind:value={query}
    />
    <Button
      variant={onlyErrors ? "primary" : "default"}
      size="sm"
      aria-pressed={onlyErrors}
      onclick={() => (onlyErrors = !onlyErrors)}
    >
      {errorCount > 0 ? `Error saja (${errorCount})` : "Error saja"}
    </Button>
    <Button variant="default" size="sm" onclick={refresh}>Refresh</Button>
    <Button variant="default" size="sm" onclick={copyLog} disabled={!logText}>
      {copied ? "Tersalin" : "Copy log"}
    </Button>
  </div>

  {#if copyError}
    <p role="alert" class="mb-2 text-[11px] text-danger">Copy gagal: {copyError}</p>
  {/if}

  <h3 class="font-display mb-2 text-sm font-semibold text-ink">
    Invoke sesi ini
    <span class="tnum ml-1 text-xs font-normal text-ink-3">
      {filtered.length} dari {entries.length}
    </span>
  </h3>

  {#if !loaded}
    <p class="py-4 text-center text-xs text-ink-3">Memuat…</p>
  {:else if entries.length === 0}
    <p class="rounded-md border border-dashed border-line px-4 py-6 text-center text-xs text-ink-3">
      Belum ada panggilan. Buka tab lain, lalu kembali ke sini.
    </p>
  {:else if filtered.length === 0}
    <p class="py-6 text-center text-xs text-ink-3">Tidak ada yang cocok dengan filter.</p>
  {:else}
    <ul class="mb-4 max-h-72 space-y-1 overflow-y-auto">
      {#each filtered as e (e.command + e.elapsedMs + String(e.error ?? ""))}
        <li
          class="flex items-start gap-2.5 rounded-md border px-2.5 py-1.5 text-[11px]
                 {e.ok
            ? 'border-line bg-surface-2/60'
            : 'border-danger-soft bg-danger-soft'}"
        >
          <span class="tnum shrink-0 text-ink-3">{fmtTime(performance.now() - e.elapsedMs)}</span>
          <span
            class="tnum shrink-0 rounded px-1.5 text-[10px] font-semibold
                   {e.ok ? 'bg-success-soft text-success' : 'bg-danger-soft text-danger'}"
          >{e.ok ? "ok" : "ERR"}</span>
          <span class="w-36 shrink-0 truncate font-mono text-ink" title={e.command}>
            {e.command}
          </span>
          <span class="tnum shrink-0 text-ink-3">{e.elapsedMs}ms</span>
          <span class="min-w-0 flex-1 truncate text-ink-3" title={e.error ?? JSON.stringify(e.result)}>
            {e.error ?? JSON.stringify(e.result)}
          </span>
        </li>
      {/each}
    </ul>
  {/if}

  <details class="rounded-md border border-line bg-surface-2/60">
    <summary
      class="cursor-pointer list-none px-3 py-2 text-xs font-medium text-ink-2 select-none
             transition-colors hover:text-ink"
    >
      Log file mentah (JSON Lines)
    </summary>
    <pre
      class="max-h-64 overflow-auto border-t border-line px-3 py-2.5 font-mono text-[10px] leading-relaxed text-ink-3"
    >{logText || "(kosong — cek izin tulis app data dir)"}</pre>
  </details>
</Panel>
