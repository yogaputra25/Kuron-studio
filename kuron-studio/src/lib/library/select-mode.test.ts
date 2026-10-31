// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import MoveDialog from "./MoveDialog.svelte";
import PageGrid from "./PageGrid.svelte";
import type { Project, Page } from "../contracts/types";

describe("select mode gating and MoveDialog behavior", () => {
  const dummyPages: Page[] = [
    { id: "p1", path: "f1/01.jpg", width: 800, height: 1200, status: "idle", bubbles: [], translation: null },
    { id: "p2", path: "f1/02.jpg", width: 800, height: 1200, status: "idle", bubbles: [], translation: null },
  ];

  const dummyProject: Project = {
    id: "proj-1",
    name: "Manga Title",
    folders: [
      { id: "f1", name: "Chapter 1", pages: [dummyPages[0], dummyPages[1]], parentId: null },
      { id: "f2", name: "Chapter 2", pages: [], parentId: null },
    ],
    pages: dummyPages,
  };

  it("PageGrid only renders checkboxes when selectMode is true", () => {
    const { queryByRole, unmount } = renderWithI18n(PageGrid, {
      props: {
        pages: dummyPages,
        thumbs: {},
        broken: {},
        selectedIds: [],
        selectMode: false,
        dragging: false,
        onCardClick: vi.fn(),
        onCardPress: vi.fn(),
        onRelease: vi.fn(),
        onToggleSelect: vi.fn(),
        observeCard: () => {},
      },
    });

    expect(queryByRole("checkbox")).toBeNull();
    unmount();

    const { getAllByRole } = renderWithI18n(PageGrid, {
      props: {
        pages: dummyPages,
        thumbs: {},
        broken: {},
        selectedIds: [],
        selectMode: true,
        dragging: false,
        onCardClick: vi.fn(),
        onCardPress: vi.fn(),
        onRelease: vi.fn(),
        onToggleSelect: vi.fn(),
        observeCard: () => {},
      },
    });

    expect(getAllByRole("checkbox").length).toBe(2);
  });

  it("MoveDialog disables the source folder", () => {
    const { getByRole } = renderWithI18n(MoveDialog, {
      props: {
        project: dummyProject,
        fromFolderId: "f1",
        pageIds: ["p1"],
        busy: false,
        onMoved: vi.fn(),
        onClose: vi.fn(),
        onError: vi.fn(),
      },
    });

    const sourceOpt = getByRole("option", { name: /Chapter 1/i }) as HTMLButtonElement;
    expect(sourceOpt.disabled).toBe(true);
    expect(sourceOpt.getAttribute("aria-disabled")).toBe("true");

    const destOpt = getByRole("option", { name: /Chapter 2/i }) as HTMLButtonElement;
    expect(destOpt.disabled).toBe(false);
  });

  it("MoveDialog close button calls onClose", async () => {
    const onClose = vi.fn();
    const { getAllByRole } = renderWithI18n(MoveDialog, {
      props: {
        project: dummyProject,
        fromFolderId: "f1",
        pageIds: ["p1"],
        busy: false,
        onMoved: vi.fn(),
        onClose,
        onError: vi.fn(),
      },
    });

    const closeBtns = getAllByRole("button", { name: /Tutup|Close/i });
    expect(closeBtns.length).toBeGreaterThan(0);
    await fireEvent.click(closeBtns[0]);
    expect(onClose).toHaveBeenCalled();
  });
});
