// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { fireEvent, screen } from "@testing-library/svelte";
import { renderWithI18n } from "../../test-utils";
import AppHeader from "./AppHeader.svelte";
import { checkForUpdate } from "../updater";

vi.mock("../updater", () => ({
  checkForUpdate: vi.fn(),
  relaunchApp: vi.fn(),
}));

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
    onUpdateToast: vi.fn(),
  };
}

describe("AppHeader lebar vs sempit", () => {
  it("lebar: bar penuh tanpa tombol menu, About via menu ⋯ memanggil onAbout", async () => {
    const p = props(false);
    renderWithI18n(AppHeader, { props: p });
    expect(screen.queryByRole("button", { name: "Buka menu" })).toBeNull();
    expect(screen.getByRole("button", { name: "Providers" })).toBeTruthy();
    // Aksi jarang (About, hapus, dsb) tinggal di menu ⋯, bukan bar.
    expect(screen.queryByRole("menuitem", { name: "Tentang" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Aksi lainnya" }));
    await fireEvent.click(screen.getByRole("menuitem", { name: "Tentang" }));
    expect(p.onAbout).toHaveBeenCalledOnce();
  });

  it("sempit: bar hanya ☰ (tanpa Detect ganda), Detect + update sekali di drawer", async () => {
    const p = props(true);
    const { container } = renderWithI18n(AppHeader, { props: p });
    expect(screen.getByRole("button", { name: "Buka menu" })).toBeTruthy();
    // Bar sempit: hanya hamburger. CTA Detect pindah ke drawer (tanpa ganda).
    expect(screen.queryByRole("button", { name: "Detect semua" })).toBeNull();
    // Bar penuh disembunyikan sampai drawer dibuka.
    expect(screen.queryByRole("button", { name: "Glossary" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Buka menu" }));
    const dialog = screen.getByRole("dialog", { name: "Buka menu" });
    expect(dialog).toBeTruthy();
    expect(screen.getByRole("button", { name: "Glossary" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Tentang" })).toBeTruthy();
    // Satu-satunya Detect ada di drawer.
    expect(screen.getAllByRole("button", { name: "Detect semua" })).toHaveLength(1);
    // Baris "Cek update" ada di drawer (desktop): klik → toast + drawer tutup.
    vi.mocked(checkForUpdate).mockResolvedValueOnce(null);
    await fireEvent.click(screen.getByRole("button", { name: "Cek update" }));
    await vi.waitFor(() =>
      expect(p.onUpdateToast).toHaveBeenCalledWith("Sudah versi terbaru.", "success"),
    );
    expect(container.querySelector('[role="dialog"]')).toBeNull();
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

  it("lebar: switcher + toolbar satu baris space-between (41 pages … RTL)", () => {
    const p = {
      ...props(false),
      current: { id: "p1", name: "Dou", folders: [] },
      projects: [{ id: "p1", name: "Dou", folders: [] }],
    };
    const { container } = renderWithI18n(AppHeader, { props: p });
    const rows = container.querySelectorAll("header > div");
    const bar = rows[1] as HTMLElement | undefined;
    expect(bar).toBeTruthy();
    // Flutter spaceBetween: switcher kiri, toolbar kanan, ruang bebas di tengah.
    expect(bar!.className).toContain("justify-between");
    expect(bar!.className).not.toContain("flex-wrap");
    // Hitung halaman dan tombol RTL satu baris yang sama.
    expect(bar!.textContent ?? "").toContain("0 halaman");
    expect(bar!.contains(screen.getByText("0 halaman"))).toBe(true);
    expect(bar!.contains(screen.getByRole("button", { name: "RTL→" }))).toBe(true);
  });

  it("lebar: klik ↻ teruskan hasil ke onUpdateToast (toast global)", async () => {
    vi.mocked(checkForUpdate).mockResolvedValueOnce(null);
    const p = props(false);
    renderWithI18n(AppHeader, { props: p });
    await fireEvent.click(screen.getByRole("button", { name: "Cek update" }));
    await vi.waitFor(() =>
      expect(p.onUpdateToast).toHaveBeenCalledWith("Sudah versi terbaru.", "success"),
    );
  });
});
