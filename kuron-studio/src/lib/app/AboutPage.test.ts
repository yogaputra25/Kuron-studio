// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent, screen } from "@testing-library/svelte";
import { renderWithI18n } from "../test-utils";
import { version } from "../../../package.json";
import AboutPage from "./AboutPage.svelte";

describe("AboutPage", () => {
  it("tampilkan versi + tombol khusus + kredit (bahasa default id)", () => {
    renderWithI18n(AboutPage, { props: { onClose: vi.fn() } });
    expect(screen.getByText(new RegExp(version.replace(/\./g, "\\.")))).toBeTruthy();
    expect(screen.getByRole("button", { name: /Privasi/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Syarat/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Lisensi/ })).toBeTruthy();
    expect(screen.getByText(/Muse/)).toBeTruthy();
  });

  it("tombol tutup memanggil onClose", async () => {
    const onClose = vi.fn();
    renderWithI18n(AboutPage, { props: { onClose } });
    await fireEvent.click(screen.getByRole("button", { name: "Tutup" }));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("tombol khusus membuka Privacy dari PRIVACY.md dan kembali", async () => {
    const onClose = vi.fn();
    renderWithI18n(AboutPage, { props: { onClose } });
    await fireEvent.click(screen.getByRole("button", { name: /Privasi/ }));
    expect(screen.getByText(/tidak ada server kami/)).toBeTruthy();
    // Kembali ke utama: tombol Privacy muncul lagi.
    await fireEvent.click(screen.getByRole("button", { name: /Kembali/ }));
    expect(screen.getByRole("button", { name: /Privasi/ })).toBeTruthy();
  });

  it("tombol Terms dan License menampilkan isi file repo", async () => {
    renderWithI18n(AboutPage, { props: { onClose: vi.fn() } });
    await fireEvent.click(screen.getByRole("button", { name: /Syarat/ }));
    expect(screen.getByText(/SEBAGAIMANA ADANYA/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: /Kembali/ }));
    await fireEvent.click(screen.getByRole("button", { name: /Lisensi/ }));
    expect(screen.getByText(/Proprietary/)).toBeTruthy();
    expect(screen.getByText(/All rights reserved/)).toBeTruthy();
  });
});
