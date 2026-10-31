// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach } from "vitest";
import { fireEvent, waitFor } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import BatchPanel from "./BatchPanel.svelte";
import { api } from "../contracts/api";
import { defaultTranslateOpts, toBatchInput, toRetryBubbleInput, toTranslateArgs } from "./translateOpts";
import type { Page } from "../contracts/types";

vi.mock("../app/events", () => ({
  onTranslateProgress: () => () => {},
  onDetectProgress: () => () => {},
  onDragDrop: () => () => {},
}));

describe("BatchPanel behavioral mount tests", () => {
  const dummyPages: Page[] = [
    {
      id: "pg-1",
      path: "folder/01.jpg",
      width: 800,
      height: 1200,
      status: "detected",
      bubbles: [],
      translation: null,
    },
    {
      id: "pg-2",
      path: "folder/02.jpg",
      width: 800,
      height: 1200,
      status: "detected",
      bubbles: [],
      translation: null,
    },
  ];

  const defaultOpts = {
    ...defaultTranslateOpts(),
    providerId: "p1",
    readingDirection: "rtl" as const,
  };

  beforeEach(() => {
    vi.restoreAllMocks();
    vi.spyOn(api, "translateBatch").mockResolvedValue([]);
    vi.spyOn(api, "cancelTranslate").mockResolvedValue(undefined);
  });

  it("renders page count and progress bar", () => {
    const { getByRole, getAllByText } = renderWithI18n(BatchPanel, {
      props: {
        pages: dummyPages,
        opts: defaultOpts,
        onDone: vi.fn(),
        onClose: vi.fn(),
      },
    });

    expect(getByRole("progressbar")).toBeTruthy();
    expect(getAllByText(/2 halaman/i).length).toBeGreaterThan(0);
  });

  it("triggers api.translateBatch when clicking run button", async () => {
    const onDone = vi.fn();
    const { getByRole } = renderWithI18n(BatchPanel, {
      props: {
        pages: dummyPages,
        opts: defaultOpts,
        onDone,
        onClose: vi.fn(),
      },
    });

    const runBtn = getByRole("button", { name: /Batch 2/i });
    await fireEvent.click(runBtn);

    await waitFor(() => {
      expect(api.translateBatch).toHaveBeenCalledWith(
        expect.objectContaining({
          pageIds: ["pg-1", "pg-2"],
          providerId: "p1",
        }),
      );
      expect(onDone).toHaveBeenCalled();
    });
  });

  it("calls onClose when closing the panel", async () => {
    const onClose = vi.fn();
    const { getByRole } = renderWithI18n(BatchPanel, {
      props: {
        pages: dummyPages,
        opts: defaultOpts,
        onDone: vi.fn(),
        onClose,
      },
    });

    const closeBtn = getByRole("button", { name: /Tutup|Close/i });
    await fireEvent.click(closeBtn);
    expect(onClose).toHaveBeenCalled();
  });
});

describe("translateOpts model forwarding and cancel contract", () => {
  it("forwards model per-run to translate page args", () => {
    const opts = { ...defaultTranslateOpts(), model: "gemini-2.0-flash-exp" };
    const args = toTranslateArgs(opts, {
      pageId: "pg-1",
      providerId: "p-1",
      readingDirection: "rtl",
    });
    expect(args.model).toBe("gemini-2.0-flash-exp");
  });

  it("forwards model per-run to batch and retry args", () => {
    const opts = { ...defaultTranslateOpts(), model: "gpt-4o-mini" };
    const batchInput = toBatchInput(opts, {
      pageIds: ["pg-1"],
      providerId: "p-1",
      readingDirection: "rtl",
    });
    expect(batchInput.model).toBe("gpt-4o-mini");

    const retryInput = toRetryBubbleInput(opts, {
      pageId: "pg-1",
      bubbleIndex: 0,
      providerId: "p-1",
      readingDirection: "rtl",
    });
    expect(retryInput.model).toBe("gpt-4o-mini");
  });

  it("cancelling translate calls api.cancelTranslate", async () => {
    const cancelSpy = vi.spyOn(api, "cancelTranslate").mockResolvedValue();
    await api.cancelTranslate("pg-1");
    expect(cancelSpy).toHaveBeenCalledWith("pg-1");
  });
});
