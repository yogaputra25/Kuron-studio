// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import ProjectCard from "./shell/ProjectCard.svelte";
import type { Project } from "../contracts/types";

describe("ProjectCard behavioral component test", () => {
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
        bubbles: [],
        translation: null,
      },
    ],
  };

  it("renders card variant with open and results actions", async () => {
    const onOpen = vi.fn();
    const onResults = vi.fn();

    const { getByText, getByRole } = renderWithI18n(ProjectCard, {
      props: {
        project: dummyProject,
        selected: false,
        variant: "card",
        onOpen,
        onResults,
      },
    });

    expect(getByText("Naruto Chapter 1")).toBeTruthy();

    const resultsBtn = getByRole("button", { name: /Hasil & Ekspor|Results & Export/i });
    expect(resultsBtn).toBeTruthy();

    await fireEvent.click(resultsBtn);
    expect(onResults).toHaveBeenCalledWith("proj-1");
    expect(onOpen).not.toHaveBeenCalled(); // stopPropagation prevented open
  });

  it("clicking the main card triggers onOpen", async () => {
    const onOpen = vi.fn();
    const { getByText } = renderWithI18n(ProjectCard, {
      props: {
        project: dummyProject,
        selected: false,
        variant: "card",
        onOpen,
        onResults: vi.fn(),
      },
    });

    const cardTitle = getByText("Naruto Chapter 1");
    await fireEvent.click(cardTitle);
    expect(onOpen).toHaveBeenCalledWith("proj-1");
  });

  it("renders row variant as a dropdown option", async () => {
    const onOpen = vi.fn();
    const { getByRole, getByText } = renderWithI18n(ProjectCard, {
      props: {
        project: dummyProject,
        selected: true,
        variant: "row",
        onOpen,
      },
    });

    const option = getByRole("option");
    expect(option.getAttribute("aria-selected")).toBe("true");
    expect(getByText("Naruto Chapter 1")).toBeTruthy();

    await fireEvent.click(option);
    expect(onOpen).toHaveBeenCalledWith("proj-1");
  });
});
