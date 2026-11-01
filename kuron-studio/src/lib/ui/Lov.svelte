<script lang="ts">
  import { t } from "../app/i18n";
  import Icon from "./Icon.svelte";
  /** Satu opsi dropdown. */
  export interface LovOption {
    value: string;
    label: string;
  }

  interface Props {
    label?: string;
    /** Teks bantu di bawah field. */
    hint?: string;
    error?: string;
    /** Nilai terpilih — satu arah; perubahan keluar lewat `onchange`. */
    value?: string;
    options: LovOption[];
    placeholder?: string;
    disabled?: boolean;
    class?: string;
    /** Dipanggil setiap user memilih opsi. */
    onchange?: (value: string) => void;
  }

  let {
    label,
    hint,
    error,
    value = "",
    options,
    placeholder = "— pilih —",
    disabled = false,
    class: cls = "",
    onchange,
  }: Props = $props();

  const uid = $props.id();
  const btnId = $derived(`lov-${uid}`);
  const listId = $derived(`lov-${uid}-list`);
  const searchId = $derived(`lov-${uid}-search`);

  let open = $state(false);
  let query = $state("");

  const q = $derived(query.trim().toLowerCase());
  const filtered = $derived(q ? options.filter((o) => o.label.toLowerCase().includes(q)) : options);
  const selected = $derived(options.find((o) => o.value === value));

  function pick(v: string) {
    open = false;
    query = "";
    onchange?.(v);
  }

  function closeOnFocusOut(e: FocusEvent) {
    if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node | null)) open = false;
  }
</script>

<div class={["flex flex-col gap-1.5", cls]}>
  {#if label}
    <span id={`${btnId}-label`} class="text-[11px] font-medium tracking-wide text-ink-2 uppercase">
      {label}
    </span>
  {/if}
  <div class="relative" onfocusout={closeOnFocusOut}>
    <button
      type="button"
      id={btnId}
      aria-labelledby={label ? `${btnId}-label` : undefined}
      aria-haspopup="listbox"
      aria-expanded={open}
      aria-controls={listId}
      disabled={disabled}
      onclick={() => (open = !open)}
      onkeydown={(e) => { if (e.key === "Escape") open = false; }}
      class={[
        "flex h-9 w-full items-center gap-2 rounded-md border bg-surface-2 px-3 text-sm text-ink",
        "outline-none transition-colors",
        "disabled:cursor-not-allowed disabled:opacity-50",
        error ? "border-danger" : "border-line hover:border-line-strong",
        "focus:border-accent focus:ring-2 focus:ring-accent/25",
      ]}
    >
      <span class="min-w-0 flex-1 truncate text-left {selected ? '' : 'text-ink-3'}">
        {selected?.label ?? placeholder}
      </span>
      <span class="shrink-0 text-ink-3" aria-hidden="true"><Icon name={open ? "chevron-up" : "chevron-down"} size="sm" /></span>
    </button>
    {#if open}
      <div
        id={listId}
        role="listbox"
        aria-labelledby={label ? `${btnId}-label` : btnId}
        class="absolute left-0 top-full z-50 mt-1 max-h-64 w-full overflow-y-auto rounded-md border
               border-line-strong bg-surface-2 py-1 shadow-lg"
      >
        {#if options.length > 5}
          <div class="px-1 pb-1">
            <input
              id={searchId}
              class="h-8 w-full rounded border border-line bg-surface px-2 text-xs text-ink
                     placeholder:text-ink-3 outline-none focus:border-accent"
              placeholder={$t("searchList")}
              aria-label={$t("searchList")}
              bind:value={query}
              onkeydown={(e) => { if (e.key === "Escape") open = false; }}
            />
          </div>
        {/if}
        {#each filtered as o (o.value)}
          <button
            type="button"
            role="option"
            data-value={o.value}
            aria-selected={o.value === value}
            onclick={() => pick(o.value)}
            class="flex w-full items-center gap-2 px-3 py-1.5 text-left text-sm transition-colors
                   hover:bg-raised {o.value === value ? 'bg-accent/10 font-semibold' : ''}"
          >
            <span class="min-w-0 flex-1 truncate">{o.label}</span>
          </button>
        {:else}
          <p class="px-3 py-2 text-xs text-ink-3">Tidak ada yang cocok.</p>
        {/each}
      </div>
    {/if}
  </div>
  {#if error}
    <p class="text-[11px] leading-relaxed text-danger">{error}</p>
  {:else if hint}
    <p class="text-[11px] leading-relaxed text-ink-3">{hint}</p>
  {/if}
</div>
