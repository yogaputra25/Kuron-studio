<script lang="ts">
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";
  import type { SnackbarTone } from "../ui/Snackbar.svelte";
  import { t } from "./i18n";
  import { checkForUpdate, relaunchApp } from "./updater";
  // M4-7: check GitHub Releases via updater plugin; graceful no-op when
  // pubkey placeholder belum diganti atau plugin tak tersedia (dev browser).
  // Feedback TIDAK dirender di sini: hasil diteruskan ke pemilik (toast
  // global di AlertBanners) supaya tak kena clip `overflow-hidden` menu ⋯
  // dan tak menggeser layout bar.

  interface Props {
    /** Teks label di samping ikon (dipakai di menu ⋯); kosong = ikon saja. */
    label?: string | null;
    /** "drawer": baris .drow (gaya drawer, label wajib). Default: baris menu ⋯ / ikon. */
    kind?: "menu" | "drawer";
    /** Dipanggil saat diklik (pemilik menutup menu ⋯ / drawer). */
    onDone?: () => void;
    /** Hasil cek (sukses/error) untuk toast global pemilik. */
    onToast: (message: string, tone: SnackbarTone) => void;
  }

  let { label = null, kind = "menu", onDone, onToast }: Props = $props();
  let checking = $state(false);

  async function check() {
    if (checking) return;
    checking = true;
    try {
      const update = await checkForUpdate();
      if (!update) {
        onToast($t("updateLatest"), "success");
        return;
      }
      onToast($t("updateReady").replace("{version}", update.version), "success");
      await relaunchApp();
      onToast($t("updateDone"), "success");
    } catch (e) {
      onToast(String(e).slice(0, 200), "error");
    } finally {
      checking = false;
    }
  }
</script>

{#if kind === "drawer"}
  <button
    class="drow"
    onclick={() => { onDone?.(); void check(); }}
    disabled={checking}
  ><span aria-hidden="true" class="dic"><Icon name="refresh" /></span>{label}</button>
{:else if label}
  <button
    class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs transition-colors hover:bg-raised"
    role="menuitem"
    onclick={() => { onDone?.(); void check(); }}
    disabled={checking}
  ><span aria-hidden="true" class="contents text-ink-3"><Icon name="refresh" /></span>{label}</button>
{:else}
  <Button
    variant="ghost"
    size="icon"
    onclick={check}
    disabled={checking}
    title={$t("checkUpdate")}
    aria-label={$t("checkUpdate")}
  >↻</Button>
{/if}
