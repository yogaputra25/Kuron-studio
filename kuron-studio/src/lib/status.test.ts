import { describe, expect, it } from "vitest";
import { sortPagesByName, statusBadgeClass, statusLabel } from "./status";
import type { Page, PageStatus } from "./types";

const page = (path: string): Page => ({ id: path, path, width: 0, height: 0, status: "idle", bubbles: [] });

const ALL: PageStatus[] = [
  "idle",
  "detecting",
  "detected",
  "noBubbles",
  "translating",
  "translated",
  "failed",
];

describe("statusBadgeClass", () => {
  it("maps every status to a badge", () => {
    for (const s of ALL) {
      expect(statusBadgeClass(s)).toMatch(/^bg-/);
    }
  });

  // Badge lama pakai palet Tailwind mentah (bg-zinc-500) yang tidak punya
  // pasangan kontras di light theme. Semua harus lewat token semantik.
  it("uses theme tokens, never raw Tailwind palette", () => {
    for (const s of ALL) {
      const cls = statusBadgeClass(s);
      expect(cls, `${s} masih pakai palet mentah`).not.toMatch(
        /\b(bg|text)-(zinc|sky|emerald|rose|amber|violet|slate|red|blue|neutral|stone|gray|green|indigo|purple|pink|yellow|orange)-/,
      );
      // Token = prefiks yang kita daftarkan di @theme inline (app.css).
      expect(cls, `${s} bukan token yang dikenal`).toMatch(
        /\b(bg|text)-(ink|ink-2|ink-3|accent|accent-soft|accent-fg|cyan|cyan-soft|success|warn|danger|danger-soft|info|raised|surface|surface-2|line|line-strong|hover|)\b/,
      );
    }
  });

  it("labels every status", () => {
    for (const s of ALL) {
      // Tidak boleh `not.toBe(s)`: "idle" memang sama dalam Bahasa Indonesia,
      // jadi label yang identik dengan key-nya itu sah.
      expect(statusLabel(s).length).toBeGreaterThan(0);
    }
  });

  it("gives each status a distinct tone", () => {
    // detecting/translating sengaja sama (keduanya "sedang jalan") — jadi
    // detecting/translating sengaja sama tone (keduanya "sedang jalan") —
    // jadi cukup pastikan tidak semua status identik.
    expect(new Set(ALL.map(statusBadgeClass)).size).toBeGreaterThan(3);
  });
});

describe("sortPagesByName", () => {
  it("orders numerically by filename", () => {
    const out = sortPagesByName([page("p10.jpg"), page("p2.jpg"), page("p1.jpg")]);
    expect(out.map((p) => p.path)).toEqual(["p1.jpg", "p2.jpg", "p10.jpg"]);
  });
});
