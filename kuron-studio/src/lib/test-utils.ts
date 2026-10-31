import { cleanup, render } from "@testing-library/svelte";
import { afterEach } from "vitest";
import { lang, type Lang } from "./app/i18n";

export type { Lang };
export { cleanup };

afterEach(() => {
  cleanup();
});

export function setupDomMock(): void {
  if (typeof window !== "undefined" && !("IntersectionObserver" in window)) {
    class MockIntersectionObserver {
      observe(): void {}
      unobserve(): void {}
      disconnect(): void {}
    }
    Object.defineProperty(window, "IntersectionObserver", {
      writable: true,
      configurable: true,
      value: MockIntersectionObserver,
    });
    Object.defineProperty(globalThis, "IntersectionObserver", {
      writable: true,
      configurable: true,
      value: MockIntersectionObserver,
    });
  }

  if (typeof Element !== "undefined" && !Element.prototype.animate) {
    Element.prototype.animate = (() => {
      const anim = {
        onfinish: null,
        oncancel: null,
        play: () => {},
        pause: () => {},
        cancel: () => {},
        finish: () => {},
        currentTime: 0,
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
      };
      return anim;
    }) as unknown as typeof Element.prototype.animate;
  }

  if (typeof HTMLCanvasElement !== "undefined") {
    const proto = HTMLCanvasElement.prototype;
    const dummyCtx = {
      fillRect: () => {},
      clearRect: () => {},
      getImageData: (_x: number, _y: number, w: number, h: number) => ({ data: new Array(w * h * 4) }),
      putImageData: () => {},
      createImageData: () => [],
      setTransform: () => {},
      drawImage: () => {},
      save: () => {},
      fillText: () => {},
      restore: () => {},
      beginPath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      closePath: () => {},
      stroke: () => {},
      translate: () => {},
      scale: () => {},
      rotate: () => {},
      arc: () => {},
      fill: () => {},
      measureText: () => ({ width: 0 }),
      transform: () => {},
      rect: () => {},
      clip: () => {},
      imageSmoothingEnabled: true,
      canvas: null,
    };
    proto.getContext = (() => dummyCtx) as unknown as typeof HTMLCanvasElement.prototype.getContext;
  }
}

export function setupCanvasMock(): void {
  setupDomMock();
}

export type RenderFunction = typeof render;

export const renderWithI18n: RenderFunction = (
  component,
  options,
  renderOptions,
) => {
  setupDomMock();
  if (options && typeof options === "object") {
    const candidate = (options as Record<string, unknown>).lang;
    if (candidate === "en" || candidate === "id" || candidate === "zh") {
      lang.set(candidate);
    }
  }
  return render(component, options, renderOptions);
};
