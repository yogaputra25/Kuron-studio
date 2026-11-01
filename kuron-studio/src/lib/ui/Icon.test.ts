// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import Icon, { type IconName } from "./Icon.svelte";

const names: IconName[] = [
  "key", "layers", "book", "download", "upload", "folder", "image",
  "minus-circle", "trash", "repeat", "sun", "moon", "globe", "activity", "info",
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
});
