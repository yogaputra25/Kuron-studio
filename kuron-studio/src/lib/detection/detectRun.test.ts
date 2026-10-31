import { describe, expect, it, vi } from "vitest";
import { createDetectRun } from "./detectRun.svelte.ts";
import type { DetectProgress, Project } from "../contracts/types";

vi.mock("../contracts/api", () => ({
  api: {
    detectBatch: vi.fn(),
    getProject: vi.fn(),
  },
}));

const unlisten = vi.fn();
let handler: ((e: { payload: DetectProgress }) => void) | null = null;
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (_event: string, cb: (e: { payload: DetectProgress }) => void) => {
    handler = cb;
    return unlisten;
  }),
}));

import { api } from "../contracts/api";

const proj = (id: string): Project => ({ id, name: id }) as unknown as Project;
const ev = (pageId: string, done = 1, total = 1) => ({ pageId, done, total, ok: true, message: "" });

describe("detectRun", () => {
  it("start memasang listener dan resolve project terbaru", async () => {
    const updated = proj("p");
    vi.mocked(api.detectBatch).mockResolvedValue([]);
    vi.mocked(api.getProject).mockResolvedValue(updated);
    const r = createDetectRun();
    const { done } = r.start("p", ["a"], new Map([["a", "a.jpg"]]));
    expect(r.state?.total).toBe(1);
    expect(await done).toBe(updated);
    expect(vi.mocked(api.detectBatch)).toHaveBeenCalledTimes(1);
    expect(vi.mocked(api.getProject)).toHaveBeenCalledWith("p");
  });

  it("cancel melepas listener dan membuang hasil", async () => {
    vi.mocked(api.detectBatch).mockImplementation(() => new Promise<never>(() => {}));
    const r = createDetectRun();
    const { done, cancel, gen } = r.start("p", ["a"], new Map());
    expect(r.isCurrent(gen)).toBe(true);
    cancel();
    expect(r.isCurrent(gen)).toBe(false);
    await expect(done).resolves.toBeNull();
    expect(unlisten).toHaveBeenCalled();
  });

  it("closeOverlay: jalan disembunyikan, selesai dibuang", async () => {
    vi.mocked(api.detectBatch).mockImplementation(() => new Promise<never>(() => {}));
    const r = createDetectRun();
    r.start("p", ["a"], new Map());
    await new Promise((r) => setTimeout(r, 0));
    r.closeOverlay();
    expect(r.hidden).toBe(true);
    expect(r.state).not.toBeNull();
    handler?.({ payload: ev("a") });
    expect(r.state?.finished).toBe(true);
    r.closeOverlay();
    expect(r.state).toBeNull();
  });
});
