// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import Snackbar from "./Snackbar.svelte";
import AlertBanners from "../app/shell/AlertBanners.svelte";

describe("Snackbar component", () => {
  it("renders status message and close button", async () => {
    const onClose = vi.fn();
    const { getByRole, getByText } = renderWithI18n(Snackbar, {
      props: { message: "41 pages moved to Chapter 2", onClose },
    });

    const statusEl = getByRole("status");
    expect(statusEl).toBeTruthy();
    expect(getByText("41 pages moved to Chapter 2")).toBeTruthy();

    const closeBtn = getByRole("button");
    await fireEvent.click(closeBtn);
    expect(onClose).toHaveBeenCalled();
  });

  it("automatically closes after duration timer expires", async () => {
    vi.useFakeTimers();
    const onClose = vi.fn();
    renderWithI18n(Snackbar, {
      props: { message: "Auto close message", duration: 1000, onClose },
    });

    expect(onClose).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1000);
    expect(onClose).toHaveBeenCalledTimes(1);
    vi.useRealTimers();
  });
});

describe("AlertBanners shell integration", () => {
  it("renders Snackbar when info is present", () => {
    const onClearInfo = vi.fn();
    const { getByRole, getByText } = renderWithI18n(AlertBanners, {
      props: {
        lastInvokeError: null,
        error: "",
        info: "Operation succeeded",
        onOpenDiagnostics: vi.fn(),
        onClearInfo,
      },
    });

    expect(getByRole("status")).toBeTruthy();
    expect(getByText("Operation succeeded")).toBeTruthy();
  });

  it("renders warning banner when error is present", () => {
    const { getByRole, getByText, queryByRole } = renderWithI18n(AlertBanners, {
      props: {
        lastInvokeError: null,
        error: "Folder already exists",
        info: "",
        onOpenDiagnostics: vi.fn(),
        onClearInfo: vi.fn(),
      },
    });

    const alert = getByRole("alert");
    expect(alert).toBeTruthy();
    expect(getByText("Folder already exists")).toBeTruthy();
    expect(queryByRole("status")).toBeNull();
  });

  it("renders invoke error with command, message, and Detail button", async () => {
    const onOpenDiagnostics = vi.fn();
    const { getByRole, getByText } = renderWithI18n(AlertBanners, {
      props: {
        lastInvokeError: { command: "import_pages", error: "Permission denied" },
        error: "",
        info: "",
        onOpenDiagnostics,
        onClearInfo: vi.fn(),
      },
    });

    const alert = getByRole("alert");
    expect(alert).toBeTruthy();
    expect(getByText("import_pages")).toBeTruthy();
    expect(getByText("Permission denied")).toBeTruthy();

    const detailBtn = getByRole("button", { name: "Detail" });
    await fireEvent.click(detailBtn);
    expect(onOpenDiagnostics).toHaveBeenCalledTimes(1);
  });
});
