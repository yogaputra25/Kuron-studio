import { describe, expect, it } from "vitest";
import { scrub } from "./log";

// Log ikut ke file yang nanti ditempel ke issue tracker. Kalau API key bocor
// ke sana, itu kebocoran kredensial yang permanen — test ini penjaganya.

describe("scrub tidak membocorkan rahasia ke log", () => {
  it("menyensor apiKey di level atas", () => {
    const out = scrub({ name: "local", apiKey: "sk-super-secret" }) as Record<string, unknown>;
    expect(out.apiKey).toBe("<redacted>");
    expect(out.name).toBe("local");
  });

  it("menyensor di dalam object bersarang", () => {
    const out = scrub({
      outer: { inner: { key: "hunter2", model: "muse" } },
    }) as any;
    expect(out.outer.inner.key).toBe("<redacted>");
    expect(out.outer.inner.model).toBe("muse");
  });

  it("menyensor di dalam array", () => {
    const out = scrub({ items: [{ password: "rahasia" }] }) as any;
    expect(out.items[0].password).toBe("<redacted>");
  });

  it("menandai key kosong secara berbeda dari yang terisi", () => {
    const out = scrub({ apiKey: "", token: "ada" }) as Record<string, unknown>;
    expect(out.apiKey, "kosong vs terisi harus bisa dibedakan").toBe("<empty>");
    expect(out.token).toBe("<redacted>");
  });

  it("menyensor semua nama field sensitif yang terdaftar", () => {
    for (const k of [
      "apiKey", "api_key", "key", "authorization", "bearer", "password", "token",
    ]) {
      const out = scrub({ [k]: "rahasia" }) as Record<string, unknown>;
      expect(out[k], `field ${k} bocor`).not.toBe("rahasia");
    }
  });

  it("memotong string panjang supaya log tidak membloki file", () => {
    const out = scrub({ note: "x".repeat(10_000) }) as Record<string, string>;
    expect(out.note.length).toBeLessThanOrEqual(301);
    expect(out.note.endsWith("…")).toBe(true);
  });

  it("meringkas array besar tanpa membanjiri log", () => {
    const out = scrub({ ids: Array.from({ length: 500 }, (_, i) => i) }) as Record<
      string,
      unknown
    >;
    expect(typeof out.ids).toBe("string");
    expect(String(out.ids)).toContain("len=500");
  });

  it("menghentikan rekursi tak berujung pada data bersiklus", () => {
    const cyclic: Record<string, unknown> = { name: "a" };
    cyclic.self = cyclic;
    expect(() => scrub(cyclic, 0)).not.toThrow();
  });

  it("membiarkan primitif apa adanya", () => {
    expect(scrub(42)).toBe(42);
    expect(scrub(true)).toBe(true);
    expect(scrub(null)).toBe(null);
    expect(scrub(undefined)).toBe(undefined);
  });
});
