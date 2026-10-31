// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { renderWithI18n } from "../test-utils";
import Field from "./Field.svelte";
import FormHarness from "./__fixtures__/FormHarness.svelte";
import { fireEvent } from "@testing-library/svelte";

describe("ui/Field behavioral component test", () => {
  it("renders label and associates with input", () => {
    const { getByLabelText } = renderWithI18n(Field, {
      props: { label: "Username" },
    });
    const input = getByLabelText("Username");
    expect(input.tagName).toBe("INPUT");
  });

  it("renders error message and marks input invalid", () => {
    const { getByText, getByRole } = renderWithI18n(Field, {
      props: { label: "Email", error: "Invalid email address" },
    });
    expect(getByText("Invalid email address")).toBeTruthy();
    const input = getByRole("textbox");
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect(input.getAttribute("aria-describedby")).toContain("-err");
  });

  it("renders hint text when no error", () => {
    const { getByText, getByRole } = renderWithI18n(Field, {
      props: { label: "Password", hint: "Minimum 8 characters" },
    });
    expect(getByText("Minimum 8 characters")).toBeTruthy();
    const input = getByRole("textbox");
    expect(input.getAttribute("aria-invalid")).toBeNull();
    expect(input.getAttribute("aria-describedby")).toContain("-hint");
  });

  it("forwards extra HTML attributes like placeholder and disabled", () => {
    const { getByPlaceholderText } = renderWithI18n(Field, {
      props: { placeholder: "Search here...", disabled: true },
    });
    const input = getByPlaceholderText("Search here...") as HTMLInputElement;
    expect(input.disabled).toBe(true);
  });

  it("binds value two-way in a parent component", async () => {
    const { getByTestId } = renderWithI18n(FormHarness);
    const input = getByTestId("name") as HTMLInputElement;
    const stateEl = getByTestId("state");
    const saveBtn = getByTestId("save");
    const outEl = getByTestId("out");

    expect(input.value).toBe("");
    expect(stateEl.textContent).toBe("");

    await fireEvent.input(input, { target: { value: "my-name" } });
    expect(input.value).toBe("my-name");
    expect(stateEl.textContent).toBe("my-name");

    await fireEvent.click(saveBtn);
    expect(outEl.textContent).toContain("my-name");
  });
});
