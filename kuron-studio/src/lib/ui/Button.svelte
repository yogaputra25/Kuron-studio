<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "primary" | "default" | "ghost" | "danger" | "quiet";
  type Size = "sm" | "md" | "lg" | "icon";

  interface Props extends Omit<HTMLButtonAttributes, "class"> {
    variant?: Variant;
    size?: Size;
    /** Stretch ke lebar container. */
    block?: boolean;
    /** Ikon/label di kiri — dipakai untuk tombol dengan glyph. */
    leading?: Snippet;
    trailing?: Snippet;
    class?: string;
    children?: Snippet;
  }

  let {
    variant = "default",
    size = "md",
    block = false,
    leading,
    trailing,
    class: cls = "",
    children,
    ...rest
  }: Props = $props();

  // Visual datang dari variant, bentuk dari size, sisanya utility caller.
  const VARIANT: Record<Variant, string> = {
    primary: "bg-accent text-accent-fg hover:bg-accent-hover active:brightness-95",
    default:
      "bg-raised text-ink border border-line hover:border-line-strong hover:bg-hover",
    ghost: "text-ink-2 hover:bg-hover hover:text-ink",
    danger: "bg-danger-soft text-danger border border-danger/40 hover:bg-danger hover:text-accent-fg",
    quiet: "text-ink-3 hover:text-ink hover:bg-hover",
  };

  const SIZE: Record<Size, string> = {
    sm: "h-7 px-2.5 text-xs gap-1.5",
    md: "h-8 px-3 text-sm gap-2",
    lg: "h-10 px-4 text-sm gap-2",
    icon: "h-8 w-8 p-0 gap-0",
  };
</script>

<button
  {...rest}
  class={[
    "inline-flex shrink-0 items-center justify-center rounded-md font-medium whitespace-nowrap",
    "transition-colors duration-100 select-none",
    "disabled:pointer-events-none disabled:opacity-40",
    VARIANT[variant],
    SIZE[size],
    block && "w-full",
    cls,
  ]}
>
  {#if leading}{@render leading()}{/if}
  {#if children}<span class="truncate">{@render children()}</span>{/if}
  {#if trailing}{@render trailing()}{/if}
</button>
