<!-- Halaman Tentang (About): info app + privasi + syarat. Dibuka dari
  header (desktop) / drawer (sempit). Versi dari package.json. -->
<script lang="ts">
  import { version } from "../../../package.json";
  import { t } from "./i18n";
  import Panel from "../ui/Panel.svelte";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
</script>

<Panel title={$t("about")} {onClose} class="max-w-2xl">
  <div class="flex items-center gap-3">
    <div class="flex size-11 shrink-0 items-center justify-center rounded-xl bg-accent-soft text-xl font-bold text-accent" aria-hidden="true">K</div>
    <div>
      <p class="font-display text-base font-semibold tracking-tight">Kuron Studio</p>
      <p class="tnum text-xs text-ink-3">{$t("version")} {version} · {$t("appTagline")}</p>
    </div>
  </div>

  <p class="mt-4 text-sm leading-relaxed text-ink-2">{$t("aboutIntro")}</p>

  <h3 class="mt-5 text-xs font-semibold uppercase tracking-wide text-ink-3">{$t("aboutFeatures")}</h3>
  <ul class="mt-1.5 list-disc space-y-1 pl-5 text-sm leading-relaxed text-ink-2">
    <li>{$t("aboutFeatLocal")}</li>
    <li>{$t("aboutFeatByok")}</li>
    <li>{$t("aboutFeatExport")}</li>
  </ul>

  <h3 class="mt-5 text-xs font-semibold uppercase tracking-wide text-ink-3">{$t("aboutPrivacy")}</h3>
  <ul class="mt-1.5 list-disc space-y-1 pl-5 text-sm leading-relaxed text-ink-2">
    <li>{$t("aboutPrivLocal")}</li>
    <li>{$t("aboutPrivKeys")}</li>
    <li>{$t("aboutPrivNoAccount")}</li>
  </ul>

  <h3 class="mt-5 text-xs font-semibold uppercase tracking-wide text-ink-3">{$t("aboutTerms")}</h3>
  <ul class="mt-1.5 list-disc space-y-1 pl-5 text-sm leading-relaxed text-ink-2">
    <li>{$t("aboutTermsContent")}</li>
    <li>{$t("aboutTermsQuota")}</li>
  </ul>
</Panel>
