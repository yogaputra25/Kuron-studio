// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach } from "vitest";
import { fireEvent, waitFor } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import BatchPanel from "./BatchPanel.svelte";
import TranslatePanel from "./TranslatePanel.svelte";
import { createExportState } from "../export/exportState.svelte";
import { api } from "../contracts/api";
import { defaultTranslateOpts, toBatchInput, toRetryBubbleInput, toTranslateArgs } from "./translateOpts";
import type { AiModelOption, Page, Project, ProviderView } from "../contracts/types";

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

describe("exportState converter wiring (review-grid-split)", () => {
  // Sejak split #50, retry dari ReviewGrid lewat exportState — harus tetap
  // memakai konverter (port behavioral dari assertion source-pattern main).
  const page: Page = {
    id: "pg-1",
    path: "folder/01.jpg",
    width: 800,
    height: 1200,
    status: "detected",
    bubbles: [],
    translation: null,
  };
  const project = { id: "pr-1", name: "P", pages: [page], folders: [] } as Project;
  const opts = {
    ...defaultTranslateOpts(),
    providerId: "p1",
    model: "gpt-4o-mini",
    readingDirection: "rtl" as const,
  };

  function wire(getOpts: () => typeof opts | null = () => opts) {
    const onRefresh = vi.fn();
    const st = createExportState({
      getProject: () => project,
      getOpts,
      getSelectedIds: () => [],
      onRefresh,
      getPageFilter: () => "all",
      getFolderFilter: () => "all",
    });
    return { st, onRefresh };
  }

  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("retryPage goes through toBatchInput with opts model", async () => {
    const batchSpy = vi.spyOn(api, "translateBatch").mockResolvedValue([]);
    const { st, onRefresh } = wire();
    await st.retryPage("pg-1");
    expect(batchSpy).toHaveBeenCalledWith(
      expect.objectContaining({ pageIds: ["pg-1"], providerId: "p1", model: "gpt-4o-mini" }),
    );
    expect(onRefresh).toHaveBeenCalled();
  });

  it("retryBubble goes through toRetryBubbleInput with opts model", async () => {
    const retrySpy = vi.spyOn(api, "retryBubble").mockResolvedValue({} as never);
    const { st, onRefresh } = wire();
    await st.retryBubble("pg-1", 0);
    expect(retrySpy).toHaveBeenCalledWith(
      expect.objectContaining({ pageId: "pg-1", bubbleIndex: 0, providerId: "p1", model: "gpt-4o-mini" }),
    );
    expect(onRefresh).toHaveBeenCalled();
  });

  it("retry without provider sets error without POST", async () => {
    const batchSpy = vi.spyOn(api, "translateBatch").mockResolvedValue([]);
    const { st } = wire(() => null);
    await st.retryPage("pg-1");
    expect(batchSpy).not.toHaveBeenCalled();
    expect(st.error).not.toBe("");
  });
});

describe("TranslatePanel card order and session-only model", () => {
  const providers: ProviderView[] = [{ id: "p1", name: "P", model: "m1" } as ProviderView];
  const models: AiModelOption[] = [
    { id: "m1", label: "M1", vision: true },
    { id: "m2", label: "M2", vision: false },
  ];

  function mountPanel(onOptsChange = vi.fn()) {
    return {
      onOptsChange,
      ...renderWithI18n(TranslatePanel, {
        props: {
          pageId: "pg-1",
          providerId: "p1",
          bubblesCount: 0,
          readingDir: "rtl" as const,
          originals: [],
          onProviderChange: vi.fn(),
          onOptsChange,
          onTranslated: vi.fn(),
          onError: vi.fn(),
        },
      }),
    };
  }

  beforeEach(() => {
    vi.restoreAllMocks();
    vi.spyOn(api, "getProviders").mockResolvedValue(providers);
    vi.spyOn(api, "listModels").mockResolvedValue(models);
    vi.spyOn(api, "glossaryContext").mockResolvedValue(null);
    vi.spyOn(api, "saveProvider").mockResolvedValue({} as ProviderView);
  });

  it("Model renders before Target, advanced opts in details, action bar sticky", async () => {
    const { getByText, container } = mountPanel();
    await waitFor(() => expect(getByText("Model")).toBeTruthy());
    const modelEl = getByText("Model");
    const targetEl = getByText("Target");
    expect(modelEl.compareDocumentPosition(targetEl) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(container.querySelector("details")).toBeTruthy();
    expect(container.querySelector(".sticky.bottom-0")).toBeTruthy();
  });

  it("changing model notifies parent without writing provider DB", async () => {
    const onOptsChange = vi.fn();
    const { getByRole, getAllByRole, getByText } = mountPanel(onOptsChange);
    await waitFor(() => expect(getByText("2 model.")).toBeTruthy());
    await fireEvent.click(getByRole("button", { name: /Model/i }));
    const items = getAllByRole("option");
    expect(items.length).toBe(2);
    await fireEvent.click(items[1]); // M2
    await waitFor(() => {
      expect(onOptsChange).toHaveBeenCalledWith(expect.objectContaining({ model: "m2" }));
      expect(api.saveProvider).not.toHaveBeenCalled();
    });
  });
});
