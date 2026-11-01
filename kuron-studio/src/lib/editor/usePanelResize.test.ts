// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { usePanelResize } from "./usePanelResize.svelte";

function fakePointer(): PointerEvent {
  return {
    preventDefault: vi.fn(),
    target: document.createElement("div"),
    clientX: 100,
    clientY: 100,
    pointerId: 1,
  } as unknown as PointerEvent;
}

describe("resize drag selection lock", () => {
  it("down kunci user-select, up melepas", () => {
    document.body.style.userSelect = "";
    const body = document.createElement("div");
    const resize = usePanelResize({ getBody: () => body, onLayoutSettled: () => {} });
    resize.down(fakePointer());
    expect(document.body.style.userSelect).toBe("none");
    resize.up();
    expect(document.body.style.userSelect).toBe("");
  });
});
