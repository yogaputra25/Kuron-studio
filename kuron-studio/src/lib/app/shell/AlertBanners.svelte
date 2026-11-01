<!-- Banner error/info (FE-3): 3 banner App digabung jadi 1 komponen.
  Presentasional murni: data + aksi lewat props/callback dari App. -->
<script lang="ts">
  import Button from "../../ui/Button.svelte";
  import Snackbar, { type SnackbarTone } from "../../ui/Snackbar.svelte";

  interface Props {
    lastInvokeError: { command: string; error: string } | null;
    error: string;
    info: string;
    /** Toast hasil cek update (di luar <main> tak kena clip) — null = sembunyi. */
    updateToast?: { message: string; tone: SnackbarTone } | null;
    onOpenDiagnostics: () => void;
    onClearInfo: () => void;
    onClearUpdateToast?: () => void;
  }

  let {
    lastInvokeError,
    error,
    info,
    updateToast = null,
    onOpenDiagnostics,
    onClearInfo,
    onClearUpdateToast = () => {},
  }: Props = $props();
</script>

{#if lastInvokeError}
  <p
    role="alert"
    class="flex items-center gap-2 border-b border-danger-soft bg-danger-soft px-5 py-1.5 text-xs text-danger"
  >
    <span class="font-mono">{lastInvokeError.command}</span>
    <span class="min-w-0 flex-1 truncate" title={lastInvokeError.error}>
      {lastInvokeError.error}
    </span>
    <Button variant="default" size="sm" onclick={onOpenDiagnostics}>
      Detail
    </Button>
  </p>
{/if}

{#if error}
  <p role="alert" class="border-b border-warn-soft bg-warn-soft px-5 py-2 text-sm text-warn">{error}</p>
{/if}
{#if info}
  {#key info}
    <Snackbar message={info} onClose={onClearInfo} />
  {/key}
{/if}
{#if updateToast}
  <Snackbar message={updateToast.message} tone={updateToast.tone} onClose={onClearUpdateToast} />
{/if}
