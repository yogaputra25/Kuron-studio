// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent, screen } from "@testing-library/svelte";
import { renderWithI18n } from "../../test-utils";
import AppHeader from "./AppHeader.svelte";

function props(narrow: boolean) {
  return {
    current: null,
    projects: [],
    busy: false,
    readingDir: "rtl" as const,
    lastInvokeError: null,
    brokenCount: 0,
    canDetect: false,
    narrow,
    onToggleReadingDir: vi.fn(),
    onProviders: vi.fn(),
    onBatch: vi.fn(),
    onGlossary: vi.fn(),
    onResults: vi.fn(),
    onDetect: vi.fn(),
    onImportFolder: vi.fn(),
    onImportFiles: vi.fn(),
    onClean: vi.fn(),
    onDelete: vi.fn(),
    onOpenDiagnostics: vi.fn(),
    onAbout: vi.fn(),
    onPickProject: vi.fn(),
  };
}

describe("AppHeader lebar vs sempit", () => {
  it("lebar: bar penuh tanpa tombol menu, About memanggil onAbout", async () => {
    const p = props(false);
    renderWithI18n(AppHeader, { props: p });
    expect(screen.queryByRole("button", { name: "Buka menu" })).toBeNull();
    expect(screen.getByRole("button", { name: "Providers" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Tentang" }));
    expect(p.onAbout).toHaveBeenCalledOnce();
  });

  it("sempit: tombol menu + Detect, aksi di drawer, Esc menutup", async () => {
    const p = props(true);
    const { container } = renderWithI18n(AppHeader, { props: p });
    expect(screen.getByRole("button", { name: "Buka menu" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Detect semua" })).toBeTruthy();
    // Bar penuh disembunyikan sampai drawer dibuka.
    expect(screen.queryByRole("button", { name: "Glossary" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Buka menu" }));
    const dialog = screen.getByRole("dialog", { name: "Buka menu" });
    expect(dialog).toBeTruthy();
    expect(screen.getByRole("button", { name: "Glossary" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Tentang" })).toBeTruthy();
    await fireEvent.keyDown(window, { key: "Escape" });
    expect(container.querySelector('[role="dialog"]')).toBeNull();
  });

  it("sempit: klik overlay menutup drawer", async () => {
    const { container } = renderWithI18n(AppHeader, { props: props(true) });
    await fireEvent.click(screen.getByRole("button", { name: "Buka menu" }));
    expect(screen.getByRole("dialog", { name: "Buka menu" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Tutup menu" }));
    expect(container.querySelector('[role="dialog"]')).toBeNull();
  });

  it("sempit: import expand + baris pengaturan ada", async () => {
    renderWithI18n(AppHeader, { props: props(true) });
    await fireEvent.click(screen.getByRole("button", { name: "Buka menu" }));
    await fireEvent.click(screen.getByRole("button", { name: /Import/ }));
    expect(screen.getByRole("button", { name: /Import files/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Urutan baca/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Bahasa/ })).toBeTruthy();
  });
});
