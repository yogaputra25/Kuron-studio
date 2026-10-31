// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { renderWithI18n } from "../test-utils";
import Button from "../ui/Button.svelte";
import Panel from "../ui/Panel.svelte";
import ProjectSwitcher from "./shell/ProjectSwitcher.svelte";
import type { Project } from "../contracts/types";

describe("Motion and interactive feedback", () => {
  it("Button includes active press feedback class and handles click", async () => {
    const onClick = vi.fn();
    const { getByRole } = renderWithI18n(Button, {
      props: { onclick: onClick },
    });

    const btn = getByRole("button");
    expect(btn.className).toContain("active:scale-[0.97]");

    await fireEvent.click(btn);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("Panel mounts with role dialog, renders children, and handles close", async () => {
    const onClose = vi.fn();
    const children = createRawSnippet(() => ({
      render: () => "<p>Panel Content Body</p>",
    }));

    const { getByRole, getByText } = renderWithI18n(Panel, {
      props: { title: "Settings Panel", onClose, children },
    });

    const dialog = getByRole("dialog");
    expect(dialog).toBeTruthy();
    expect(dialog.getAttribute("aria-label")).toBe("Settings Panel");
    expect(getByText("Panel Content Body")).toBeTruthy();

    const closeBtn = getByRole("button", { name: /Tutup|Close/i });
    await fireEvent.click(closeBtn);
    expect(onClose).toHaveBeenCalled();
  });

  it("ProjectSwitcher opens dropdown menu and selects project", async () => {
    const onPick = vi.fn();
    const dummyProjects: Project[] = [
      { id: "p1", name: "Project Alpha", folders: [] },
      { id: "p2", name: "Project Beta", folders: [] },
    ];

    const { getByRole, getAllByRole } = renderWithI18n(ProjectSwitcher, {
      props: {
        current: dummyProjects[0],
        projects: dummyProjects,
        onPick,
      },
    });

    const trigger = getByRole("button", { name: "Pilih project" });
    await fireEvent.click(trigger);

    const options = getAllByRole("option");
    expect(options.length).toBe(2);

    await fireEvent.click(options[1]);
    expect(onPick).toHaveBeenCalledWith("p2");
  });
});
