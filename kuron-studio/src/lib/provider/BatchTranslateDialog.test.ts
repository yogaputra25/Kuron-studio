import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("../..", import.meta.url));
const dlg = readFileSync(join(SRC, "lib/provider/BatchTranslateDialog.svelte"), "utf8");

describe("batch dialog: ganti provider reload model", () => {
  it("loadModelsFor memanggil api.listModels + default simpanan lalu vision pertama", () => {
    expect(dlg).toContain("listModels(pid)");
    expect(dlg).toContain("providers.find((p) => p.id === pid)?.model");
    expect(dlg).toContain("models.find((m) => m.vision)");
    expect(dlg).toContain("onProviderChange");
  });

  it("label provider memakai format nama (model)", () => {
    expect(dlg).toContain("`${p.name} (${p.model})`");
  });
});

describe("batch dialog: scope + skip → pageIds benar", () => {
  it("scope dihitung via allPages / subtreePages dari props", () => {
    expect(dlg).toContain("allPages(project)");
    expect(dlg).toContain("subtreePages(project, folderId)");
    expect(dlg).toContain("visiblePages");
    expect(dlg).toContain("selectedIds");
  });

  it("skip-done default on memfilter status translated", () => {
    expect(dlg).toContain("let skipDone = $state(true)");
    expect(dlg).toContain('p.status !== "translated"');
  });

  it("submit meng-emit onSubmit({providerId, opts}, pageIds)", () => {
    expect(dlg).toContain("onSubmit({ providerId, opts }, pageIds)");
  });

  it("dialog mengekspos onClose (Panel + Batal)", () => {
    expect(dlg).toContain("onClose");
  });
});

describe("batch dialog: Mulai disabled tanpa provider / cakupan kosong", () => {
  it("tombol Start disabled sampai provider dipilih + hint", () => {
    expect(dlg).toContain("disabled={!providerId || pageIds.length === 0}");
    expect(dlg).toContain("if (!providerId || pageIds.length === 0) return;");
    expect(dlg).toContain("batchDlgNeedProvider");
  });
  it("cakupan kosong disabled + hint cakupan", () => {
    expect(dlg).toContain("pageIds.length === 0");
    expect(dlg).toContain("batchDlgEmptyScope");
  });
  it("default scope cerdas: tanpa pilihan buka di visible", () => {
    expect(dlg).toContain('selectedIds.length > 0 ? "selected" : "visible"');
  });
});

describe("batch dialog: kontrak props + storage + i18n", () => {
  it("props EXACT sesuai kontrak", () => {
    for (const p of [
      "project: Project",
      "selectedIds: string[]",
      'folderId: string | "all"',
      "visiblePages: Page[]",
      "readingDirection: ReadingDirection",
      "onSubmit",
      "onClose",
    ]) {
      expect(dlg).toContain(p);
    }
  });

  it("baca/tulis STORAGE_KEYS.batchProvider + batchOpts", () => {
    expect(dlg).toContain("STORAGE_KEYS.batchProvider");
    expect(dlg).toContain("STORAGE_KEYS.batchOpts");
  });

  it("semua string baru via $t key prefix batchDlg", () => {
    const keys = dlg.match(/\$t\("batchDlg[A-Za-z]*"\)/g) ?? [];
    expect(keys.length).toBeGreaterThan(8);
    // Tidak ada label Indonesia hardcode di template dialog.
    expect(dlg).not.toContain(">Provider<");
    expect(dlg).not.toContain(">Mulai<");
    expect(dlg).not.toContain(">Batal<");
  });
});
