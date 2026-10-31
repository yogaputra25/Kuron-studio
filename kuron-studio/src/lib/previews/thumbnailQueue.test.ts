import { beforeEach, describe, expect, it, vi } from "vitest";
import { createThumbnailQueue } from "./thumbnailQueue.svelte.ts";
import type { Page } from "../contracts/types";

vi.mock("../contracts/api", () => ({
  api: {
    getImagePreview: vi.fn(),
  },
}));

import { api } from "../contracts/api";
beforeEach(() => {
  vi.mocked(api.getImagePreview).mockClear();
});
const pg = (id: string): Page => ({ id, path: `${id}.jpg` }) as unknown as Page;
const flush = async (ms = 20) => await new Promise((r) => setTimeout(r, ms));

describe("thumbnailQueue", () => {
  it("memompa sesuai urutan nama (terlihat dulu, lalu prefetch)", async () => {
    vi.mocked(api.getImagePreview).mockImplementation(async (path: unknown) => `ok:${String(path)}`);
    const q = createThumbnailQueue();
    q.loadThumbs([pg("c"), pg("a"), pg("b")]);
    await flush(50);
    const calls = vi.mocked(api.getImagePreview).mock.calls.map((c) => c[0]);
    expect(calls).toEqual(["a.jpg", "b.jpg", "c.jpg"]);
    expect(Object.keys(q.thumbs)).toEqual(["a", "b", "c"]);
    expect(q.done).toBe(3);
  });

  it("generasi basi dibuang: load kedua menang", async () => {
    let resolveFirst!: (v: string) => void;
    vi.mocked(api.getImagePreview)
      .mockImplementationOnce(() => new Promise<string>((r) => (resolveFirst = r)))
      .mockImplementation(async (path: unknown) => `ok:${String(path)}`);
    const q = createThumbnailQueue();
    q.loadThumbs([pg("lama")]);
    q.loadThumbs([pg("baru")]);
    resolveFirst("ok:lama.jpg");
    await flush(50);
    expect(Object.keys(q.thumbs)).toEqual(["baru"]);
  });

  it("retry hanya memuat ulang yang gagal", async () => {
    const pages = [pg("a"), pg("b")];
    vi.mocked(api.getImagePreview).mockImplementation(async (path: unknown) => {
      if (String(path) === "b.jpg") throw new Error("hilang");
      return `ok:${String(path)}`;
    });
    const q = createThumbnailQueue();
    q.loadThumbs(pages);
    await flush(50);
    expect(Object.keys(q.broken)).toEqual(["b"]);
    expect(q.failed).toBe(1);
    vi.mocked(api.getImagePreview).mockClear();
    vi.mocked(api.getImagePreview).mockImplementation(async (path: unknown) => `ok:${String(path)}`);
    q.retryThumbs(pages);
    await flush(50);
    const calls = vi.mocked(api.getImagePreview).mock.calls.map((c) => c[0]);
    expect(calls).toEqual(["b.jpg"]);
    expect(q.broken).toEqual({});
    expect(q.thumbs["b"]).toBe("ok:b.jpg");
  });

  it("melewati yang sudah ada (tak fetch ulang)", async () => {
    vi.mocked(api.getImagePreview).mockImplementation(async (path: unknown) => `ok:${String(path)}`);
    const q = createThumbnailQueue();
    q.loadThumbs([pg("a")]);
    await flush(50);
    expect(vi.mocked(api.getImagePreview)).toHaveBeenCalledTimes(1);
    q.loadThumbs([pg("a"), pg("b")]);
    await flush(50);
    const calls = vi.mocked(api.getImagePreview).mock.calls.map((c) => c[0]);
    expect(calls).toEqual(["a.jpg", "b.jpg"]);
  });
});
