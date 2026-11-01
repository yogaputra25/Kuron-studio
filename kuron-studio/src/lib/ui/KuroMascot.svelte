<!-- Maskot Kuro (diadaptasi dari `prototype/kuro-a.svg` milik nhasixapp,
  satu brand "Kuron"). Dekoratif: `aria-hidden`, teks status ditulis
  terpisah oleh pemanggil. Animasi kedip + napas mati total bila
  `prefers-reduced-motion` (DESIGN). -->
<script lang="ts">
  interface Props {
    /** Diameter px. */
    size?: number;
    /** 0–100: tampilkan ring progres ala `kuro-splash.svg`. Tanpa prop = wajah saja. */
    progress?: number | null;
  }

  let { size = 48, progress = null }: Props = $props();

  const R = 92;
  const CIRC = 2 * Math.PI * R;
  const clamped = $derived(
    progress == null ? null : Math.min(100, Math.max(0, progress)),
  );
  const dash = $derived(clamped == null ? null : `${((clamped / 100) * CIRC).toFixed(1)} ${CIRC.toFixed(1)}`);
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 200 200"
  aria-hidden="true"
  class="kuro shrink-0"
>
  {#if dash}
    <circle cx="100" cy="100" r={R} fill="none" stroke="var(--ks-accent)" stroke-opacity=".2" stroke-width="9" />
    <circle
      cx="100"
      cy="100"
      r={R}
      fill="none"
      stroke="var(--ks-accent)"
      stroke-width="9"
      stroke-linecap="round"
      stroke-dasharray={dash}
      transform="rotate(-90 100 100)"
    />
  {/if}
  <g class="kuro-breathe">
    <path d="M40 70 L52 28 L80 52 Z" fill="#1A1A1F" stroke="var(--ks-line-strong)" stroke-width="4" stroke-linejoin="round" />
    <path d="M160 70 L148 28 L120 52 Z" fill="#1A1A1F" stroke="var(--ks-line-strong)" stroke-width="4" stroke-linejoin="round" />
    <path d="M48 62 L54 40 L72 52 Z" fill="var(--ks-accent)" />
    <path d="M152 62 L146 40 L128 52 Z" fill="var(--ks-accent)" />
    <rect x="34" y="52" width="132" height="112" rx="58" fill="#1A1A1F" stroke="var(--ks-line-strong)" stroke-width="4" />
    <g class="kuro-eyes">
      <ellipse cx="72" cy="108" rx="20" ry="24" fill="#fff" />
      <ellipse cx="128" cy="108" rx="20" ry="24" fill="#fff" />
      <circle cx="75" cy="114" r="11" fill="#3A2B2B" />
      <circle cx="125" cy="114" r="11" fill="#3A2B2B" />
      <circle cx="78" cy="110" r="4" fill="#fff" />
      <circle cx="128" cy="110" r="4" fill="#fff" />
    </g>
    <ellipse cx="52" cy="130" rx="10" ry="6" fill="var(--ks-accent)" />
    <ellipse cx="148" cy="130" rx="10" ry="6" fill="var(--ks-accent)" />
    <path d="M94 132 q6 6 12 0" stroke="var(--ks-accent)" stroke-width="3" fill="none" stroke-linecap="round" />
    <path d="M160 40 l3 8 8 3 -8 3 -3 8 -3 -8 -8 -3 8 -3z" fill="#FFE66D" />
  </g>
</svg>

<style>
  .kuro-breathe {
    animation: kuro-breathe 3s ease-in-out infinite alternate;
  }
  .kuro-eyes {
    transform-box: fill-box;
    transform-origin: center;
    animation: kuro-blink 4.2s infinite;
  }
  @keyframes kuro-breathe {
    from { transform: translateY(0); }
    to { transform: translateY(-3px); }
  }
  @keyframes kuro-blink {
    0%, 93%, 100% { transform: scaleY(1); }
    96% { transform: scaleY(0.08); }
  }
  @media (prefers-reduced-motion: reduce) {
    .kuro-breathe, .kuro-eyes { animation: none; }
  }
</style>
