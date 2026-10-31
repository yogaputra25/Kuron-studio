// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import ReviewGrid from "./ReviewGrid.svelte";
import type { Project, BatchOpts } from "../contracts/types";
import { defaultTranslateOpts } from "../provider/translateOpts";

describe("ReviewGrid behavioral mount tests", () => {
  const dummyProject: Project = {
    id: "proj-1",
    name: "Naruto Chapter 1",
    folders: [],
    pages: [
      {
        id: "p1",
        path: "01.jpg",
        width: 800,
        height: 1200,
        status: "translated",
        bubbles: [{ x: 10, y: 10, w: 20, h: 20, confidence: 1, shape: null, kind: "rect", tail: null }],
        translation: {
          pageId: "p1",
          targetLang: "id",
          style: "natural",
          model: "gemini",
          bubbles: [
            {
              index: 0,
              x: 10,
              y: 10,
              w: 20,
              h: 20,
              original: "hi",
              reading: "",
              translated: "halo",
              aiOriginal: "hi",
              aiReading: "",
              aiTranslated: "halo",
              needsWhitePatch: false,
              isUserEdited: false,
            },
          ],
        },
      },
      {
        id: "p2",
        path: "02.jpg",
        width: 800,
        height: 1200,
        status: "detected",
        bubbles: [{ x: 30, y: 30, w: 20, h: 20, confidence: 1, shape: null, kind: "rect", tail: null }],
        translation: null,
      },
    ],
  };

  const dummyOpts: BatchOpts = {
    ...defaultTranslateOpts(),
    providerId: "p1",
    readingDirection: "rtl",
  };

  it("renders progress summary, percentages, and progressbar", () => {
    const { getByRole, getAllByText } = renderWithI18n(ReviewGrid, {
      props: {
        project: dummyProject,
        opts: dummyOpts,
        selectedIds: [],
        onRefresh: vi.fn(),
        onClose: vi.fn(),
      },
    });

    expect(getByRole("dialog")).toBeTruthy();
    const pb = getByRole("progressbar");
    expect(pb.getAttribute("aria-valuenow")).toBe("50");
    expect(getAllByText(/50%/).length).toBeGreaterThan(0);
    expect(getAllByText(/1 selesai/i).length).toBeGreaterThan(0);
  });

  it("renders export scope radiogroup and format buttons", () => {
    const { getByRole } = renderWithI18n(ReviewGrid, {
      props: {
        project: dummyProject,
        opts: dummyOpts,
        selectedIds: [],
        onRefresh: vi.fn(),
        onClose: vi.fn(),
      },
    });

    expect(getByRole("radiogroup")).toBeTruthy();
    expect(getByRole("button", { name: /Export JSON/i })).toBeTruthy();
    expect(getByRole("button", { name: /Export PNG/i })).toBeTruthy();
    expect(getByRole("button", { name: /Export CBZ/i })).toBeTruthy();
    expect(getByRole("button", { name: /^PSD$/i })).toBeTruthy();
    expect(getByRole("button", { name: /^ZIP$/i })).toBeTruthy();
  });

  it("calls onClose when close button is clicked", async () => {
    const onClose = vi.fn();
    const { getAllByRole } = renderWithI18n(ReviewGrid, {
      props: {
        project: dummyProject,
        opts: dummyOpts,
        selectedIds: [],
        onRefresh: vi.fn(),
        onClose,
      },
    });

    const closeBtns = getAllByRole("button", { name: /Tutup|Close/i });
    expect(closeBtns.length).toBeGreaterThan(0);
    await fireEvent.click(closeBtns[0]);
    expect(onClose).toHaveBeenCalled();
  });
});
