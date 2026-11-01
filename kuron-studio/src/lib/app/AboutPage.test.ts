// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent, screen } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import { version } from "../../../package.json";
import AboutPage from "./AboutPage.svelte";

describe("AboutPage", () => {
  it("tampilkan versi + 3 seksi (bahasa default id)", () => {
    renderWithI18n(AboutPage, { props: { onClose: vi.fn() } });
    expect(screen.getByText(new RegExp(version.replace(/\./g, "\\.")))).toBeTruthy();
    expect(screen.getByText("Yang bisa dilakukan")).toBeTruthy();
    expect(screen.getByText("Privasi")).toBeTruthy();
    expect(screen.getByText("Syarat")).toBeTruthy();
  });

  it("tombol tutup memanggil onClose", async () => {
    const onClose = vi.fn();
    renderWithI18n(AboutPage, { props: { onClose } });
    await fireEvent.click(screen.getByRole("button", { name: "Tutup" }));
    expect(onClose).toHaveBeenCalledOnce();
  });
});
