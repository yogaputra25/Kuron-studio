// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import Onboarding from "./Onboarding.svelte";

describe("Onboarding component behavioral tests", () => {
  it("renders choice view initially with empty and sample options", async () => {
    const onSample = vi.fn();
    const { getByText, getByRole } = renderWithI18n(Onboarding, {
      props: {
        busy: false,
        onCreate: vi.fn(),
        onSample,
      },
    });

    expect(getByText(/Kuron Studio/i)).toBeTruthy();

    const sampleBtn = getByRole("button", { name: /Contoh|Sample/i });
    await fireEvent.click(sampleBtn);
    expect(onSample).toHaveBeenCalledTimes(1);
  });

  it("navigates from choice view to wizard steps and creates project", async () => {
    const onCreate = vi.fn();
    const { getByRole, getByPlaceholderText } = renderWithI18n(Onboarding, {
      props: {
        busy: false,
        onCreate,
        onSample: vi.fn(),
      },
    });

    const emptyChoiceBtn = getByRole("button", { name: /Project kosong|Empty project/i });
    await fireEvent.click(emptyChoiceBtn);

    // Step 1 -> Step 2
    const nextBtn1 = getByRole("button", { name: /Lanjut|Next/i });
    await fireEvent.click(nextBtn1);

    // Step 2 -> Step 3
    const nextBtn2 = getByRole("button", { name: /Lanjut|Next/i });
    await fireEvent.click(nextBtn2);

    // Step 3 -> Step 4
    const nextBtn3 = getByRole("button", { name: /Lanjut|Next/i });
    await fireEvent.click(nextBtn3);

    // Step 4: Name input
    const nameInput = getByPlaceholderText(/Nama project|Project name/i);
    await fireEvent.input(nameInput, { target: { value: "My Awesome Manga" } });

    const createBtn = getByRole("button", { name: /Buat|Create/i });
    await fireEvent.click(createBtn);

    expect(onCreate).toHaveBeenCalledWith("My Awesome Manga");
  });

  it("disables buttons when busy is true", () => {
    const { getAllByRole } = renderWithI18n(Onboarding, {
      props: {
        busy: true,
        onCreate: vi.fn(),
        onSample: vi.fn(),
      },
    });

    const buttons = getAllByRole("button") as HTMLButtonElement[];
    expect(buttons.length).toBeGreaterThan(0);
    for (const btn of buttons) {
      expect(btn.disabled).toBe(true);
    }
  });
});
