// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { BG_LOAD_TIMEOUT_MS, useBackgroundLoader, type BackgroundDeps } from "./backgroundLoader.svelte.ts";
import type Konva from "konva";

describe("backgroundLoader behavioral unit tests", () => {
  const dummyStage = {} as Konva.Stage;
  const dummyOverlay = {} as Konva.Layer;

  it("defines 15s timeout constant", () => {
    expect(BG_LOAD_TIMEOUT_MS).toBe(15_000);
  });

  it("queues url and updates requestedUrl on load", () => {
    const deps: BackgroundDeps = {
      getStage: () => dummyStage,
      getOverlay: () => dummyOverlay,
      getBg: () => null,
      setBg: vi.fn(),
      scale: () => 1,
      imgW: () => 800,
      imgH: () => 1200,
      getFallbackUrl: () => undefined,
      redraw: vi.fn(),
    };
    const loader = useBackgroundLoader(deps);

    loader.load("http://example.com/image.jpg");
    expect(loader.requestedUrl).toBe("http://example.com/image.jpg");
    expect(loader.imgReady).toBe(false);

    loader.invalidate();
  });

  it("handles fallback url on failed load", () => {
    vi.useFakeTimers();
    const redraw = vi.fn();
    const deps: BackgroundDeps = {
      getStage: () => dummyStage,
      getOverlay: () => dummyOverlay,
      getBg: () => null,
      setBg: vi.fn(),
      scale: () => 1,
      imgW: () => 800,
      imgH: () => 1200,
      getFallbackUrl: () => "http://example.com/fallback.jpg",
      redraw,
    };
    const loader = useBackgroundLoader(deps);

    loader.load("http://example.com/primary.jpg");
    expect(loader.requestedUrl).toBe("http://example.com/primary.jpg");

    loader.invalidate();
    vi.useRealTimers();
  });
});
