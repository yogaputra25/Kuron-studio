// Unit modul storage (FE-8 app-storage 1.2). Node env + stub localStorage,
// tanpa jsdom. Stub disuntik via globalThis dan dipulihkan tiap test.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { read, remove, STORAGE_KEYS, write } from "./storage";

function stubStorage(impl?: Partial<Storage>) {
  const store = new Map<string, string>();
  const base: Storage = {
    get length() { return store.size; },
    clear: () => { store.clear(); },
    getItem: (k: string) => (store.has(k) ? store.get(k)! : null),
    key: (i: number) => [...store.keys()][i] ?? null,
    removeItem: (k: string) => { store.delete(k); },
    setItem: (k: string, v: string) => { store.set(k, String(v)); },
    ...impl,
  };
  vi.stubGlobal("localStorage", base);
  return store;
}

beforeEach(() => {
  vi.unstubAllGlobals();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("read", () => {
  it("nilai valid dikembalikan apa adanya", () => {
    const store = stubStorage();
    store.set("kuron-lang:v1", "en");
    expect(read(STORAGE_KEYS.lang, "id", (v): v is string => v === "en" || v === "id" || v === "zh")).toBe("en");
  });

  it("nilai rusak dibuang + fallback, tak pernah throw", () => {
    const store = stubStorage();
    store.set("kuron-sidebar-w:v1", "abc");
    expect(read(STORAGE_KEYS.sidebarW, 300)).toBe(300);
    expect(store.has("kuron-sidebar-w:v1")).toBe(false);
  });

  it("versi lama migrasi sekali: pindah + key lama hilang", () => {
    const store = stubStorage();
    store.set("kuron-theme", "light");
    const v = read(STORAGE_KEYS.theme, "dark", (x): x is string => x === "light" || x === "dark");
    expect(v).toBe("light");
    expect(store.get("kuron-theme:v1")).toBe("light");
    expect(store.has("kuron-theme")).toBe(false);
  });

  it("tipe salah (guard tolak) = fallback + key dibuang", () => {
    const store = stubStorage();
    store.set("kuron-lang:v1", "xx");
    const v = read(STORAGE_KEYS.lang, "id", (x): x is string => x === "en" || x === "id" || x === "zh");
    expect(v).toBe("id");
    expect(store.has("kuron-lang:v1")).toBe(false);
  });

  it("storage melempar = hilang (baca fallback, tulis no-op)", () => {
    stubStorage({
      getItem: () => { throw new DOMException("denied"); },
      setItem: () => { throw new DOMException("denied"); },
      removeItem: () => { throw new DOMException("denied"); },
    });
    expect(read(STORAGE_KEYS.pageView, "grid")).toBe("grid");
    expect(() => write(STORAGE_KEYS.pageView, "list")).not.toThrow();
    expect(() => remove(STORAGE_KEYS.pageView)).not.toThrow();
  });
});

describe("write/remove", () => {
  it("round-trip + hapus", () => {
    stubStorage();
    write(STORAGE_KEYS.sidebarW, 320);
    expect(read(STORAGE_KEYS.sidebarW, 300)).toBe(320);
    remove(STORAGE_KEYS.sidebarW);
    expect(read(STORAGE_KEYS.sidebarW, 300)).toBe(300);
  });
});
