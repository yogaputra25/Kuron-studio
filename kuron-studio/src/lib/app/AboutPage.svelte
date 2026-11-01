<!-- Halaman Tentang (About): info app + tombol khusus Privacy/Terms/License
  (sub-tampilan dalam modal, isi dari file .md repo via sync:legal) +
  kredit. Dibuka dari header (desktop) / drawer (sempit) / menu bar macOS.
  Versi dari package.json. Teks legal bahasa Indonesia (authoritatif). -->
<script lang="ts">
  import { version } from "../../../package.json";
  import licenseText from "./legal/LICENSE?raw";
  import privacyText from "./legal/PRIVACY.md?raw";
  import termsText from "./legal/TERMS.md?raw";
  import { t } from "./i18n";
  import Panel from "../ui/Panel.svelte";
  import Button from "../ui/Button.svelte";
  import Markdown from "../ui/Markdown.svelte";
  import KuroMascot from "../ui/KuroMascot.svelte";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  type View = "main" | "privacy" | "terms" | "license";
  let view = $state<View>("main");

  const body = $derived(
    view === "privacy" ? privacyText
    : view === "terms" ? termsText
    : view === "license" ? licenseText
    : "",
  );
</script>

<Panel title={view === "main" ? $t("about") : view === "privacy" ? $t("aboutPrivacy") : view === "terms" ? $t("aboutTerms") : $t("license")} {onClose} class="max-w-2xl">
  {#if view !== "main"}
    <div class="mb-3">
      <Button variant="ghost" size="sm" onclick={() => (view = "main")}>‹ {$t("onboardBack")}</Button>
    </div>
  {/if}

  {#if view === "main"}
    <div class="flex items-center gap-4 rounded-card border border-line bg-surface-2/60 px-4 py-3.5">
      <div class="shrink-0 rounded-2xl bg-accent-soft p-1.5">
        <KuroMascot size={56} />
      </div>
      <div class="min-w-0">
        <p class="font-display text-lg font-semibold tracking-tight">Kuron Studio</p>
        <p class="mt-1 flex flex-wrap items-center gap-1.5">
          <span class="tnum rounded-full border border-line bg-surface px-2 py-0.5 text-[11px] text-ink-2">v{version}</span>
          <span class="text-xs text-ink-3">{$t("appTagline")}</span>
        </p>
      </div>
    </div>

    <p class="mt-4 text-sm leading-relaxed text-ink-2">{$t("aboutIntro")}</p>

    <div class="mt-4 grid grid-cols-3 gap-2">
      <Button variant="default" size="sm" onclick={() => (view = "privacy")}>🔒 {$t("aboutPrivacy")}</Button>
      <Button variant="default" size="sm" onclick={() => (view = "terms")}>§ {$t("aboutTerms")}</Button>
      <Button variant="default" size="sm" onclick={() => (view = "license")}>© {$t("license")}</Button>
    </div>

    <h3 class="mb-1 mt-5 text-xs font-semibold uppercase tracking-wider text-ink-3">{$t("aboutFeatures")}</h3>
    <ul class="space-y-1.5 text-sm leading-relaxed text-ink-2">
      <li class="flex gap-2"><span class="text-success" aria-hidden="true">✓</span><span>{$t("aboutFeatLocal")}</span></li>
      <li class="flex gap-2"><span class="text-success" aria-hidden="true">✓</span><span>{$t("aboutFeatByok")}</span></li>
      <li class="flex gap-2"><span class="text-success" aria-hidden="true">✓</span><span>{$t("aboutFeatExport")}</span></li>
    </ul>

    <div class="mt-5 flex items-center gap-3 rounded-card border border-line bg-surface-2/60 px-3.5 py-2.5">
      <svg width="26" height="18" viewBox="0 0 36 24" fill="none" aria-hidden="true" class="shrink-0 text-info">
        <path d="M18 12C18 7.5 14.5 4 9.5 4 4.8 4 1.5 7.6 1.5 12c0 4.4 3.3 8 8 8 5 0 8.5-3.5 8.5-8 0-4.5 3.5-8 8.5-8 4.7 0 8 3.6 8 8 0 4.4-3.3 8-8 8-5 0-8.5-3.5-8.5-8Z" stroke="currentColor" stroke-width="3.2" stroke-linecap="round" />
      </svg>
      <p class="text-xs leading-relaxed text-ink-3">{$t("aboutCredits")}</p>
    </div>
  {:else}
    <div class="max-h-80 overflow-auto rounded-card border border-line bg-surface-2/60 py-1 pl-4 pr-3"><Markdown text={body} /></div>
  {/if}
</Panel>
