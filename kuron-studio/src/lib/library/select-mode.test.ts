import { describe, expect, it } from "vitest";
import appSrc from "../../App.svelte?raw";
import gridSrc from "./PageGrid.svelte?raw";
import moveSrc from "./MoveDialog.svelte?raw";

// Mode pilih manual: checkbox kartu + "Pilih semua" disembunyikan sampai
// user masuk mode (tombol Pilih / ctrl-klik / shift-klik / tahan gambar).
// Folder asal pilihan tampil disabled di dialog pindah.
describe("select mode gating + move source", () => {
  it("checkbox kartu hanya dirender di mode pilih", () => {
    // page-list-view 2.3: kartu pindah ke PageGrid — gating sama, lokasi baru.
    const gate = gridSrc.indexOf("{#if selectMode}");
    const checkbox = gridSrc.indexOf('type="checkbox"');
    expect(gate, "gate {#if selectMode} hilang").toBeGreaterThan(-1);
    expect(checkbox, "checkbox hilang").toBeGreaterThan(gate);
  });

  it("tombol Pilih masuk mode; Pilih semua di dalam mode", () => {
    expect(appSrc).toContain("{$t(\"selectMode\")}");
    const gate = appSrc.indexOf("{#if !selection.selectMode}");
    const selectAll = appSrc.indexOf("selection.selectAll(");
    expect(gate).toBeGreaterThan(-1);
    expect(selectAll).toBeGreaterThan(gate);
  });

  it("masuk mode via ctrl/shift-klik dan tahan gambar", () => {
    expect(gridSrc).toContain("onpointerdown={() => onCardPress(pg)}");
    expect(appSrc).toContain("selection.click(e, pg,");
    expect(appSrc).toContain("selection.press(pg)");
  });

  it("keluar mode saat Batal, pindah folder, dan selesai pindah", () => {
    expect(appSrc).toContain("selection.reset()");
    expect(appSrc).toContain("selection.setSelectMode(false)");
  });

  it("dialog pindah menandai folder asal disabled", () => {
    expect(moveSrc).toContain("singleOwnerFolderId");
    expect(moveSrc).toContain("aria-disabled={isSource}");
    expect(moveSrc).toContain("disabled={isSource}");
    expect(moveSrc).toContain("$t(\"moveCurrent\")");
  });
});

describe("move progress delight (no standard spinner)", () => {
  it("punya fase moving/moved + label tahap", () => {
    expect(moveSrc).toContain('phase = "moving"');
    expect(moveSrc).toContain('phase = "moved"');
    expect(moveSrc).toContain("movingTo");
    expect(moveSrc).toContain("moveStagePack");
    expect(moveSrc).toContain("moveStageSort");
    expect(moveSrc).toContain("moveStageSave");
  });

  it("bar gradien + angka count-up, bukan spinner standar", () => {
    expect(moveSrc).toContain("bg-gradient-to-r");
    expect(moveSrc).toContain('role="progressbar"');
    expect(moveSrc).not.toContain("animate-spin");
  });

  it("dialog dikunci saat pindah, layar sukses menutup manual", () => {
    expect(moveSrc).toContain("phase === \"moving\" ? undefined : onClose");
    expect(moveSrc).toContain("$t(\"moveDone\")");
  });
});
