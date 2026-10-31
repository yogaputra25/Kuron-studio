import { describe, expect, it } from "vitest";
import { ARCHIVE_EXTS, IMAGE_EXTS, PDF_EXTS } from "./types";
import extensions from "./extensions.json";
import dialogsSrc from "../app/dialogs.ts?raw";

// Kontrak lintas bahasa, sumber = `extensions.json` — TANPA membaca source
// Rust (tidak ada `?raw` ke `.rs`, tidak ada path `src-tauri/`):
// - TS (`types.ts`) mengimpor JSON langsung (bukan salinan),
// - Rust (`commands/project.rs`) membaca JSON yang sama via `include_str!` +
//   `OnceLock` (dipin test Rust `extensions_match_json_contract` +
//   `image_format_support_is_real`),
// - test ini memastikan pemakaian TS tetap sinkron dengan JSON.

describe("image/archive extension contract (TS picker == JSON)", () => {
  it("JSON memuat daftar yang diharapkan", () => {
    expect([...extensions.image].sort()).toEqual([
      "bmp",
      "gif",
      "jfif",
      "jpe",
      "jpeg",
      "jpg",
      "png",
      "tif",
      "tiff",
      "webp",
    ]);
    expect([...extensions.archive]).toEqual(["zip", "cbz"]);
    expect([...extensions.pdf]).toEqual(["pdf"]);
  });

  it("IMAGE_EXTS/ARCHIVE_EXTS/PDF_EXTS == JSON", () => {
    expect([...IMAGE_EXTS]).toEqual([...extensions.image]);
    expect([...ARCHIVE_EXTS]).toEqual([...extensions.archive]);
    expect([...PDF_EXTS]).toEqual([...extensions.pdf]);
  });

  it("no duplicates", () => {
    for (const list of [IMAGE_EXTS, ARCHIVE_EXTS, PDF_EXTS]) {
      expect(new Set(list).size, `duplikat di ${list.join(",")}`).toBe(list.length);
    }
  });

  it("pdf-import: PDF_EXTS terpisah, tak masuk IMAGE_EXTS; dialog tawarkan PDF", () => {
    // Backend kenali pdf via is_pdf() terpisah (output = jpg/png) — bentuknya
    // dijaga key `pdf` tersendiri di JSON + test Rust
    // `pdf_recognized_but_not_image_or_archive`.
    expect([...PDF_EXTS]).toEqual(["pdf"]);
    expect([...IMAGE_EXTS]).not.toContain("pdf");
    // Dialog tawarkan filter PDF documents (sejak FE-7: di dialogs.ts).
    expect(dialogsSrc).toContain("PDF documents");
  });
});
