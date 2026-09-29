import { describe, expect, it } from "vitest";
import projectRs from "../../src-tauri/src/commands/project.rs?raw";
import { ARCHIVE_EXTS, IMAGE_EXTS } from "./types";

// Cegah dialog picker dan backend_import menerima daftar format berbeda.
// Kalau melebar, user bisa pilih file yang import-nya ditolak diam-diam;
// kalau menyempit, file valid jadi tidak bisa dipilih sama sekali.
//
// Rust adalah sumber kebenaran (crate `image` yang menentukan apa yang bisa
// di-decode), jadi test ini membandingkan TS → Rust, bukan sebaliknya.
// Bentuk sumber dibaca via `?raw` supaya tidak perlu parser.

const RUST_IMAGE_EXTS = /const IMAGE_EXTS: \[&str; \d+\] = \[([^\]]*)\]/.exec(projectRs);
const RUST_ARCHIVE_EXTS = /matches!\(ext\.to_ascii_lowercase\(\)\.as_str\(\), ([^)]*)\)/.exec(
  projectRs,
);

function literalList(raw: string): string[] {
  return [...raw.matchAll(/"([^"]+)"/g)].map((m) => m[1]);
}

describe("image/archive extension contract (TS picker == Rust import filter)", () => {
  it("parses the Rust constants", () => {
    expect(RUST_IMAGE_EXTS, "IMAGE_EXTS not found in project.rs").not.toBeNull();
    expect(RUST_ARCHIVE_EXTS, "is_archive ext list not found in project.rs").not.toBeNull();
  });

  it("IMAGE_EXTS matches", () => {
    expect([...IMAGE_EXTS].sort()).toEqual(literalList(RUST_IMAGE_EXTS![1]).sort());
  });

  it("ARCHIVE_EXTS matches", () => {
    expect([...ARCHIVE_EXTS].sort()).toEqual(literalList(RUST_ARCHIVE_EXTS![1]).sort());
  });

  it("no duplicates", () => {
    for (const list of [IMAGE_EXTS, ARCHIVE_EXTS]) {
      expect(new Set(list).size, `duplikat di ${list.join(",")}`).toBe(list.length);
    }
  });
});
