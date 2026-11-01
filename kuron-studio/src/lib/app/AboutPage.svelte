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
  import KuroMascot from "../ui/KuroMascot.svelte";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  type View = "main" | "privacy" | "terms" | "license";
  let view = $state<View>("main");

  const title = $derived(
    view === "main" ? $t("about")
    : view === "privacy" ? $t("aboutPrivacy")
    : view === "terms" ? $t("aboutTerms")
    : $t("license"),
  );

  const body = $derived(
    view === "privacy" ? privacyText
    : view === "terms" ? termsText
    : view === "license" ? licenseText
    : "",
  );
</script>

<Panel {title} {onClose} class="max-w-2xl">
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

    <p class="mt-5 rounded-card border border-line bg-surface-2/60 px-3 py-2 text-xs leading-relaxed text-ink-3">
      🤖 {$t("aboutCredits")}
    </p>
  {:else}
    <pre class="tnum max-h-80 overflow-auto rounded-card border border-line bg-surface-2/60 p-3 text-xs leading-relaxed whitespace-pre-wrap text-ink-2">{body}</pre>
  {/if}
</Panel>
