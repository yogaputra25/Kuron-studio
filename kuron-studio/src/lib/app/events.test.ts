// Unit bridge event (FE-7 tauri-event-bridge 1.2).
// Mock modul Tauri: tanpa runtime backend, janji listen dikendalikan manual.
import { beforeEach, describe, expect, it, vi } from "vitest";

const listenMock = vi.fn();
const dragMock = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => listenMock(...args),
}));

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({ onDragDropEvent: (...args: unknown[]) => dragMock(...args) }),
}));

import { onDetectProgress, onDragDrop, onTranslateProgress } from "./events";

function deferred<T>() {
  let resolve!: (v: T) => void;
  const promise = new Promise<T>((r) => { resolve = r; });
  return { promise, resolve };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("onTranslateProgress", () => {
  it("langganan event yang benar dan teruskan payload", async () => {
    const off = deferred<() => void>();
    listenMock.mockReturnValue(off.promise);
    const seen: unknown[] = [];
    const unsub = onTranslateProgress((e) => seen.push(e));
    const handler = listenMock.mock.calls[0][1] as (e: { payload: unknown }) => void;
    handler({ payload: { pageId: "p" } });
    // Bridge meneruskan event Tauri utuh (pemanggil baca .payload).
    expect(seen).toEqual([{ payload: { pageId: "p" } }]);
    const realOff = vi.fn();
    off.resolve(realOff);
    await off.promise;
    unsub();
    expect(realOff).toHaveBeenCalledTimes(1);
  });

  it("buka-tutup 10x: tiap langganan tepat satu unlisten", async () => {
    const offs: Array<() => void> = [];
    for (let k = 0; k < 10; k++) {
      const d = deferred<() => void>();
      listenMock.mockReturnValueOnce(d.promise);
      const realOff = vi.fn();
      offs.push(realOff);
      const unsub = onTranslateProgress(() => {});
      d.resolve(realOff);
      await d.promise;
      unsub();
    }
    expect(listenMock).toHaveBeenCalledTimes(10);
    for (const o of offs) expect(o).toHaveBeenCalledTimes(1);
  });

  it("unmount sebelum registrasi selesai tetap membersihkan", async () => {
    const d = deferred<() => void>();
    listenMock.mockReturnValue(d.promise);
    const unsub = onTranslateProgress(() => {});
    unsub();
    const realOff = vi.fn();
    d.resolve(realOff);
    await d.promise;
    await Promise.resolve();
    expect(realOff).toHaveBeenCalledTimes(1);
  });

  it("registrasi gagal tak melempar saat cleanup", async () => {
    listenMock.mockReturnValue(Promise.reject(new Error("no backend")));
    const unsub = onTranslateProgress(() => {});
    await Promise.resolve();
    expect(() => unsub()).not.toThrow();
  });
});

describe("onDetectProgress", () => {
  it("langganan detect_progress", () => {
    const d = deferred<() => void>();
    listenMock.mockReturnValue(d.promise);
    onDetectProgress(() => {});
    expect(listenMock.mock.calls[0][0]).toBe("detect_progress");
  });
});

describe("onDragDrop", () => {
  it("terus ke webview + unlisten jalan", async () => {
    const d = deferred<() => void>();
    dragMock.mockReturnValue(d.promise);
    const unsub = onDragDrop(() => {});
    expect(dragMock).toHaveBeenCalledTimes(1);
    const realOff = vi.fn();
    d.resolve(realOff);
    await d.promise;
    unsub();
    expect(realOff).toHaveBeenCalledTimes(1);
  });
});
