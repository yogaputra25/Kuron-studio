<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./api";
  import { t } from "./i18n";
  import { DEFAULT_BASE_URLS, PROVIDER_TYPES } from "./types";
  import type { AiModelOption, AiProviderType, ProviderView } from "./types";
  import Button from "./ui/Button.svelte";
  import Field from "./ui/Field.svelte";
  import Panel from "./ui/Panel.svelte";
  import Select from "./ui/Select.svelte";

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

  /**
   * Ganti tipe provider.
   *
   * Base URL hanya ditimpa kalau user belum mengetik apa pun. Sebelumnya
   * `!fBaseUrl` hampir selalu benar (default preset terisi sejak mount),
   * jadi mengetik `http://localhost:PORT/v1` DULU lalu memilih tipe akan
   * menimpa ketikan itu dengan preset — localhost ikut hilang.
   */
  function pickType(t: AiProviderType) {
    const wasPristine = fBaseUrl === DEFAULT_BASE_URLS[fType] || !fBaseUrl.trim();
    fType = t;
    if (t === "custom") {
      if (wasPristine) fBaseUrl = "";
    } else if (wasPristine || fType === t) {
      fBaseUrl = DEFAULT_BASE_URLS[t];
    }
    // Ganti tipe = model lama tidak berlaku -> jangan diam-diam kirim model
    // yang salah ke provider baru.
    if (fModel) { fModel = ""; models = []; modelsMsg = ""; }
    error = "";
  }

  const NAME_REQUIRED = "Nama wajib diisi.";

  // Base URL hanya relevan untuk provider custom — untuk preset lain Rust
  // menempelkan default-nya sendiri, jadi jangan beri hint yang noisy.
  const baseUrlHint = $derived(
    fType === "custom" ? "Wajib diisi untuk provider custom." : "Otomatis dari preset.",
  );

  // Pesan validasi per-field, bukan banner global. Banner "Nama wajib diisi."
  // dulu nempel di panel atas walau user sudah memperbaiki/menutup form.
  const nameError = $derived(error === NAME_REQUIRED ? "Wajib diisi." : "");

  /** Draft form siap-simpan? — dipakai untuk mengaktifkan Load models. */
  const canListModels = $derived(fKey.trim().length > 0 || !!editId);
  const modelsHint = $derived(
    canListModels
      ? ""
      : "Isi API key dulu — sebagian provider butuh key untuk melihat daftar model.",
  );

  // typedApiKey hanya untuk display: fKey sengaja tidak pernah di-backfill
  // dari provider yang tersimpan (Rust hanya menyimpan boolean hasKey).
  // Rust hanya menyimpan boolean hasKey, bukan key-nya, jadi `fKey` tidak
  // pernah di-backfill dari provider tersimpan. Penanda ini untuk menampilkan
  // "tersimpan" tanpa membocorkan isi key ke form.
  let hasStoredKey = $state(false);

  function startNew() {
    // Reset error juga — kalau tidak, "Nama wajib diisi." dari percobaan
    // sebelumnya masih nempel padahal form sudah kosong.
    editId = null; fName = ""; fKey = ""; fModel = "";
    models = []; modelsMsg = ""; validMsg = ""; validOk = null; error = "";
    hasStoredKey = false;
  }

  function startEdit(p: ProviderView) {
    editId = p.id; fType = p.providerType; fName = p.name; fBaseUrl = p.baseUrl;
    fKey = ""; fModel = p.model; models = []; modelsMsg = ""; validMsg = ""; validOk = null;
    hasStoredKey = p.hasKey;
    error = "";
  }

  /**
   * Ambil daftar model.
   *
   * Provider tersimpan -> pakai `list_models` (id). Provider baru -> pakai
   * `list_models_draft` dengan type+baseUrl+key yang SEDANG diketik, supaya
   * user tidak harus simpan dulu cuma untuk melihat pilihannya.
   */
  async function loadModels() {
    if (!canListModels) return;
    busy = true; modelsMsg = ""; models = [];
    try {
      models = editId
        ? await api.listModels(editId)
        : await api.listModelsDraft({ providerType: fType, baseUrl: fBaseUrl, apiKey: fKey.trim() });
      modelsMsg = models.length > 0 ? `${models.length} model.` : "Provider tidak mengembalikan model.";
      if (models.length > 0 && !fModel) fModel = models[0].id;
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
    if (!fName.trim()) { error = NAME_REQUIRED; return; }
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

<Panel title={$t("providers")} {onClose} class="max-w-2xl">
  {#if error}
    <p role="alert" class="mb-3 rounded-md border border-warn-soft bg-warn-soft px-2.5 py-1.5 text-xs text-warn">
      {error}
    </p>
  {/if}

  <ul class="mb-4 space-y-1">
    {#each providers as p (p.id)}
      <li class="flex items-center gap-2 rounded-md border border-line bg-surface-2 px-2 py-1.5 text-xs">
        <span class="font-semibold text-ink">{p.name}</span>
        <span class="text-ink-2">{typeLabel(p.providerType)}</span>
        <span class="truncate font-mono text-[11px] text-ink-3">{p.model}</span>
        <span
          class={[
            "rounded px-1.5 py-0.5 text-[10px] font-semibold",
            p.hasKey ? "bg-success-soft text-success" : "bg-danger-soft text-danger",
          ]}
        >{p.hasKey ? "key ✓" : "no key"}</span>
        {#if p.isVisionCapable}
          <span class="rounded bg-cyan-soft px-1.5 py-0.5 text-[10px] text-cyan" title="Vision capable">👁</span>
        {/if}
        <span class="ml-auto flex gap-1">
          <Button variant="default" size="sm" onclick={() => startEdit(p)}>{$t("edit")}</Button>
          <Button
            variant="danger"
            size="sm"
            onclick={() => remove(p.id)}
            aria-label="{$t('remove')} provider {p.name}"
          >{$t("remove")}</Button>
        </span>
      </li>
    {:else}
      <li class="py-4 text-center text-xs text-ink-3">Belum ada provider. Tambah di bawah.</li>
    {/each}
  </ul>

  <div class="rounded-lg border border-line bg-surface-2/60 p-5">
    <h3 class="font-display mb-4 text-sm font-semibold text-ink">
      {editId ? $t("edit") : $t("add")}
    </h3>

    <div class="grid grid-cols-2 gap-4">
      <Select
        label="Type"
        bind:value={fType}
        onchange={(e) => pickType((e.target as HTMLSelectElement).value as AiProviderType)}
      >
        {#each PROVIDER_TYPES as t (t.value)}<option value={t.value}>{t.label}</option>{/each}
      </Select>
      <Field
        label="Nama"
        bind:value={fName}
        placeholder="Gemini saya"
        error={nameError}
        oninput={() => (error = "")}
      />
      <Field
        label="Base URL"
        bind:value={fBaseUrl}
        class="col-span-2"
        hint={baseUrlHint}
        oninput={() => (error = "")}
      />
      <Field
        label="API key"
        type="password"
        autocomplete="off"
        bind:value={fKey}
        error=""
        hint={hasStoredKey
          ? "Ada key tersimpan. Kosongkan hanya kalau mau menggantinya."
          : "Tidak ditampilkan. Kosong = tidak berubah."}
        oninput={() => { error = ""; models = []; }}
      />
      <Field
        label="Model"
        bind:value={fModel}
        placeholder="gemini-2.0-flash"
        hint={fModel ? "" : "Ketik manual, atau tekan Load models."}
        oninput={() => (error = "")}
      />
    </div>

    {#if models.length > 0}
      <Select
        label="Model tersedia"
        class="mt-4"
        value={fModel}
        onchange={(e) => (fModel = (e.target as HTMLSelectElement).value)}
      >
        <option value="">— pilih dari daftar —</option>
        {#each models as m (m.id)}
          <option value={m.id}>{m.label}{m.vision ? " · vision" : ""}</option>
        {/each}
      </Select>
    {/if}

    {#if modelsMsg}
      <p role="status" class="mt-2 text-[11px] leading-relaxed text-ink-2">{modelsMsg}</p>
    {/if}
    {#if modelsHint}
      <p class="mt-2 text-[11px] leading-relaxed text-ink-3">{modelsHint}</p>
    {/if}
    {#if validMsg}
      <p role="status" class="mt-2 text-[11px] leading-relaxed {validOk ? 'text-success' : 'text-warn'}">
        {validMsg}
      </p>
    {/if}

    <div class="mt-5 flex flex-wrap items-center gap-2 border-t border-line pt-4">
      <Button
        variant="default"
        size="sm"
        onclick={loadModels}
        disabled={busy || !canListModels}
        title={canListModels ? "Ambil daftar model dari provider" : "Isi API key dulu"}
      >Load models</Button>
      <Button variant="default" size="sm" onclick={validate} disabled={busy || !editId}>
        Validate
      </Button>
      <Button variant="ghost" size="sm" class="ml-auto" onclick={startNew}>Baru</Button>
      <!-- Tidak di-`disabled` saat Nama kosong: user lalustitutions
           "kenapa tidak bisa disimpan" tanpa sebab. Tombol tetap aktif,
           save() yang memunculkan alasannya di field. -->
      <Button
        variant="primary"
        onclick={save}
        disabled={busy}
        title={fName.trim() ? "" : "Isi nama provider dulu."}
      >
        {busy ? "…" : $t("save")}
      </Button>
    </div>
  </div>
</Panel>
