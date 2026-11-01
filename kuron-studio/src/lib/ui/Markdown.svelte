<!-- Perender Markdown mini khusus teks legal (PRIVACY/TERMS/LICENSE).
  Subset: `#`/`##` heading, `- ` bullet, `**tebal**`, paragraf. Tanpa dep.
  Konten = file repo sendiri (tepercaya), tapi HTML tetap di-escape dulu. -->
<script lang="ts" module>
  function escapeHtml(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }

  /** Inline: escape lalu `**tebal**` -> <strong>. */
  export function renderInline(s: string): string {
    return escapeHtml(s).replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  }

  /** Blok -> HTML: h1/h2/ul/p. Baris lain (mis. `---`) diabaikan. */
  export function renderMarkdown(src: string): string {
    const out: string[] = [];
    const lines = src.split("\n");
    let items: string[] = [];
    const flushList = () => {
      if (items.length > 0) {
        out.push(`<ul>${items.map((i) => `<li>${i}</li>`).join("")}</ul>`);
        items = [];
      }
    };
    let para: string[] = [];
    const flushPara = () => {
      if (para.length > 0) {
        out.push(`<p>${para.map(renderInline).join(" ")}</p>`);
        para = [];
      }
    };
    for (const raw of lines) {
      const line = raw.trim();
      if (line.startsWith("## ")) {
        flushList(); flushPara();
        out.push(`<h2>${renderInline(line.slice(3).trim())}</h2>`);
      } else if (line.startsWith("# ")) {
        flushList(); flushPara();
        out.push(`<h1>${renderInline(line.slice(2).trim())}</h1>`);
      } else if (line.startsWith("- ")) {
        flushPara();
        items.push(renderInline(line.slice(2).trim()));
      } else if (line === "" || line === "---") {
        flushList(); flushPara();
      } else {
        para.push(line);
      }
    }
    flushList(); flushPara();
    return out.join("");
  }
</script>

<script lang="ts">
  interface Props {
    text: string;
  }

  let { text }: Props = $props();
</script>

<div class="legal">
  {@html renderMarkdown(text)}
</div>

<style>
  .legal :global(h1) {
    font-family: var(--font-display);
    font-size: 1rem;
    font-weight: 600;
    letter-spacing: -0.01em;
    margin-bottom: 0.5rem;
  }
  .legal :global(h2) {
    font-size: 0.72rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--ks-text-3);
    margin: 1rem 0 0.375rem;
  }
  .legal :global(p) {
    font-size: 0.85rem;
    line-height: 1.6;
    color: var(--ks-text-2);
    margin-bottom: 0.375rem;
  }
  .legal :global(ul) {
    list-style: disc;
    padding-left: 1.25rem;
    display: grid;
    gap: 0.375rem;
    margin-bottom: 0.375rem;
  }
  .legal :global(li) {
    font-size: 0.85rem;
    line-height: 1.6;
    color: var(--ks-text-2);
  }
  .legal :global(strong) {
    color: var(--ks-text);
    font-weight: 600;
  }
</style>
