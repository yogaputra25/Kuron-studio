<script lang="ts">
  import Field from "../ui/Field.svelte";
  import Lov from "../ui/Lov.svelte";

  // Reproduksi persis alur ProviderSettings: `value` + `onchange` + error
  // dibersihkan lewat `oninput`. Kalau wiring putus, "Nama wajib diisi."
  // muncul walau input penuh teks (bug screenshot 2026-09-28).

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

<Lov
  label="Tipe"
  value={ptype}
  onchange={(v) => (ptype = v)}
  options={[
    { value: "", label: "— pilih —" },
    { value: "openai", label: "openai" },
    { value: "anthropic", label: "anthropic" },
  ]}
/>

<button data-testid="save" onclick={save}>Simpan</button>

<p data-testid="err">{error}</p>
<!-- `state` = $state apa adanya; `out` = hasil save. Dua-duanya dicek. -->
<p data-testid="state">{name}</p>
<p data-testid="out">{saved ? JSON.stringify(saved) : ""}</p>
