<script lang="ts">
  import Button from "../ui/Button.svelte";
  import { checkForUpdate, relaunchApp } from "./updater";
  // M4-7: check GitHub Releases via updater plugin; graceful no-op when
  // pubkey placeholder belum diganti atau plugin tak tersedia (dev browser).
  let status = $state<"idle" | "checking" | "ready" | "done" | "error">("idle");
  let msg = $state("");

  async function check() {
    status = "checking"; msg = "";
    try {
      const update = await checkForUpdate();
      if (!update) { status = "idle"; msg = "Sudah versi terbaru."; return; }
      status = "ready";
      msg = `v${update.version} tersedia.`;
      await relaunchApp();
      status = "done";
      msg = "Terinstal — restart untuk memakai versi baru.";
    } catch (e) {
      status = "error";
      msg = String(e).slice(0, 200);
    }
  }
</script>

<Button
  variant="ghost"
  size="icon"
  onclick={check}
  disabled={status === "checking"}
  title="Cek update dari GitHub Releases"
  aria-label="Cek update"
>↻</Button>
{#if msg}
  <span class="text-xs text-ink-2" role="status">{msg}</span>
{/if}
