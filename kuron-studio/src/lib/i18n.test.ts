import { describe, expect, it } from "vitest";
import { tr, type Lang } from "./i18n";

const KEYS = [
  "providers",
  "batch",
  "glossary",
  "review",
  "detectAll",
  "importFolder",
  "importFiles",
  "close",
  "newProject",
  "projectName",
  "create",
  "emptyHint",
  "batchTitle",
  "parallel",
  "failed",
  "retryFailed",
  "running",
  "exportJson",
  "exportPng",
  "exportCbz",
  "refresh",
] as const;

const LANGS: Lang[] = ["en", "id", "zh"];

describe.each(LANGS)("i18n %s", (l: Lang) => {
  it("covers every UI key without empty strings", () => {
    for (const k of KEYS) {
      const v = tr(l, k);
      expect(v.length).toBeGreaterThan(0);
    }
  });
});
