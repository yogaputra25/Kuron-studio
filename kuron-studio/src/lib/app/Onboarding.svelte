<script lang="ts">
  import Button from "../ui/Button.svelte";
  import { t } from "./i18n";
  import { fly } from "svelte/transition";

  interface Props {
    busy: boolean;
    onCreate: (name: string) => void;
    onSample: () => void;
  }

  let { busy, onCreate, onSample }: Props = $props();

  /** choice = dua pintu (V4) · wizard = tur 4 langkah (V3). */
  let view = $state<"choice" | "wizard">("choice");
  let wstep = $state(0);
  let name = $state("");

  const sampleCta = $derived(busy ? $t("running") : `${$t("onboardOptSampleT")} →`);
  const createCta = $derived(busy ? $t("running") : $t("create"));
  const slides = [1, 2, 3, 4];
  const last = $derived(wstep === slides.length - 1);

  function create(e: SubmitEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    onCreate(name.trim());
  }

</script>

{#if view === "choice"}
  <section class="mx-auto mt-10 w-full max-w-xl px-5 sm:mt-16" aria-label={$t("onboardChoiceT")} in:fly={{ y: 16, duration: 250 }}>
    <div class="rounded-card border border-line bg-surface p-6 text-center sm:p-7">
      <p class="text-[11px] font-semibold tracking-wide text-accent uppercase">Kuron Studio</p>
      <h2 class="font-display mt-1 text-xl font-semibold tracking-tight">{$t("onboardChoiceT")}</h2>
      <p class="mx-auto mt-1 max-w-md text-xs leading-relaxed text-ink-2">{$t("onboardChoiceD")}</p>
      <svg class="mx-auto mt-4 block" width="280" height="104" viewBox="0 0 280 104" aria-hidden="true">
        <rect x="14" y="12" width="72" height="84" rx="6" style="fill:var(--ks-surface);stroke:var(--ks-line-strong)" transform="rotate(-6 50 54)" />
        <ellipse cx="50" cy="50" rx="19" ry="12" style="fill:var(--ks-bg);stroke:var(--ks-text-3)" transform="rotate(-6 50 54)" />
        <rect x="104" y="12" width="72" height="84" rx="6" style="fill:var(--ks-surface);stroke:var(--ks-accent)" stroke-width="2" />
        <ellipse cx="140" cy="50" rx="19" ry="12" style="fill:var(--ks-bg);stroke:var(--ks-accent)" stroke-width="2" />
        <circle cx="164" cy="38" r="10" style="fill:var(--ks-accent)" /><text x="164" y="42" font-size="10" text-anchor="middle" style="fill:var(--ks-accent-fg)">✓</text>
        <rect x="194" y="12" width="72" height="84" rx="6" style="fill:var(--ks-illust-solid)" transform="rotate(6 230 54)" />
        <text x="212" y="56" font-size="11" style="fill:var(--ks-illust-on-solid)">Anu…</text>
      </svg>
      <div class="mt-5 grid gap-3 text-left sm:grid-cols-2">
        <div class="flex flex-col gap-2 rounded-xl border border-line bg-surface p-4 transition-all duration-150 hover:-translate-y-0.5 hover:border-line-strong hover:shadow-lg">
          <b class="text-sm font-semibold">{$t("onboardOptEmptyT")}</b>
          <small class="flex-1 text-xs leading-relaxed text-ink-2">{$t("onboardOptEmptyD")}</small>
          <Button variant="default" size="sm" onclick={() => { view = "wizard"; wstep = 0; }} disabled={busy}>
            {$t("onboardOptEmptyT")} →
          </Button>
        </div>
        <div class="flex flex-col gap-2 rounded-xl border border-accent bg-surface p-4 shadow-[0_0_0_1px_var(--ks-accent)] transition-all duration-150 hover:-translate-y-0.5 hover:shadow-xl">
          <span class="self-start rounded-full bg-accent px-2 py-0.5 text-[10px] font-bold text-accent-fg">
            {$t("onboardRecommended")}
          </span>
          <b class="text-sm font-semibold">{$t("onboardOptSampleT")}</b>
          <small class="flex-1 text-xs leading-relaxed text-ink-2">{$t("onboardOptSampleD")}</small>
          <Button variant="primary" size="sm" onclick={onSample} disabled={busy}>
            {sampleCta}
          </Button>
        </div>
      </div>
    </div>
  </section>
{:else}
  <section class="mx-auto mt-10 w-full max-w-md px-5 sm:mt-16" aria-label={$t("onboardTitle")} in:fly={{ y: 16, duration: 250 }}>
    <div class="rounded-card border border-line bg-surface p-6">
      <p class="text-[11px] font-semibold tracking-wide text-accent uppercase">Kuron Studio · {$t("onboardWizardTag")}</p>
      {#key wstep}
      <h2 class="font-display mt-1 text-lg font-semibold tracking-tight">{$t(`onboardStep${wstep + 1}t`)}</h2>
      <p class="mt-1 text-xs leading-relaxed text-ink-2">{$t(`onboardStep${wstep + 1}d`)}</p>
      <div class="mt-4 flex h-36 items-center justify-center rounded-xl border border-line bg-surface-2" aria-hidden="true" in:fly={{ x: 14, duration: 180 }}>
        {#if wstep === 0}
          <svg width="190" height="112" viewBox="0 0 190 112">
            <rect x="50" y="6" width="90" height="100" rx="6" style="fill:var(--ks-surface);stroke:var(--ks-line-strong)" />
            <rect x="60" y="16" width="70" height="38" rx="3" style="fill:var(--ks-raised)" />
            <rect x="60" y="60" width="70" height="28" rx="3" style="fill:var(--ks-raised)" />
            <ellipse cx="95" cy="35" rx="23" ry="13" style="fill:var(--ks-bg);stroke:var(--ks-text-3)" />
            <text x="150" y="62" font-size="20" style="fill:var(--ks-accent)">⤵</text>
          </svg>
        {:else if wstep === 1}
          <svg width="190" height="112" viewBox="0 0 190 112">
            <rect x="50" y="6" width="90" height="100" rx="6" style="fill:var(--ks-surface);stroke:var(--ks-line-strong)" />
            <ellipse cx="95" cy="50" rx="28" ry="17" style="fill:var(--ks-bg);stroke:var(--ks-accent)" stroke-width="3" />
            <circle cx="121" cy="36" r="12" style="fill:var(--ks-accent)" />
            <text x="121" y="41" font-size="12" text-anchor="middle" style="fill:var(--ks-accent-fg)">✓</text>
          </svg>
        {:else if wstep === 2}
          <svg width="210" height="112" viewBox="0 0 210 112">
            <rect x="8" y="26" width="80" height="60" rx="6" style="fill:var(--ks-illust-solid)" />
            <text x="20" y="62" font-size="13" style="fill:var(--ks-illust-on-solid-muted)">あの…</text>
            <text x="96" y="62" font-size="18" style="fill:var(--ks-accent)">→</text>
            <rect x="122" y="26" width="80" height="60" rx="6" style="fill:var(--ks-accent)" />
            <text x="134" y="62" font-size="13" font-weight="bold" style="fill:var(--ks-accent-fg)">Anu…</text>
          </svg>
        {:else}
          <svg width="190" height="112" viewBox="0 0 190 112">
            <rect x="55" y="10" width="80" height="58" rx="6" style="fill:var(--ks-surface);stroke:var(--ks-line-strong)" />
            <rect x="63" y="18" width="64" height="8" rx="4" style="fill:var(--ks-accent)" />
            <rect x="63" y="30" width="64" height="8" rx="4" style="fill:var(--ks-raised)" />
            <rect x="63" y="42" width="40" height="8" rx="4" style="fill:var(--ks-raised)" />
            <rect x="55" y="76" width="80" height="22" rx="6" style="fill:var(--ks-illust-solid)" />
            <text x="95" y="91" font-size="11" text-anchor="middle" style="fill:var(--ks-illust-on-solid)">Ekspor ✓</text>
          </svg>
        {/if}
      </div>
      {/key}
      <div class="mt-4 flex gap-1.5" role="tablist" aria-label="Langkah">
        {#each slides as s, i (s)}
          <button
            class="h-1.5 flex-1 rounded-full transition-colors {i === wstep ? 'bg-accent' : 'bg-raised hover:bg-line-strong'}"
            role="tab"
            aria-selected={i === wstep}
            aria-label={`Langkah ${s}`}
            onclick={() => (wstep = i)}
          ></button>
        {/each}
      </div>
      {#if last}
        <form class="mt-4 flex gap-2" onsubmit={create}>
          <input
            class="h-9 min-w-0 flex-1 rounded-md border border-line bg-surface-2 px-3 text-sm text-ink
                   placeholder:text-ink-3 outline-none transition-colors
                   focus:border-accent focus:ring-2 focus:ring-accent/25"
            placeholder={$t("projectName")}
            aria-label={$t("projectName")}
            bind:value={name}
          />
          <Button type="submit" variant="primary" disabled={busy || !name.trim()}>{createCta}</Button>
        </form>
      {/if}
      <div class="mt-4 flex items-center justify-between">
        <Button
          variant="ghost"
          size="sm"
          onclick={() => { if (wstep === 0) view = "choice"; else wstep -= 1; }}
        >← {$t("onboardBack")}</Button>
        {#if !last}
          <Button variant="primary" size="sm" onclick={() => (wstep += 1)}>{$t("onboardNext")} →</Button>
        {/if}
      </div>
    </div>
  </section>
{/if}
