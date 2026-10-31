// Unit bridge dialog (FE-7 tauri-event-bridge 2.2): normalisasi hasil.
import { beforeEach, describe, expect, it, vi } from "vitest";

const openMock = vi.fn();
const saveMock = vi.fn();

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: (...args: unknown[]) => openMock(...args),
  save: (...args: unknown[]) => saveMock(...args),
}));

import { pickFiles, pickFolder, saveAs } from "./dialogs";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("pickFolder", () => {
  it("teruskan opsi directory + title, kembalikan string / null", async () => {
    openMock.mockResolvedValue("/a");
    await expect(pickFolder("T")).resolves.toBe("/a");
    expect(openMock).toHaveBeenCalledWith({ directory: true, title: "T" });
    openMock.mockResolvedValue(null);
    await expect(pickFolder()).resolves.toBeNull();
  });
});

describe("pickFiles", () => {
  it("selalu array; string tunggal dibungkus, batal jadi []", async () => {
    openMock.mockResolvedValue(["/a.png", "/b.png"]);
    expect(await pickFiles()).toEqual(["/a.png", "/b.png"]);
    openMock.mockResolvedValue("/a.png");
    expect(await pickFiles()).toEqual(["/a.png"]);
    openMock.mockResolvedValue(null);
    expect(await pickFiles()).toEqual([]);
  });
});

describe("saveAs", () => {
  it("teruskan defaultPath + filter, kembalikan string / null", async () => {
    saveMock.mockResolvedValue("/a/x.cbz");
    await expect(saveAs("x.cbz", "CBZ", ["cbz"])).resolves.toBe("/a/x.cbz");
    expect(saveMock).toHaveBeenCalledWith({ defaultPath: "x.cbz", filters: [{ name: "CBZ", extensions: ["cbz"] }] });
    saveMock.mockResolvedValue(null);
    await expect(saveAs("x.cbz", "CBZ", ["cbz"])).resolves.toBeNull();
  });
});
