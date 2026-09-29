<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLSelectAttributes } from "svelte/elements";

  interface Props extends Omit<HTMLSelectAttributes, "class" | "id"> {
    label?: string;
    /** Teks bantu di bawah field. */
    hint?: Snippet | string;
    error?: string;
    id?: string;
    class?: string;
    /** Isi <option> / isi select. */
    children?: Snippet;
  }

  let {
    label,
    hint,
    error,
    class: cls = "",
    id,
    children,
    value = $bindable(),
    ...rest
  }: Props = $props();

  const uid = $props.id();
  const selectId = $derived(id ?? `sel-${uid}`);
  const describedBy = $derived(
    error ? `${selectId}-err` : hint ? `${selectId}-hint` : undefined,
  );
</script>

<div class={["flex flex-col gap-1.5", cls]}>
  {#if label}
    <label for={selectId} class="text-[11px] font-medium tracking-wide text-ink-2 uppercase">
      {label}
    </label>
  {/if}
  <select
    {...rest}
    bind:value
    id={selectId}
    aria-invalid={error ? "true" : undefined}
    aria-describedby={describedBy}
    class={[
      // Height + padding SAMA PERSIS dengan ui/Field.svelte supaya baris
      // form sejajar. Native select menggambar arrow-nya sendiri, jadi arrow
      // bawaan lebih rapi daripada custom selama tingginya sama.
      "h-9 w-full rounded-md border bg-surface-2 px-3 text-sm text-ink",
      "outline-none transition-colors",
      "disabled:cursor-not-allowed disabled:opacity-50",
      error ? "border-danger" : "border-line hover:border-line-strong",
      "focus:border-accent focus:ring-2 focus:ring-accent/25",
    ]}
  >
    {@render children?.()}
  </select>
  {#if error}
    <p id={`${selectId}-err`} class="text-[11px] leading-relaxed text-danger">{error}</p>
  {:else if hint}
    <p id={`${selectId}-hint`} class="text-[11px] leading-relaxed text-ink-3">
      {#if typeof hint === "string"}{hint}{:else}{@render hint()}{/if}
    </p>
  {/if}
</div>
