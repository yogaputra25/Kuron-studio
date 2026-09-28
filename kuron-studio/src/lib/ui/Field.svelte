<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLInputAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLInputAttributes, "class" | "id"> {
    label?: string;
    id?: string;
    /** Teks bantu di bawah field. */
    hint?: Snippet | string;
    /** Pesan error — mengganti hint dan mewarnai border. */
    error?: string;
    class?: string;
  }

  let { label, hint, error, class: cls = "", id, value = $bindable(), ...rest }: Props = $props();

  const uid = $props.id();
  const inputId = $derived(id ?? `in-${uid}`);
  const describedBy = $derived(
    error ? `${inputId}-err` : hint ? `${inputId}-hint` : undefined,
  );
</script>

<div class={["flex flex-col gap-1.5", cls]}>
  {#if label}
    <label for={inputId} class="text-xs font-medium text-ink-2">{label}</label>
  {/if}
  <input
    {...rest}
    bind:value
    id={inputId}
    aria-invalid={error ? "true" : undefined}
    aria-describedby={describedBy}
    class={[
      "h-9 w-full rounded-md border bg-surface-2 px-3 text-sm text-ink",
      "placeholder:text-ink-3 outline-none transition-colors",
      "focus:border-accent focus:ring-2 focus:ring-accent/25",
      "disabled:cursor-not-allowed disabled:opacity-50",
      error ? "border-danger" : "border-line hover:border-line-strong",
    ]}
  />
  {#if error}
    <p id={`${inputId}-err`} class="text-xs text-danger">{error}</p>
  {:else if hint}
    <p id={`${inputId}-hint`} class="text-xs text-ink-3">
      {#if typeof hint === "string"}{hint}{:else}{@render hint()}{/if}
    </p>
  {/if}
</div>
