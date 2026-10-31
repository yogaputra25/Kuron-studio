<!-- Header aplikasi (FE-3): logo + switcher + toolbar kanan.
  Presentasional: state baca dari props/store baca-saja
  (theme/lang/isMobile), aksi lewat callback. showImportMenu lokal. -->
<script lang="ts">
  import iconUrl from "../../icon.svg";
  import { isMobile } from "../../app/device";
  import { lang, t, theme, type Lang, type Theme } from "../../app/i18n";
  import Button from "../../ui/Button.svelte";
  import UpdateButton from "../UpdateButton.svelte";
  import ProjectSwitcher from "./ProjectSwitcher.svelte";
  import type { BatchOpts, Project } from "../../contracts/types";
  import type { ReadingDirection } from "../../contracts/types";

  interface Props {
    current: Project | null;
    projects: Project[];
    busy: boolean;
    readingDir: ReadingDirection;
    batchOpts: BatchOpts | null;
    lastInvokeError: { command: string; error: string } | null;
    brokenCount: number;
    canDetect: boolean;
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
    onPickProject: (id: string) => void;
  }

  let {
    current,
    projects,
    busy,
    readingDir,
    batchOpts,
    lastInvokeError,
    brokenCount,
    canDetect,
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
    onPickProject,
  }: Props = $props();
  let showImportMenu = $state(false);

  const cycleLang = () => {
    lang.update((v: Lang) => (v === "id" ? "en" : v === "en" ? "zh" : "id"));
  };
</script>

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

    <div class="ml-auto flex min-w-0 flex-1 items-center gap-1.5 overflow-x-auto sm:flex-none sm:flex-wrap sm:overflow-visible">
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

      <Button variant="ghost" size="sm" onclick={onProviders}>{$t("providers")}</Button>
      {#if !isMobile}<UpdateButton />{/if}
      <Button
        variant="ghost"
        size="sm"
        onclick={onBatch}
        disabled={!current || !batchOpts?.providerId}
        title={batchOpts?.providerId ? "Batch translate" : "Buka satu halaman di editor + pilih provider dulu"}
      >{$t("batch")}</Button>
      <Button variant="ghost" size="sm" onclick={onGlossary}>{$t("glossary")}</Button>
      <Button
        variant="ghost"
        size="sm"
        onclick={onResults}
        disabled={!current}
      >{$t("resultsExport")}</Button>

      <div class="mx-1 hidden h-5 w-px bg-line sm:block"></div>

      <Button
        variant="primary"
        size="sm"
        onclick={onDetect}
        disabled={!canDetect || busy}
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
    </div>
  </div>
</header>
