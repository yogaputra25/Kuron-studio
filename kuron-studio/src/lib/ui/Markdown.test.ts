// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { render } from "@testing-library/svelte";
import Markdown, { renderInline, renderMarkdown } from "./Markdown.svelte";

describe("renderMarkdown", () => {
  it("heading, paragraf, list, dan bold", () => {
    const html = renderMarkdown("# Judul\n\nHalo **dunia**.\n\n- a\n- b");
    expect(html).toContain("<h1>Judul</h1>");
    expect(html).toContain("<p>Halo <strong>dunia</strong>.</p>");
    expect(html).toContain("<ul><li>a</li><li>b</li></ul>");
  });

  it("h2 dan paragraf multi-baris digabung", () => {
    const html = renderMarkdown("## Privasi\nbaris satu\nbaris dua");
    expect(html).toContain("<h2>Privasi</h2>");
    expect(html).toContain("<p>baris satu baris dua</p>");
  });

  it("HTML di-escape, bukan dieksekusi", () => {
    expect(renderInline('<script>alert(1)</' + 'script>')).not.toContain("<script>");
    expect(renderMarkdown("- <b>x</b>")).toContain("&lt;b&gt;x&lt;/b&gt;");
  });

  it("mount menampilkan heading", () => {
    const { container } = render(Markdown, { props: { text: "# Halo" } });
    expect(container.querySelector("h1")?.textContent).toBe("Halo");
  });
});
