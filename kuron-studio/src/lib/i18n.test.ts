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
  "emptyHintMobile",
  "batchTitle",
  "parallel",
  "failed",
  "retryFailed",
  "running",
  "exportJson",
  "exportPng",
  "exportCbz",
  "refresh",
  "cleanImages",
  "deleteProject",
  "confirmDeleteProject",
  "confirmClean",
  "cleanDone",
  "nothingToClean",
  "done",
  "pages",
  "bubbles",
  "original",
  "reading",
  "translated",
  "add",
  "edit",
  "save",
  "remove",
  "export",
  "share",
  "search",
  "collapseSidebar",
  "expandSidebar",
  "sheetHandle",
  "shrinkSheet",
  "dashboard",
  "openProject",
  "backToLibrary",
  "newFolder",
  "folderName",
  "allFolders",
  "kanbanTodo",
  "kanbanDoing",
  "kanbanDone",
  "kanbanFailed",
  "remainingTodo",
  "lastWorked",
  "deleteFolder",
  "confirmDeleteFolder",
  "moveToFolder",
];

const LANGS: Lang[] = ["en", "id", "zh"];

describe.each(LANGS)("i18n %s", (l: Lang) => {
  it("covers every UI key without empty strings", () => {
    for (const k of KEYS) {
      const v = tr(l, k);
      expect(v.length).toBeGreaterThan(0);
    }
  });
});
