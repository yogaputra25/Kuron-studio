// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, screen } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import UpdateButton from "./UpdateButton.svelte";
import { checkForUpdate, relaunchApp } from "./updater";

vi.mock("./updater", () => ({
  checkForUpdate: vi.fn(),
  relaunchApp: vi.fn(),
}));

const mockCheck = vi.mocked(checkForUpdate);
const mockRelaunch = vi.mocked(relaunchApp);

beforeEach(() => {
  vi.resetAllMocks();
});

function renderBtn(label?: string) {
  const onToast = vi.fn();
  const props = label === undefined ? { onToast } : { onToast, label };
  renderWithI18n(UpdateButton, { props });
  return onToast;
}

describe("UpdateButton teruskan hasil ke toast pemilik", () => {
  it("tanpa label: hanya ikon, tanpa teks/status inline", () => {
    const { container } = renderWithI18n(UpdateButton, { props: { onToast: vi.fn() } });
    expect(screen.getByRole("button", { name: "Cek update" })).toBeTruthy();
    expect(screen.queryByRole("status")).toBeNull();
    expect(container.textContent ?? "").not.toContain("versi");
  });

  it("dengan label: baris menu berikon + berteks", () => {
    renderBtn("Cek update");
    const btn = screen.getByRole("menuitem", { name: "Cek update" });
    // Ikon SVG set seragam (size-5) seperti baris menu lain — bukan glyph teks.
    expect(btn.querySelector("svg")).toBeTruthy();
    expect(btn.querySelector("svg")?.classList.contains("size-5")).toBe(true);
  });

  it("kind drawer: baris .drow berikon + onDone menutup drawer", async () => {
    const onToast = vi.fn();
    const onDone = vi.fn();
    renderWithI18n(UpdateButton, { props: { kind: "drawer", label: "Cek update", onToast, onDone } });
    const btn = screen.getByRole("button", { name: "Cek update" });
    expect(btn.classList.contains("drow")).toBe(true);
    expect(btn.querySelector("svg")).toBeTruthy();
    mockCheck.mockResolvedValueOnce(null);
    await fireEvent.click(btn);
    expect(onDone).toHaveBeenCalledOnce();
    await vi.waitFor(() => expect(onToast).toHaveBeenCalledWith("Sudah versi terbaru.", "success"));
  });

  it("kind menu + onDone: klik tutup menu lalu toast hasil", async () => {
    const onToast = vi.fn();
    const onDone = vi.fn();
    renderWithI18n(UpdateButton, { props: { label: "Cek update", onToast, onDone } });
    mockCheck.mockResolvedValueOnce(null);
    await fireEvent.click(screen.getByRole("menuitem", { name: "Cek update" }));
    expect(onDone).toHaveBeenCalledOnce();
    await vi.waitFor(() => expect(onToast).toHaveBeenCalledWith("Sudah versi terbaru.", "success"));
  });

  it("error: onToast dipanggil dengan nada error", async () => {
    const onToast = renderBtn();
    mockCheck.mockRejectedValueOnce(new Error("network down"));
    await fireEvent.click(screen.getByRole("button", { name: "Cek update" }));
    await vi.waitFor(() => expect(onToast).toHaveBeenCalledTimes(1));
    expect(onToast).toHaveBeenCalledWith("Error: network down", "error");
  });

  it("sudah terbaru: onToast sukses", async () => {
    const onToast = renderBtn();
    mockCheck.mockResolvedValueOnce(null);
    await fireEvent.click(screen.getByRole("button", { name: "Cek update" }));
    await vi.waitFor(() => expect(onToast).toHaveBeenCalledWith("Sudah versi terbaru.", "success"));
  });

  it("update tersedia: toast versi lalu relaunch + toast done", async () => {
    const onToast = renderBtn();
    mockCheck.mockResolvedValueOnce({ version: "9.9.9" });
    mockRelaunch.mockResolvedValueOnce(undefined);
    await fireEvent.click(screen.getByRole("button", { name: "Cek update" }));
    expect(mockRelaunch).toHaveBeenCalledOnce();
    await vi.waitFor(() => expect(onToast).toHaveBeenCalledTimes(2));
    expect(onToast).toHaveBeenNthCalledWith(1, "v9.9.9 tersedia.", "success");
    expect(onToast).toHaveBeenNthCalledWith(2, expect.stringContaining("Terinstal"), "success");
  });
});
