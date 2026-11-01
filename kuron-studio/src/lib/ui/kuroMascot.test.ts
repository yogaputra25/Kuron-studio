// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import KuroMascot from "./KuroMascot.svelte";

describe("KuroMascot", () => {
  it("render wajah dekoratif dengan ukuran default", () => {
    const { container } = render(KuroMascot);
    const svg = container.querySelector("svg");
    expect(svg?.getAttribute("width")).toBe("48");
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
    // Kedua mata ada (grup kedip), mulut senyum ada.
    expect(container.querySelector(".kuro-eyes ellipse")).not.toBeNull();
  });

  it("tanpa progress = tanpa ring", () => {
    const { container } = render(KuroMascot);
    expect(container.querySelectorAll("svg > circle").length).toBe(0);
  });

  it("progress 25% = ring seperempat keliling", () => {
    const { container } = render(KuroMascot, { props: { size: 52, progress: 25 } });
    const circles = container.querySelectorAll("svg > circle");
    expect(circles.length).toBe(2);
    const dash = circles[1].getAttribute("stroke-dasharray") ?? "";
    const [filled, total] = dash.split(" ").map(Number);
    expect(total).toBeCloseTo(2 * Math.PI * 92, 1);
    expect(filled).toBeCloseTo(total / 4, 0);
  });

  it("progress dijepit 0–100", () => {
    const { container } = render(KuroMascot, { props: { progress: 140 } });
    const dash = container.querySelectorAll("svg > circle")[1].getAttribute("stroke-dasharray") ?? "";
    const [filled, total] = dash.split(" ").map(Number);
    expect(filled).toBeCloseTo(total, 1);
  });
});
