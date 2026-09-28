<script lang="ts">
  import Field from "../ui/Field.svelte";
  import Select from "../ui/Select.svelte";

  // Reproduksi persis alur ProviderSettings: `bind:value` + error dibersihkan
  // lewat `oninput`. Kalau binding putus, "Nama wajib diisi." muncul walau
  // input penuh teks (bug screenshot 2026-09-28).

  let name = $state("");
  let ptype = $state("");
  let error = $state("");
  let saved = $state<Record<string, string> | null>(null);

  const NAME_REQUIRED = "Nama wajib diisi.";

  function save() {
    error = "";
    if (!name.trim()) {
      error = NAME_REQUIRED;
      return;
    }
    saved = { name: name.trim(), providerType: ptype };
  }
</script>

<Field
  id="h-name"
  data-testid="name"
  label="Nama"
  bind:value={name}
  error={error}
  oninput={() => (error = "")}
/>

<Select label="Tipe" bind:value={ptype}>
  <option value="">— pilih —</option>
  <option value="openai">openai</option>
  <option value="anthropic">anthropic</option>
</Select>

<button data-testid="save" onclick={save}>Simpan</button>

<p data-testid="err">{error}</p>
<!-- `state` = $state apa adanya; `out` = hasil save. Dua-duanya dicek. -->
<p data-testid="state">{name}</p>
<p data-testid="out">{saved ? JSON.stringify(saved) : ""}</p>
