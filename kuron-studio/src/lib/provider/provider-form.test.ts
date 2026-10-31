// @vitest-environment jsdom
import { describe, expect, it, vi, beforeEach } from "vitest";
import { fireEvent, waitFor } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import ProviderSettings from "./ProviderSettings.svelte";
import { api } from "../contracts/api";
import type { ProviderView } from "../contracts/types";

describe("ProviderSettings behavioral mount tests", () => {
  const dummyProviders: ProviderView[] = [
    {
      id: "p1",
      providerType: "gemini",
      name: "My Gemini",
      baseUrl: "https://generativelanguage.googleapis.com",
      model: "gemini-1.5-flash",
      hasKey: true,
      isVisionCapable: true,
    },
  ];

  beforeEach(() => {
    vi.restoreAllMocks();
    vi.spyOn(api, "getProviders").mockResolvedValue(dummyProviders);
    vi.spyOn(api, "listModelsDraft").mockResolvedValue([
      { id: "gemini-2.0-flash", label: "Gemini 2.0 Flash", vision: true },
      { id: "gemini-text-only", label: "Gemini Text Only", vision: false },
    ]);
    vi.spyOn(api, "saveProvider").mockImplementation(async (arg) => ({
      id: arg.id ?? "p-new",
      providerType: arg.providerType,
      name: arg.name,
      baseUrl: arg.baseUrl,
      model: arg.model,
      hasKey: !!arg.apiKey,
      isVisionCapable: true,
    }));
    vi.spyOn(api, "validateProvider").mockResolvedValue({
      ok: true,
      message: "Koneksi berhasil",
    });
  });

  it("loads and displays providers on mount", async () => {
    const { getByText } = renderWithI18n(ProviderSettings, {
      props: { onClose: vi.fn() },
    });
    await waitFor(() => {
      expect(getByText("My Gemini")).toBeTruthy();
    });
  });

  it("shows validation error on empty name when saving", async () => {
    const { getByRole, getByText } = renderWithI18n(ProviderSettings, {
      props: { onClose: vi.fn() },
    });

    await waitFor(() => {
      expect(getByText("My Gemini")).toBeTruthy();
    });

    const saveBtn = getByRole("button", { name: /^Simpan$/i });
    await fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(getByText("Wajib diisi.")).toBeTruthy();
    });
  });

  it("clears validation error when typing name", async () => {
    const { getByRole, getByText, getByLabelText, queryByText } = renderWithI18n(ProviderSettings, {
      props: { onClose: vi.fn() },
    });

    await waitFor(() => {
      expect(getByText("My Gemini")).toBeTruthy();
    });

    const saveBtn = getByRole("button", { name: /^Simpan$/i });
    await fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(getByText("Wajib diisi.")).toBeTruthy();
    });

    const nameInput = getByLabelText(/^Nama$/i);
    await fireEvent.input(nameInput, { target: { value: "New Provider" } });

    await waitFor(() => {
      expect(queryByText("Wajib diisi.")).toBeNull();
    });
  });

  it("loads models in draft mode with api key typed", async () => {
    const { getByRole, getByLabelText, getByText } = renderWithI18n(ProviderSettings, {
      props: { onClose: vi.fn() },
    });

    await waitFor(() => {
      expect(getByText("My Gemini")).toBeTruthy();
    });

    const keyInput = getByLabelText(/API key/i);
    await fireEvent.input(keyInput, { target: { value: "secret-key-123" } });

    const loadModelsBtn = getByRole("button", { name: /Load models/i });
    await fireEvent.click(loadModelsBtn);

    await waitFor(() => {
      expect(api.listModelsDraft).toHaveBeenCalledWith({
        providerType: "gemini",
        baseUrl: expect.any(String),
        apiKey: "secret-key-123",
      });
      expect(getByText(/2 model\./i)).toBeTruthy();
    });
  });

  it("saves provider and remains in edit mode so validate can be used immediately", async () => {
    const { getByRole, getByLabelText, getByText } = renderWithI18n(ProviderSettings, {
      props: { onClose: vi.fn() },
    });

    await waitFor(() => {
      expect(getByText("My Gemini")).toBeTruthy();
    });

    const nameInput = getByLabelText(/^Nama$/i);
    await fireEvent.input(nameInput, { target: { value: "Gemini Pro" } });

    const keyInput = getByLabelText(/API key/i);
    await fireEvent.input(keyInput, { target: { value: "test-api-key" } });

    const saveBtn = getByRole("button", { name: /^Simpan$/i });
    await fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.saveProvider).toHaveBeenCalled();
    });

    const validateBtn = getByRole("button", { name: /Validate/i });
    await fireEvent.click(validateBtn);

    await waitFor(() => {
      expect(api.validateProvider).toHaveBeenCalledWith("p-new");
      expect(getByText("Koneksi berhasil")).toBeTruthy();
    });
  });
});
