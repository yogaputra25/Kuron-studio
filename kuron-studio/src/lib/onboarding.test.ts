import { describe, expect, it } from "vitest";
import appSrc from "../App.svelte?raw";
import onbSrc from "./Onboarding.svelte?raw";

// Onboarding: header disembunyikan saat 0 project; layar kosong =
// pilihan dua pintu (V4) + wizard 4 langkah (V3).
describe("onboarding shell", () => {
  it("header hanya tampil setelah project pertama dimuat", () => {
    expect(appSrc).toContain("{#if loaded && projects.length > 0}");
    expect(appSrc).not.toContain("Satu project = satu judul");
  });

  it("loading tidak flash onboarding", () => {
    const gate = appSrc.indexOf("{#if !loaded}");
    const mount = appSrc.indexOf("<Onboarding");
    expect(gate).toBeGreaterThan(-1);
    expect(mount).toBeGreaterThan(gate);
  });

  it("mount Onboarding dengan create + sample", () => {
    expect(appSrc).toContain("onCreate={(n) => void createProject(n)}");
    expect(appSrc).toContain("onSample={() => void createSample()}");
    expect(appSrc).toContain("api.createSample($t(\"onboardSample\"))");
  });
});

describe("onboarding choice + wizard", () => {
  it("dua pintu: kosong vs contoh", () => {
    for (const k of ["onboardChoiceT", "onboardChoiceD", "onboardOptEmptyT", "onboardOptEmptyD", "onboardOptSampleT", "onboardOptSampleD", "onboardRecommended"]) {
      expect(onbSrc).toContain(`$t("${k}")`);
    }
  });

  it("wizard 4 langkah + form di slide terakhir", () => {
    expect(onbSrc).toContain("onboardStep${wstep + 1}t");
    expect(onbSrc).toContain("onboardStep${wstep + 1}d");
    expect(onbSrc).toContain("const slides = [1, 2, 3, 4]");
    expect(onbSrc).toContain("{#each slides as s, i (s)}");
    expect(onbSrc).toContain('{$t("onboardBack")}');
    expect(onbSrc).toContain('{$t("onboardNext")}');
  });

  it("CTA kasih feedback saat sibuk + cegah double submit", () => {
    expect(onbSrc).toContain("{sampleCta}");
    expect(onbSrc).toContain("{createCta}");
    expect(appSrc).toContain("if (busy || !n) return;");
    expect(appSrc).toContain("if (busy) return;");
  });
});
