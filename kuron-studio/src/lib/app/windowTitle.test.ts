// Unit judul jendela native: format murni + bridge setTitle diam saat gagal.
import { beforeEach, describe, expect, it, vi } from "vitest";

const setTitleMock = vi.fn();
const isFocusedMock = vi.fn();
const requestAttentionMock = vi.fn();

vi.mock("@tauri-apps/api/window", () => ({
  UserAttentionType: { Critical: 1, Informational: 2 },
  getCurrentWindow: () => ({
    setTitle: (...args: unknown[]) => setTitleMock(...args),
    isFocused: (...args: unknown[]) => isFocusedMock(...args),
    requestUserAttention: (...args: unknown[]) => requestAttentionMock(...args),
  }),
}));

import { APP_TITLE, buildWindowTitle, nudgeWindowForBatchDone, syncWindowTitle } from "./windowTitle";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("buildWindowTitle", () => {
  it("fallback saat tak ada project", () => {
    expect(buildWindowTitle(null)).toBe("Kuron Studio");
    expect(buildWindowTitle(undefined)).toBe("Kuron Studio");
    expect(buildWindowTitle("")).toBe("Kuron Studio");
    expect(buildWindowTitle("   ")).toBe("Kuron Studio");
  });

  it("format '<NamaProject> — Kuron Studio'", () => {
    expect(buildWindowTitle("Manga Ch1")).toBe("Manga Ch1 — Kuron Studio");
  });

  it("nama di-trim dulu", () => {
    expect(buildWindowTitle("  Manga Ch1  ")).toBe("Manga Ch1 — Kuron Studio");
  });

  it("APP_TITLE = fallback", () => {
    expect(APP_TITLE).toBe("Kuron Studio");
  });
});

describe("syncWindowTitle", () => {
  it("teruskan judul jadi ke setTitle", async () => {
    setTitleMock.mockResolvedValue(undefined);
    await syncWindowTitle("Manga Ch1");
    expect(setTitleMock).toHaveBeenCalledWith("Manga Ch1 — Kuron Studio");
    await syncWindowTitle(null);
    expect(setTitleMock).toHaveBeenLastCalledWith("Kuron Studio");
  });

  it("gagal diam (tanpa runtime Tauri)", async () => {
    setTitleMock.mockRejectedValue(new Error("no backend"));
    await expect(syncWindowTitle("Manga Ch1")).resolves.toBeUndefined();
  });
});

describe("nudgeWindowForBatchDone", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    isFocusedMock.mockResolvedValue(false);
    requestAttentionMock.mockResolvedValue(undefined);
  });

  it("minta perhatian Critical saat jendela tak fokus", async () => {
    await nudgeWindowForBatchDone();
    expect(requestAttentionMock).toHaveBeenCalledTimes(1);
    expect(requestAttentionMock).toHaveBeenCalledWith(1);
  });

  it("diam bila jendela sedang fokus (user sudah melihat)", async () => {
    isFocusedMock.mockResolvedValue(true);
    await nudgeWindowForBatchDone();
    expect(requestAttentionMock).not.toHaveBeenCalled();
  });

  it("gagal diam (tanpa runtime Tauri)", async () => {
    isFocusedMock.mockRejectedValue(new Error("no backend"));
    await expect(nudgeWindowForBatchDone()).resolves.toBeUndefined();
    expect(requestAttentionMock).not.toHaveBeenCalled();
  });
});
