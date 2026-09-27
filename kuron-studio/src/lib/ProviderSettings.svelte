<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./api";
  import { DEFAULT_BASE_URLS, PROVIDER_TYPES } from "./types";
  import type { AiModelOption, AiProviderType, ProviderView } from "./types";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  let providers = $state<ProviderView[]>([]);
  let error = $state("");
  let busy = $state(false);

  // Form (new/edit): editId null = tambah baru.
  let editId = $state<string | null>(null);
  let fType = $state<AiProviderType>("gemini");
  let fName = $state("");
  let fBaseUrl = $state(DEFAULT_BASE_URLS.gemini);
  let fKey = $state("");
  let fModel = $state("");
  let models = $state<AiModelOption[]>([]);
  let modelsMsg = $state("");
  let validMsg = $state("");
  let validOk = $state<boolean | null>(null);

  const typeLabel = (t: string) => PROVIDER_TYPES.find((p) => p.value === t)?.label ?? t;

  async function load() {
    try {
      providers = await api.getProviders();
    } catch (e) {
      error = String(e);
    }
  }

  function pickType(t: AiProviderType) {
    fType = t;
    if (!fBaseUrl || t !== "custom") fBaseUrl = DEFAULT_BASE_URLS[t];
  }

  function startNew() {
    editId = null; fName = ""; fKey = ""; fModel = ""; models = []; modelsMsg = ""; validMsg = ""; validOk = null;
  }

  function startEdit(p: ProviderView) {
    editId = p.id; fType = p.providerType; fName = p.name; fBaseUrl = p.baseUrl;
    fKey = ""; fModel = p.model; models = []; modelsMsg = ""; validMsg = ""; validOk = null;
  }

  async function loadModels() {
    if (!editId) { modelsMsg = "Simpan provider dulu sebelum load models."; return; }
    busy = true; modelsMsg = "";
    try {
      models = await api.listModels(editId);
      modelsMsg = `${models.length} model.`;
    } catch (e) {
      modelsMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function validate() {
    if (!editId) { validMsg = "Simpan provider dulu sebelum validate."; validOk = false; return; }
    busy = true; validMsg = "";
    try {
      const r = await api.validateProvider(editId);
      validOk = r.ok; validMsg = r.message;
    } catch (e) {
      validOk = false; validMsg = String(e);
    } finally {
      busy = false;
    }
  }

  async function save() {
    if (!fName.trim()) { error = "Nama wajib diisi."; return; }
    busy = true; error = "";
    try {
      // ponytail: M2 simpan sqlite kolom "stored:key"; M4 pindah ke OS keychain.
      await api.saveProvider({
        ...(editId ? { id: editId } : {}),
        providerType: fType, name: fName.trim(), baseUrl: fBaseUrl,
        ...(fKey ? { apiKey: fKey } : {}), model: fModel,
      });
      fKey = "";
      startNew();
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove(id: string) {
    busy = true; error = "";
    try {
      await api.deleteProvider(id);
      if (editId === id) startNew();
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  onMount(load);
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" role="dialog" aria-modal="true">
  <div class="max-h-[90vh] w-full max-w-2xl overflow-auto rounded-lg border border-zinc-800 bg-zinc-950 p-4 text-sm text-zinc-100">
    <div class="mb-3 flex items-center gap-2">
      <h2 class="font-bold">Providers</h2>
      <button class="ml-auto rounded bg-zinc-800 px-2 py-1 hover:bg-zinc-700" onclick={onClose}>Tutup</button>
    </div>

    {#if error}<p class="mb-2 rounded bg-amber-950 px-2 py-1 text-xs text-amber-200">{error}</p>{/if}

    <ul class="mb-4 space-y-1">
      {#each providers as p (p.id)}
        <li class="flex items-center gap-2 rounded bg-zinc-900 px-2 py-1.5 text-xs">
          <span class="font-semibold">{p.name}</span>
          <span class="text-zinc-400">{typeLabel(p.providerType)}</span>
          <span class="truncate text-zinc-500">{p.model}</span>
          <span class={`rounded px-1.5 py-0.5 text-[10px] font-semibold ${p.hasKey ? "bg-emerald-700 text-white" : "bg-zinc-700 text-zinc-300"}`}>stored:{p.hasKey ? "key" : "missing"}</span>
          {#if p.isVisionCapable}<span class="rounded bg-sky-700 px-1.5 py-0.5 text-[10px] text-white">vision</span>{/if}
          <span class="ml-auto flex gap-1">
            <button class="rounded bg-zinc-800 px-2 py-0.5 hover:bg-zinc-700" onclick={() => startEdit(p)}>Edit</button>
            <button class="rounded bg-rose-900 px-2 py-0.5 text-rose-200 hover:bg-rose-800" onclick={() => remove(p.id)}>Hapus</button>
          </span>
        </li>
      {:else}
        <li class="text-xs text-zinc-500">Belum ada provider. Tambah di bawah.</li>
      {/each}
    </ul>

    <div class="rounded border border-zinc-800 bg-zinc-900 p-3">
      <h3 class="mb-2 font-semibold">{editId ? "Edit provider" : "Provider baru"}</h3>
      <div class="grid grid-cols-2 gap-2">
        <label class="text-xs">Type
          <select class="mt-0.5 w-full rounded bg-zinc-800 px-2 py-1" bind:value={fType} onchange={(e) => pickType((e.target as HTMLSelectElement).value as AiProviderType)}>
            {#each PROVIDER_TYPES as t (t.value)}<option value={t.value}>{t.label}</option>{/each}
          </select>
        </label>
        <label class="text-xs">Nama
          <input class="mt-0.5 w-full rounded bg-zinc-800 px-2 py-1 outline-none focus:ring-1 focus:ring-emerald-500" bind:value={fName} placeholder="Gemini saya" />
        </label>
        <label class="col-span-2 text-xs">Base URL
          <input class="mt-0.5 w-full rounded bg-zinc-800 px-2 py-1 outline-none focus:ring-1 focus:ring-emerald-500" bind:value={fBaseUrl} />
        </label>
        <label class="text-xs">API key (tidak ditampilkan; kosong = tak berubah)
          <input type="password" class="mt-0.5 w-full rounded bg-zinc-800 px-2 py-1 outline-none focus:ring-1 focus:ring-emerald-500" bind:value={fKey} autocomplete="off" />
        </label>
        <label class="text-xs">Model
          <input class="mt-0.5 w-full rounded bg-zinc-800 px-2 py-1 outline-none focus:ring-1 focus:ring-emerald-500" bind:value={fModel} placeholder="gemini-2.0-flash" />
        </label>
      </div>
      {#if models.length > 0}
        <select class="mt-2 w-full rounded bg-zinc-800 px-2 py-1 text-xs" onchange={(e) => (fModel = (e.target as HTMLSelectElement).value)}>
          <option value="">— pilih model —</option>
          {#each models as m (m.id)}<option value={m.id}>{m.label}{m.vision ? " 👁" : ""}</option>{/each}
        </select>
      {/if}
      {#if modelsMsg}<p class="mt-1 text-[11px] text-zinc-400">{modelsMsg}</p>{/if}
      {#if validMsg}<p class={`mt-1 text-[11px] ${validOk ? "text-emerald-300" : "text-amber-300"}`}>{validMsg}</p>{/if}
      <div class="mt-2 flex gap-2">
        <button class="rounded bg-zinc-800 px-2 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={loadModels} disabled={busy || !editId}>Load models</button>
        <button class="rounded bg-zinc-800 px-2 py-1 text-xs hover:bg-zinc-700 disabled:opacity-50" onclick={validate} disabled={busy || !editId}>Validate</button>
        <button class="ml-auto rounded bg-zinc-800 px-2 py-1 text-xs hover:bg-zinc-700" onclick={startNew}>Baru</button>
        <button class="rounded bg-emerald-600 px-3 py-1 text-xs font-semibold hover:bg-emerald-500 disabled:opacity-50" onclick={save} disabled={busy}>{busy ? "Simpan…" : "Simpan"}</button>
      </div>
    </div>
  </div>
</div>
