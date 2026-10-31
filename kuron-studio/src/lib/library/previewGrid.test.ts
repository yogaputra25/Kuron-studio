// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach } from "vitest";
import { createThumbnailQueue } from "../previews/thumbnailQueue.svelte.ts";
import { api } from "../contracts/api";
import type { Page } from "../contracts/types";
import { setupDomMock } from "../test-utils";

describe("Thumbnail queue integration behavior", () => {
  const dummyPages: Page[] = [
    { id: "p1", path: "01.jpg", width: 800, height: 1200, status: "idle", bubbles: [], translation: null },
    { id: "p2", path: "02.jpg", width: 800, height: 1200, status: "idle", bubbles: [], translation: null },
  ];

  beforeEach(() => {
    setupDomMock();
    vi.restoreAllMocks();
    vi.spyOn(api, "getImagePreview").mockImplementation(async (path) => `data:image/jpeg;base64,${path}`);
  });

  it("initializes with empty state and tracks progress", () => {
    const queue = createThumbnailQueue();
    expect(queue.total).toBe(0);
    expect(queue.done).toBe(0);
    expect(queue.failed).toBe(0);
    expect(queue.thumbs).toEqual({});
    expect(queue.broken).toEqual({});
  });

  it("loadThumbs updates total count and processes pages", () => {
    const queue = createThumbnailQueue();
    queue.loadThumbs(dummyPages);

    expect(queue.total).toBe(2);

    const div1 = document.createElement("div");
    const observer1 = queue.observeCard(div1, "p1");
    expect(typeof observer1.destroy).toBe("function");
    observer1.destroy();
  });

  it("pruneThumbs removes entries that are no longer live", () => {
    const queue = createThumbnailQueue();
    queue.loadThumbs(dummyPages);

    queue.pruneThumbs(new Set(["p1"]));
    expect(queue.thumbs["p2"]).toBeUndefined();
  });

  it("resetThumbs resets all progress and state", () => {
    const queue = createThumbnailQueue();
    queue.loadThumbs(dummyPages);
    queue.resetThumbs();

    expect(queue.total).toBe(0);
    expect(queue.done).toBe(0);
    expect(queue.thumbs).toEqual({});
  });
});
