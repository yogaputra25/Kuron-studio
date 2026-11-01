// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import Icon, { type IconName } from "./Icon.svelte";

const names: IconName[] = [
  "key", "layers", "book", "download", "upload", "folder", "image",
  "minus-circle", "trash", "repeat", "sun", "moon", "globe", "activity", "info",
  "pen", "select", "rect", "ellipse", "freeform", "tail", "search", "save",
  "lock", "file-text", "award", "code", "server", "cpu", "dots",
  "chevron-down", "grid", "list", "circle", "clock", "check-circle", "x-circle", "chevron-right", "chevron-up",
  "refresh",
];

describe("Icon", () => {
  it("semua nama render satu svg currentColor", () => {
    for (const name of names) {
      const { container, unmount } = render(Icon, { props: { name } });
      const svg = container.querySelector("svg");
      expect(svg, name).toBeTruthy();
      expect(svg?.getAttribute("stroke")).toBe("currentColor");
      unmount();
    }
  });
  it("skala sm/md/lg memetakan ke size-4/5/6", () => {
    for (const [size, cls] of [["sm", "size-4"], ["md", "size-5"], ["lg", "size-6"]] as const) {
      const { container, unmount } = render(Icon, { props: { name: "info", size } });
      expect(container.querySelector("svg")?.classList.contains(cls), size).toBe(true);
      unmount();
    }
  });
});
