// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { renderWithI18n } from "../test-utils";
import CanvasEditor from "./CanvasEditor.svelte";

describe("CanvasEditor mount test", () => {
  it("mounts CanvasEditor container and initializes stage", () => {
    const onChange = vi.fn();
    const { container } = renderWithI18n(CanvasEditor, {
      props: {
        imageUrl: "http://example.com/test.jpg",
        imgW: 800,
        imgH: 1200,
        initial: [{ x: 20, y: 30, w: 100, h: 80, confidence: 1, shape: null, kind: "rect", tail: null }],
        readingDir: "rtl",
        tool: "select",
        onChange,
      },
    });

    expect(container).toBeTruthy();
    const stageEl = container.querySelector(".konvajs-content");
    expect(stageEl).toBeTruthy();
  });
});
