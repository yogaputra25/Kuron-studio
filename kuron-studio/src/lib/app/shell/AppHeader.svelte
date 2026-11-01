<!-- Header aplikasi (FE-3): logo + switcher + toolbar kanan.
  Presentasional: state baca dari props/store baca-saja
  (theme/lang/isMobile), aksi lewat callback. showImportMenu +
  drawerOpen lokal. Sempit (`narrow` dari viewport): bar ringkas +
  drawer slide-over; tombol desktop-only tetap gate `isMobile`. -->
<script lang="ts">
  import iconUrl from "../../../../icon.svg";
  import { isMobile } from "../../app/device";
  import Button from "../../ui/Button.svelte";
  import Icon from "../../ui/Icon.svelte";
  import { lang, t, theme, type Lang, type Theme } from "../../app/i18n";
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

{#snippet barActions()}
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
      class="absolute left-0 top-0 flex h-full w-80 max-w-[88vw] flex-col overflow-y-auto border-r border-line bg-surface"
    >
      <div class="flex items-center gap-2.5 border-b border-line px-4 py-3.5">
        <img src={iconUrl} alt="" aria-hidden="true" class="size-8 shrink-0" />
        <div class="min-w-0">
          <p class="font-display text-sm font-semibold tracking-tight">Kuron Studio</p>
          <p class="truncate text-[11px] text-ink-3">{$t("appTagline")}</p>
        </div>
        <Button variant="ghost" size="icon" class="ml-auto shrink-0" aria-label={$t("close")} onclick={() => (drawerOpen = false)}>×</Button>
      </div>

      <nav class="flex flex-1 flex-col gap-0.5 overflow-y-auto p-2" aria-label={$t("openMenu")}>
        <Button variant="primary" class="min-h-[48px] w-full" onclick={() => { drawerOpen = false; onDetect(); }} disabled={!canDetect || busy}>{$t("detectAll")}</Button>

        <div class="dsec" aria-hidden="true"></div>
        <button class="drow" onclick={() => { drawerOpen = false; onProviders(); }}><span aria-hidden="true" class="dic"><Icon name="key" /></span>{$t("providers")}<span aria-hidden="true" class="dchev">›</span></button>
        <button class="drow" onclick={() => { drawerOpen = false; onBatch(); }} disabled={!current}><span aria-hidden="true" class="dic"><Icon name="layers" /></span>{$t("batch")}</button>
        <button class="drow" onclick={() => { drawerOpen = false; onGlossary(); }}><span aria-hidden="true" class="dic"><Icon name="book" /></span>{$t("glossary")}<span aria-hidden="true" class="dchev">›</span></button>
        <button class="drow" onclick={() => { drawerOpen = false; onResults(); }} disabled={!current}><span aria-hidden="true" class="dic"><Icon name="download" /></span>{$t("resultsExport")}<span aria-hidden="true" class="dchev">›</span></button>

        <div class="dsec" aria-hidden="true"></div>
        <button class="drow" aria-expanded={showImportMenu} onclick={() => (showImportMenu = !showImportMenu)} disabled={!current || busy}><span aria-hidden="true" class="dic"><Icon name="upload" /></span>{$t("importAction")}<span aria-hidden="true" class="dchev">{showImportMenu ? "▾" : "›"}</span></button>
        {#if showImportMenu}
          {#if !isMobile}
            <button class="drow dsub" onclick={() => { drawerOpen = false; showImportMenu = false; onImportFolder(); }} disabled={!current || busy}><span aria-hidden="true" class="dic"><Icon name="folder" /></span>{$t("importFolder")}</button>
          {/if}
          <button class="drow dsub" onclick={() => { drawerOpen = false; showImportMenu = false; onImportFiles(); }} disabled={!current || busy}><span aria-hidden="true" class="dic"><Icon name="image" /></span>{$t("importFiles")}</button>
        {/if}
        {#if !isMobile}
          <button class="drow" onclick={() => { drawerOpen = false; onClean(); }} disabled={!current || busy}><span aria-hidden="true" class="dic"><Icon name="minus-circle" /></span>{$t("cleanImages")}{#if brokenCount > 0}<span class="tnum dvalDanger">{brokenCount}</span>{/if}</button>
        {/if}
        <button class="drow drowDanger" onclick={() => { drawerOpen = false; onDelete(); }} disabled={!current}><span aria-hidden="true" class="dic"><Icon name="trash" /></span>{$t("deleteProject")}</button>

        <div class="dsec" aria-hidden="true"></div>
        <button class="drow" onclick={() => onToggleReadingDir()}><span aria-hidden="true" class="dic"><Icon name="repeat" /></span>{$t("drawerReading")}<span class="tnum dval">{readingDir === "rtl" ? "RTL→" : "←LTR"}</span></button>
        <button class="drow" onclick={() => theme.update((v: Theme) => (v === "dark" ? "light" : "dark"))}><span aria-hidden="true" class="dic"><Icon name={$theme === "dark" ? "moon" : "sun"} /></span>{$t("drawerTheme")}<span class="dval">{$theme === "dark" ? $t("themeDark") : $t("themeLight")}</span></button>
        <button class="drow" onclick={cycleLang}><span aria-hidden="true" class="dic"><Icon name="globe" /></span>{$t("drawerLang")}<span class="tnum dval">{$lang.toUpperCase()}</span></button>
        {#if !isMobile}
          <button class="drow" onclick={() => { drawerOpen = false; onOpenDiagnostics(); }}><span aria-hidden="true" class="dic"><Icon name="activity" /></span>Diagnostics<span aria-hidden="true" class="dchev">›</span></button>
        {/if}
        <button class="drow" onclick={() => { drawerOpen = false; onAbout(); }}><span aria-hidden="true" class="dic"><Icon name="info" /></span>{$t("about")}<span aria-hidden="true" class="dchev">›</span></button>
      </nav>
    </div>
  </div>
{/if}

<style>
  .drow {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    min-height: 48px;
    padding: 0.5rem 0.75rem;
    border-radius: 0.625rem;
    font-size: 0.875rem;
    text-align: left;
    color: var(--ks-text);
    transition: background-color 120ms ease;
  }
  .drow:hover { background: var(--ks-raised); }
  .drow:active { background: var(--ks-hover); }
  .drow:disabled { opacity: 0.45; }
  .dic { width: 1.25rem; text-align: center; color: var(--ks-text-3); flex-shrink: 0; }
  .dval { margin-left: auto; color: var(--ks-text-3); font-size: 0.8rem; }
  .dvalDanger { margin-left: auto; color: var(--ks-danger); font-size: 0.8rem; font-weight: 600; }
  .dchev { margin-left: auto; color: var(--ks-text-3); }
  .dsec { height: 1px; background: var(--ks-line); margin: 0.5rem 0.25rem; }
  .dsub { padding-left: 2.5rem; }
  .drowDanger { color: var(--ks-danger); }
  .drowDanger .dic { color: var(--ks-danger); }
</style>
