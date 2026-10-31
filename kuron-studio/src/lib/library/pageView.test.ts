// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import PageGrid from "./PageGrid.svelte";
import PageList from "./PageList.svelte";
import type { Page } from "../contracts/types";

describe("PageGrid and PageList behavioral views", () => {
  const dummyPages: Page[] = [
    {
      id: "p1",
      path: "Chapter 1/01.jpg",
      width: 800,
      height: 1200,
      status: "detected",
      bubbles: [{ x: 10, y: 10, w: 50, h: 50, confidence: 1, shape: null, kind: "rect", tail: null }],
      translation: null,
    },
    {
      id: "p2",
      path: "Chapter 1/02.jpg",
      width: 800,
      height: 1200,
      status: "translated",
      bubbles: [{ x: 20, y: 20, w: 60, h: 60, confidence: 1, shape: null, kind: "rect", tail: null }],
      translation: {
        pageId: "p2",
        targetLang: "id",
        style: "natural",
        model: "gemini",
        bubbles: [
          {
            index: 0,
            x: 20,
            y: 20,
            w: 60,
            h: 60,
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
  ];

  describe("PageGrid", () => {
    it("renders page cards with filenames and badges", () => {
      const { getByText } = renderWithI18n(PageGrid, {
        props: {
          pages: dummyPages,
          thumbs: { p1: "thumb-1.jpg" },
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

      expect(getByText(/01\.jpg/)).toBeTruthy();
      expect(getByText(/02\.jpg/)).toBeTruthy();
    });

    it("triggers onCardClick when a card is clicked", async () => {
      const onCardClick = vi.fn();
      const { getByText } = renderWithI18n(PageGrid, {
        props: {
          pages: dummyPages,
          thumbs: {},
          broken: {},
          selectedIds: [],
          selectMode: false,
          dragging: false,
          onCardClick,
          onCardPress: vi.fn(),
          onRelease: vi.fn(),
          onToggleSelect: vi.fn(),
          observeCard: () => {},
        },
      });

      const card = getByText(/01\.jpg/);
      await fireEvent.click(card);
      expect(onCardClick).toHaveBeenCalledWith(expect.any(Object), dummyPages[0]);
    });

    it("renders checkboxes in select mode and calls onToggleSelect", async () => {
      const onToggleSelect = vi.fn();
      const { getAllByRole } = renderWithI18n(PageGrid, {
        props: {
          pages: dummyPages,
          thumbs: {},
          broken: {},
          selectedIds: ["p1"],
          selectMode: true,
          dragging: false,
          onCardClick: vi.fn(),
          onCardPress: vi.fn(),
          onRelease: vi.fn(),
          onToggleSelect,
          observeCard: () => {},
        },
      });

      const checkboxes = getAllByRole("checkbox") as HTMLInputElement[];
      expect(checkboxes.length).toBe(2);
      expect(checkboxes[0].checked).toBe(true);
      expect(checkboxes[1].checked).toBe(false);

      await fireEvent.click(checkboxes[1]);
      expect(onToggleSelect).toHaveBeenCalledWith("p2");
    });
  });

  describe("PageList", () => {
    it("renders listbox with rows and parent folders", () => {
      const { getByRole, getAllByRole, getByText } = renderWithI18n(PageList, {
        props: {
          pages: dummyPages,
          thumbs: {},
          broken: {},
          selectedIds: [],
          selectMode: false,
          onCardClick: vi.fn(),
          onOpen: vi.fn(),
          onToggleSelect: vi.fn(),
          observeCard: () => {},
        },
      });

      expect(getByRole("listbox")).toBeTruthy();
      const rows = getAllByRole("option");
      expect(rows.length).toBe(2);
      expect(getByText("01.jpg")).toBeTruthy();
      expect(getByText("02.jpg")).toBeTruthy();
    });

    it("supports keyboard navigation with arrow keys and space toggle", async () => {
      const onToggleSelect = vi.fn();
      const { getByRole, getAllByRole } = renderWithI18n(PageList, {
        props: {
          pages: dummyPages,
          thumbs: {},
          broken: {},
          selectedIds: [],
          selectMode: false,
          onCardClick: vi.fn(),
          onOpen: vi.fn(),
          onToggleSelect,
          observeCard: () => {},
        },
      });

      const listbox = getByRole("listbox");
      const rows = getAllByRole("option");

      expect(rows[0].getAttribute("aria-current")).toBe("true");

      await fireEvent.keyDown(listbox, { key: "ArrowDown" });
      expect(rows[1].getAttribute("aria-current")).toBe("true");

      await fireEvent.keyDown(listbox, { key: " " });
      expect(onToggleSelect).toHaveBeenCalledWith("p2");

      await fireEvent.keyDown(listbox, { key: "ArrowUp" });
      expect(rows[0].getAttribute("aria-current")).toBe("true");
    });
  });
});
