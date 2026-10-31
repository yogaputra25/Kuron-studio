// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import DetectProgress from "./DetectProgress.svelte";
import type { DetectState } from "./detectProgress";

describe("DetectProgress overlay behavioral tests", () => {
  const inProgressState: DetectState = {
    total: 10,
    done: 4,
    failed: [],
    finished: false,
    file: "chapter1/page4.jpg",
  };

  const finishedWithFailuresState: DetectState = {
    total: 10,
    done: 10,
    failed: [
      { pageId: "p-3", file: "page3.jpg", message: "Model timeout" },
    ],
    file: "",
    finished: true,
  };

  it("renders status, progressbar, and percentage when running", () => {
    const { getByRole, getByText } = renderWithI18n(DetectProgress, {
      props: { state: inProgressState, onClose: vi.fn() },
    });

    expect(getByRole("status")).toBeTruthy();
    const progressbar = getByRole("progressbar");
    expect(progressbar.getAttribute("aria-valuenow")).toBe("40");
    expect(progressbar.getAttribute("aria-valuemin")).toBe("0");
    expect(progressbar.getAttribute("aria-valuemax")).toBe("100");

    expect(getByText("4 / 10")).toBeTruthy();
    expect(getByText("40%")).toBeTruthy();
    expect(getByText(/chapter1\/page4\.jpg/)).toBeTruthy();
  });

  it("displays summary and failure list when finished with errors", () => {
    const { getByRole, getByText, queryByRole } = renderWithI18n(DetectProgress, {
      props: { state: finishedWithFailuresState, onClose: vi.fn() },
    });

    expect(getByRole("status")).toBeTruthy();
    expect(queryByRole("progressbar")).toBeNull();
    expect(getByText(/page3\.jpg: Model timeout/i)).toBeTruthy();
  });

  it("calls onClose when clicking close button", async () => {
    const onClose = vi.fn();
    const { getByRole } = renderWithI18n(DetectProgress, {
      props: { state: inProgressState, onClose },
    });

    const closeBtn = getByRole("button", { name: /Tutup|Close/i });
    await fireEvent.click(closeBtn);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("auto-closes after 2s when finished without failures", async () => {
    vi.useFakeTimers();
    const onClose = vi.fn();
    const finishedState: DetectState = {
      total: 5,
      done: 5,
      failed: [],
      file: "",
      finished: true,
    };

    renderWithI18n(DetectProgress, {
      props: { state: finishedState, onClose },
    });

    expect(onClose).not.toHaveBeenCalled();
    vi.advanceTimersByTime(2000);
    expect(onClose).toHaveBeenCalledTimes(1);
    vi.useRealTimers();
  });
});
