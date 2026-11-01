<!-- Header aplikasi (FE-3): logo + switcher + toolbar kanan.
  Presentasional: state baca dari props/store baca-saja
  (theme/lang/isMobile), aksi lewat callback. showImportMenu +
  drawerOpen lokal. Sempit (`narrow` dari viewport): bar ringkas +
  drawer slide-over; tombol desktop-only tetap gate `isMobile`. -->
<script lang="ts">
  import iconUrl from "../../../../icon.svg";
  import { isMobile } from "../../app/device";
  import { lang, t, theme, type Lang, type Theme } from "../../app/i18n";
  import Button from "../../ui/Button.svelte";
  import UpdateButton from "../UpdateButton.svelte";
  import ProjectSwitcher from "./ProjectSwitcher.svelte";
  import type { Project } from "../../contracts/types";
  import type { ReadingDirection } from "../../contracts/types";

  interface Props {
    current: Project | null;
    projects: Project[];
    busy: boolean;
    readingDir: ReadingDirection;
    lastInvokeError: { command: string; error: string } | null;
    brokenCount: number;
    canDetect: boolean;
    /** Viewport sempit (<768px): bar ringkas + drawer, bukan wrap. */
    narrow: boolean;
    onToggleReadingDir: () => void;
    onProviders: () => void;
    onBatch: () => void;
    onGlossary: () => void;
    onResults: () => void;
    onDetect: () => void;
    onImportFolder: () => void;
    onImportFiles: () => void;
    onClean: () => void;
    onDelete: () => void;
    onOpenDiagnostics: () => void;
    onAbout: () => void;
    onPickProject: (id: string) => void;
  }

  let {
    current,
    projects,
    busy,
    readingDir,
    lastInvokeError,
    brokenCount,
    canDetect,
    narrow,
    onToggleReadingDir,
    onProviders,
    onBatch,
    onGlossary,
    onResults,
    onDetect,
    onImportFolder,
    onImportFiles,
    onClean,
    onDelete,
    onOpenDiagnostics,
    onAbout,
    onPickProject,
  }: Props = $props();
  let showImportMenu = $state(false);
  let drawerOpen = $state(false);

  const cycleLang = () => {
    lang.update((v: Lang) => (v === "id" ? "en" : v === "en" ? "zh" : "id"));
  };
</script>

<svelte:window onkeydown={(e) => { if (e.key === "Escape") { showImportMenu = false; drawerOpen = false; } }} />

{#snippet barActions(inDrawer = false)}
  {#if inDrawer}
    <div class="flex gap-1.5" role="group" aria-label="Tampilan">
      <Button variant="ghost" size="sm" class="trio-btn min-h-[44px] flex-1 justify-center font-mono" onclick={onToggleReadingDir} title="Urutan baca bubble">
        {readingDir === "rtl" ? "RTL→" : "←LTR"}
      </Button>
      <Button variant="ghost" size="sm" class="trio-btn min-h-[44px] flex-1 justify-center" onclick={() => theme.update((v: Theme) => (v === "dark" ? "light" : "dark"))} title="Tema gelap/terang" aria-label="Tema gelap/terang">
        {$theme === "dark" ? "◐" : "◑"}
      </Button>
      <Button variant="ghost" size="sm" class="trio-btn min-h-[44px] flex-1 justify-center font-mono" onclick={cycleLang} title="Bahasa / Language / 语言" aria-label="Ganti bahasa">
        {$lang.toUpperCase()}
      </Button>
    </div>
  {:else}
  <Button variant="ghost" size="sm" class="font-mono" onclick={onToggleReadingDir} title="Urutan baca bubble">
    {readingDir === "rtl" ? "RTL→" : "←LTR"}
  </Button>
  <Button variant="ghost" size="icon" onclick={() => theme.update((v: Theme) => (v === "dark" ? "light" : "dark"))} title="Tema gelap/terang" aria-label="Tema gelap/terang">
    {$theme === "dark" ? "◐" : "◑"}
  </Button>
  <Button variant="ghost" size="icon" class="font-mono" onclick={cycleLang} title="Bahasa / Language / 语言" aria-label="Ganti bahasa">
    {$lang.toUpperCase()}
  </Button>

  {#if !isMobile}<div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>{/if}
  {/if}

  <Button variant="ghost" size="sm" onclick={onProviders} title="Providers (Ctrl/⌘+,)">{$t("providers")}</Button>
  {#if !isMobile}<UpdateButton />{/if}
  <Button
    variant="ghost"
    size="sm"
    onclick={onBatch}
    disabled={!current}
    title="Batch translate (Ctrl/⌘+B)"
  >{$t("batch")}</Button>
  <Button variant="ghost" size="sm" onclick={onGlossary} title="Glossary">{$t("glossary")}</Button>
  <Button
    variant="ghost"
    size="sm"
    onclick={onResults}
    disabled={!current}
    title="Hasil & Ekspor (Ctrl/⌘+E)"
  >{$t("resultsExport")}</Button>

  <div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>

  <Button
    variant="primary"
    size="sm"
    onclick={onDetect}
    disabled={!canDetect || busy}
    title="Detect semua (Ctrl/⌘+D)"
  >{$t("detectAll")}</Button>
  <div
    class="relative"
    onfocusout={(e) => { if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node | null)) showImportMenu = false; }}
  >
    <Button
      variant="default"
      size="sm"
      onclick={() => (showImportMenu = !showImportMenu)}
      onkeydown={(e) => { if (e.key === "Escape") showImportMenu = false; }}
      disabled={!current || busy}
      aria-haspopup="menu"
      aria-expanded={showImportMenu}
      title={$t("importAction") + " (Ctrl/⌘+O)"}
    >⬆ {$t("importAction")} ▾</Button>
    {#if showImportMenu}
      <div
        class="absolute top-full z-50 mt-1 w-44 overflow-hidden rounded-md border border-line-strong bg-surface-2 py-1"
        role="menu"
        aria-label={$t("importAction")}
      >
        {#if !isMobile}
          <button
            class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs transition-colors hover:bg-raised"
            role="menuitem"
            onclick={() => { showImportMenu = false; onImportFolder(); }}
            onkeydown={(e) => {
              if (e.key === "Escape") showImportMenu = false;
              if (e.key === "ArrowDown") (e.currentTarget.nextElementSibling as HTMLElement | null)?.focus();
            }}
          >📁 {$t("importFolder")}</button>
        {/if}
        <button
          class="flex w-full items-center gap-2 px-2.5 py-1.5 text-left text-xs transition-colors hover:bg-raised"
          role="menuitem"
          onclick={() => { showImportMenu = false; onImportFiles(); }}
          onkeydown={(e) => {
            if (e.key === "Escape") showImportMenu = false;
            // iOS: folder picker tidak ada (FolderPickerNotImplemented) —
            // opsi folder disembunyikan total, bukan disabled.
            if (e.key === "ArrowUp") (e.currentTarget.previousElementSibling as HTMLElement | null)?.focus();
          }}
        >🖼️ {$t("importFiles")}</button>
      </div>
    {/if}
  </div>

  {#if !isMobile}<div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>{/if}

  {#if !isMobile}<Button
    variant="default"
    size="sm"
    onclick={onClean}
    disabled={!current || busy}
    title={brokenCount > 0 ? `${brokenCount} halaman gagal preview` : $t("nothingToClean")}
  >
    {#if brokenCount > 0}<span class="tnum text-danger">{brokenCount}</span>{/if}
    {$t("cleanImages")}
  </Button>{/if}
  <Button
    variant="danger"
    size="sm"
    onclick={onDelete}
    disabled={!current}
  >{$t("deleteProject")}</Button>

  {#if !isMobile}<div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>{/if}

  {#if !isMobile}<Button
    variant={lastInvokeError ? "danger" : "ghost"}
    size="icon"
    onclick={onOpenDiagnostics}
    title={lastInvokeError
      ? `${lastInvokeError.command}: ${lastInvokeError.error}`
      : "Diagnostics & log"}
    aria-label="Diagnostics"
  >⚙</Button>{/if}
  <Button
    variant="ghost"
    size="sm"
    onclick={onAbout}
    title={$t("about")}
    aria-label={$t("about")}
  >ⓘ {$t("about")}</Button>
{/snippet}

<header class="sticky top-0 z-30 border-b border-line bg-surface/80 backdrop-blur-md">
  <div class="flex items-center gap-2 px-4 pt-2">
    <img src={iconUrl} alt="Kuron Studio" class="size-6 shrink-0" />
    <h1 class="font-display shrink-0 text-base font-semibold tracking-tight">Kuron Studio</h1>
    <p class="hidden text-[11px] text-ink-3 sm:block">{$t("appTagline")}</p>
  </div>
  <div class="flex flex-wrap items-center gap-2 px-4 py-2">
    {#if projects.length > 0}
      <ProjectSwitcher {current} {projects} onPick={onPickProject} />
    {/if}

    {#if narrow}
      <div class="ml-auto flex items-center gap-1.5">
        <Button variant="ghost" size="icon" onclick={() => (drawerOpen = true)} aria-label={$t("openMenu")} aria-haspopup="dialog">☰</Button>
        <Button
          variant="primary"
          size="sm"
          onclick={onDetect}
          disabled={!canDetect || busy}
        >{$t("detectAll")}</Button>
      </div>
    {:else}
      <div class="ml-auto flex min-w-0 flex-1 items-center gap-1.5 overflow-x-auto sm:flex-none sm:flex-wrap sm:overflow-visible">
        {@render barActions()}
      </div>
    {/if}
  </div>
</header>

{#if narrow && drawerOpen}
  <div class="fixed inset-0 z-50">
    <button class="absolute inset-0 h-full w-full cursor-default bg-black/50" aria-label={$t("closeMenu")} onclick={() => (drawerOpen = false)} tabindex="-1"></button>
    <div
      role="dialog"
      aria-modal="true"
      aria-label={$t("openMenu")}
      class="absolute left-0 top-0 flex h-full w-72 max-w-[85vw] flex-col gap-1 overflow-y-auto border-r border-line bg-surface p-3"
    >
      <div class="mb-1 flex items-center gap-2">
        <span class="font-display text-sm font-semibold tracking-tight">Kuron Studio</span>
        <Button variant="ghost" size="icon" class="ml-auto" aria-label={$t("close")} onclick={() => (drawerOpen = false)}>×</Button>
      </div>
      <div class="grid justify-items-stretch gap-1.5 [&_.h-5]:hidden [&_button]:min-h-[44px] [&_button]:w-full [&_button]:justify-start [&_button]:px-3 [&_.trio-btn]:w-auto [&_.trio-btn]:justify-center [&_.trio-btn]:px-2">
        {@render barActions(true)}
      </div>
    </div>
  </div>
{/if}
