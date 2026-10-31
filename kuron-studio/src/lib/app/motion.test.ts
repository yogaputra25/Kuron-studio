import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import appSrc from "../../App.svelte?raw";
import gridSrc from "../library/PageGrid.svelte?raw";
import moveSrc from "../library/MoveDialog.svelte?raw";
import onbSrc from "./Onboarding.svelte?raw";
import panelSrc from "../ui/Panel.svelte?raw";
import btnSrc from "../ui/Button.svelte?raw";

// Motion pass: dialog, kartu grid, dropdown, onboarding, dan tombol
// punya animasi masuk/hover/tekan — bukan spinner, bukan instant-cut.
// prefers-reduced-motion tetap dijaga di app.css (sudah ada sebelumnya).
describe("motion pass", () => {
  it("semua dialog Panel fade + fly", () => {
    expect(panelSrc).toContain("transition:fade");
    expect(panelSrc).toContain("transition:fly");
  });

  it("semua tombol punya press feedback", () => {
    expect(btnSrc).toContain("active:scale-[0.97]");
  });

  it("kartu grid stagger masuk + hover lift", () => {
    // page-list-view 2.3: kartu pindah ke PageGrid — perilaku sama, lokasi baru.
    expect(gridSrc).toContain("as pg, idx (pg.id)");
    expect(gridSrc).toContain("in:fly={{ y: 14, duration: 220, delay: Math.min(idx * 15, 300) }}");
    expect(gridSrc).toContain("hover:-translate-y-0.5");
  });

  it("dropdown project dan bar seleksi beranimasi", () => {
    expect(appSrc).toContain("transition:scale={{ duration: 140");
    expect(appSrc).toContain('role="status" transition:fly={{ y: -8, duration: 180 }}');
  });

  it("onboarding masuk + slide + hover lift", () => {
    expect(onbSrc).toContain("in:fly={{ y: 16, duration: 250 }}");
    expect(onbSrc).toContain("{#key wstep}");
    expect(onbSrc).toContain("in:fly={{ x: 14, duration: 180 }}");
    expect(onbSrc).toContain("hover:-translate-y-0.5");
  });

  it("sukses pindah pop tanpa spinner", () => {
    expect(moveSrc).toContain("in:scale={{ start: 0.5, duration: 250 }}");
    expect(moveSrc).not.toContain("animate-spin");
  });

  it("reduced-motion tetap dijaga", () => {
    const css = readFileSync(fileURLToPath(new URL("../../app.css", import.meta.url)), "utf8");
    expect(css).toContain("prefers-reduced-motion");
  });
});
