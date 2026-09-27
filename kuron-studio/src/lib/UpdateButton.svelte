<script lang="ts">
  // M4-7: check GitHub Releases via updater plugin; graceful no-op when
  // pubkey placeholder belum diganti atau plugin tak tersedia (dev browser).
  let status = $state<"idle" | "checking" | "ready" | "done" | "error">("idle");
  let msg = $state("");

  async function check() {
    status = "checking"; msg = "";
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (!update) { status = "idle"; msg = "Sudah versi terbaru."; return; }
      status = "ready";
      msg = `v${update.version} tersedia.`;
      await update.downloadAndInstall();
      status = "done";
      msg = "Terinstal — restart untuk memakai versi baru.";
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (e) {
      status = "error";
      msg = String(e).slice(0, 200);
    }
  }
</script>

<button
  class="rounded bg-zinc-800 px-2 py-1 text-sm hover:bg-zinc-700 disabled:opacity-50"
  onclick={check} disabled={status === "checking"}
  title="Cek update dari GitHub Releases"
>↻</button>
{#if msg}<span class="text-xs text-zinc-400">{msg}</span>{/if}
