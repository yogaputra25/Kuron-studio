// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import Lov from "./Lov.svelte";
import { tr } from "../app/i18n";

describe("ui/Lov behavioral dropdown", () => {
  const options = [
    { value: "opt1", label: "Alpha" },
    { value: "opt2", label: "Beta" },
    { value: "opt3", label: "Gamma" },
    { value: "opt4", label: "Delta" },
    { value: "opt5", label: "Epsilon" },
    { value: "opt6", label: "Zeta" },
  ];

  it("renders trigger button with placeholder when no value", () => {
    const { getByRole, queryByRole } = renderWithI18n(Lov, {
      props: { label: "Category", options, value: "" },
    });
    const trigger = getByRole("button", { name: /Category/i });
    expect(trigger.getAttribute("aria-haspopup")).toBe("listbox");
    expect(trigger.getAttribute("aria-expanded")).toBe("false");
    expect(queryByRole("listbox")).toBeNull();
  });

  it("opens listbox on click and selects an option", async () => {
    const onchange = vi.fn();
    const { getByRole, getAllByRole } = renderWithI18n(Lov, {
      props: { label: "Category", options, value: "", onchange },
    });
    const trigger = getByRole("button", { name: /Category/i });
    await fireEvent.click(trigger);

    expect(trigger.getAttribute("aria-expanded")).toBe("true");
    const listbox = getByRole("listbox");
    expect(listbox).toBeTruthy();

    const items = getAllByRole("option");
    expect(items.length).toBe(6);

    await fireEvent.click(items[1]); // Beta
    expect(onchange).toHaveBeenCalledWith("opt2");
  });

  it("filters options via search input when > 5 options", async () => {
    const { getByRole, getAllByRole, getByPlaceholderText } = renderWithI18n(Lov, {
      props: { label: "Category", options, value: "" },
    });
    const trigger = getByRole("button", { name: /Category/i });
    await fireEvent.click(trigger);

    const searchInput = getByPlaceholderText(tr("id", "searchList"));
    await fireEvent.input(searchInput, { target: { value: "bet" } });

    const items = getAllByRole("option");
    expect(items.length).toBe(1);
    expect(items[0].textContent).toContain("Beta");
  });

  it("closes on Escape key", async () => {
    const { getByRole, queryByRole } = renderWithI18n(Lov, {
      props: { label: "Category", options, value: "" },
    });
    const trigger = getByRole("button", { name: /Category/i });
    await fireEvent.click(trigger);
    expect(getByRole("listbox")).toBeTruthy();

    await fireEvent.keyDown(trigger, { key: "Escape" });
    expect(queryByRole("listbox")).toBeNull();
  });

  it("searchList is translated in all 3 languages", () => {
    for (const lang of ["en", "id", "zh"] as const) {
      const label = tr(lang, "searchList");
      expect(label).toBeTruthy();
      expect(label).not.toBe("searchList");
    }
  });
});
