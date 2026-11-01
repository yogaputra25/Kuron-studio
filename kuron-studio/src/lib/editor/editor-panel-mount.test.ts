// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { renderWithI18n } from "../test-utils";
import EditorPanel from "./EditorPanel.svelte";
import type { Page } from "../contracts/types";

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: async () => "1.0.0",
}));

describe("EditorPanel mount test", () => {
  const dummyPage: Page = {
    id: "page-1",
    path: "chapter1/01.jpg",
    width: 800,
    height: 1200,
    status: "detected",
    bubbles: [{ x: 10, y: 10, w: 100, h: 50, confidence: 1, shape: null, kind: "rect", tail: null }],
    translation: null,
  };

  it("mounts EditorPanel shell and toolbar navigation", () => {
    const onClose = vi.fn();
    const onSaved = vi.fn();
    const onBatchOpts = vi.fn();

    const { getByRole, container } = renderWithI18n(EditorPanel, {
      props: {
        projectId: "proj-1",
        page: dummyPage,
        fullImageUrl: "http://example.com/page-1.jpg",
        readingDir: "rtl",
        onClose,
        onSaved,
        onBatchOpts,
      },
    });

    expect(container).toBeTruthy();
    expect(getByRole("button", { name: "← Grid" })).toBeTruthy();
    expect(getByRole("button", { name: "Before" })).toBeTruthy();
    // Palet tool sidebar selalu tampil; toolbar ramping tanpa scroll.
    expect(getByRole("button", { name: "Rect" })).toBeTruthy();
    expect(getByRole("button", { name: "Detect" })).toBeTruthy();
  });
});
