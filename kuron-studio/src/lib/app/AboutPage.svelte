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
  import Icon from "../ui/Icon.svelte";
  import Markdown from "../ui/Markdown.svelte";
  // Mark resmi Meta (Wikimedia Commons, merek dagang Meta) — hanya untuk
  // kredit atribusi "powered by".
  import metaUrl from "../../meta.svg";
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
      <Button variant="default" size="sm" onclick={() => (view = "privacy")}>{#snippet leading()}<Icon name="lock" />{/snippet}{$t("aboutPrivacy")}</Button>
      <Button variant="default" size="sm" onclick={() => (view = "terms")}>{#snippet leading()}<Icon name="file-text" />{/snippet}{$t("aboutTerms")}</Button>
      <Button variant="default" size="sm" onclick={() => (view = "license")}>{#snippet leading()}<Icon name="award" />{/snippet}{$t("license")}</Button>
    </div>

    <h3 class="mb-1 mt-5 text-xs font-semibold uppercase tracking-wider text-ink-3">{$t("aboutFeatures")}</h3>
    <ul class="space-y-1.5 text-sm leading-relaxed text-ink-2">
      <li class="flex gap-2"><span class="text-success" aria-hidden="true">✓</span><span>{$t("aboutFeatLocal")}</span></li>
      <li class="flex gap-2"><span class="text-success" aria-hidden="true">✓</span><span>{$t("aboutFeatByok")}</span></li>
      <li class="flex gap-2"><span class="text-success" aria-hidden="true">✓</span><span>{$t("aboutFeatExport")}</span></li>
    </ul>

    <h3 class="mb-1 mt-5 text-xs font-semibold uppercase tracking-wider text-ink-3">{$t("aboutTech")}</h3>
    <ul class="space-y-1.5 text-sm leading-relaxed text-ink-2">
      <li class="flex gap-2"><span class="shrink-0 text-ink-3"><Icon name="code" /></span><span>{$t("aboutTechFe")}</span></li>
      <li class="flex gap-2"><span class="shrink-0 text-ink-3"><Icon name="server" /></span><span>{$t("aboutTechBe")}</span></li>
      <li class="flex gap-2"><span class="shrink-0 text-ink-3"><Icon name="cpu" /></span><span>{$t("aboutTechAi")}</span></li>
    </ul>

    <div class="mt-5 rounded-card border border-accent/40 bg-accent-soft px-4 py-3.5">
      <div class="flex items-center gap-2.5">
        <img src={metaUrl} alt="" aria-hidden="true" width="30" height="20" class="shrink-0" />
        <p class="font-display text-sm font-semibold tracking-tight text-ink">{$t("aboutAiTitle")}</p>
      </div>
      <p class="mt-1.5 text-xs leading-relaxed text-ink-2">{$t("aboutAiDesc")}</p>
    </div>
  {:else}
    <div class="max-h-80 overflow-auto rounded-card border border-line bg-surface-2/60 py-1 pl-4 pr-3"><Markdown text={body} /></div>
  {/if}
</Panel>
