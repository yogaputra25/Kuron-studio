import { describe, expect, it } from "vitest";
import {
  allPages,
  childrenOf,
  folderPath,
  folderStats,
  groupKanban,
  hasNesting,
  isSiblingNameTaken,
  isSimpleProject,
  kanbanBucket,
  projectStats,
  rangeSelect,
  singleOwnerFolderId,
  subtreePages,
  subtreeStats,
} from "./library";
import type { Page, Project } from "./types";

const pg = (id: string, status: Page["status"]): Page => ({
  id,
  path: `${id}.jpg`,
  width: 0,
  height: 0,
  status,
  bubbles: [],
});

const proj = (): Project => ({
  id: "p",
  name: "Judul",
  folders: [
    { id: "f1", name: "default", pages: [pg("a", "translated"), pg("b", "idle")] },
    { id: "f2", name: "Ch 12", pages: [pg("c", "failed"), pg("d", "translating")] },
  ],
});

describe("kanbanBucket", () => {
  it("maps detected/noBubbles to todo, detecting/translating to doing", () => {
    expect(kanbanBucket("idle")).toBe("todo");
    expect(kanbanBucket("detected")).toBe("todo");
    expect(kanbanBucket("noBubbles")).toBe("todo");
    expect(kanbanBucket("detecting")).toBe("doing");
    expect(kanbanBucket("translating")).toBe("doing");
    expect(kanbanBucket("translated")).toBe("done");
    expect(kanbanBucket("failed")).toBe("failed");
  });
});

describe("folderStats", () => {
  it("aggregates done/failed/todo + progress", () => {
    const s = folderStats(proj().folders[0]);
    expect(s).toMatchObject({ total: 2, done: 1, todo: 1, failed: 0 });
    expect(s.progress).toBeCloseTo(0.5);
  });

  it("is 0 progress when empty", () => {
    expect(folderStats({ id: "x", name: "y", pages: [] }).progress).toBe(0);
  });
});

describe("projectStats", () => {
  it("sums across folders", () => {
    expect(projectStats(proj())).toMatchObject({ total: 4, done: 1, failed: 1, todo: 1, doing: 1 });
  });

  it("falls back to legacy flat pages", () => {
    const legacy = { id: "p", name: "L", folders: [], pages: [pg("a", "translated")] } as Project;
    expect(allPages(legacy)).toHaveLength(1);
    expect(projectStats(legacy).progress).toBe(1);
  });
});

describe("groupKanban", () => {
  it("sorts pages by filename inside buckets", () => {
    const k = groupKanban([pg("p10", "idle"), pg("p2", "idle")]);
    expect(k.todo.map((p) => p.id)).toEqual(["p2", "p10"]);
  });
});

describe("isSimpleProject", () => {
  it("is simple with 0-1 folders", () => {
    expect(isSimpleProject(proj())).toBe(false);
    expect(isSimpleProject({ id: "p", name: "S", folders: [{ id: "f", name: "default", pages: [] }] })).toBe(
      true,
    );
  });
});

const tree = (): Project => ({
  id: "p",
  name: "T",
  folders: [
    { id: "r", name: "Judul", pages: [pg("r1", "idle")] },
    { id: "c", name: "Ch 12", parentId: "r", pages: [pg("c1", "translated")] },
    { id: "b", name: "Batch 1", parentId: "c", pages: [pg("b1", "failed"), pg("b2", "idle")] },
  ],
});

describe("folder tree helpers", () => {
  it("detects nesting and lists children", () => {
    expect(hasNesting(tree())).toBe(true);
    expect(hasNesting(proj())).toBe(false);
    expect(childrenOf(tree(), "c").map((f) => f.id)).toEqual(["b"]);
    expect(childrenOf(tree(), "b")).toEqual([]);
  });

  it("builds root-first path and cuts cycles", () => {
    expect(folderPath(tree(), "b").map((f) => f.id)).toEqual(["r", "c", "b"]);
    expect(folderPath(tree(), "r").map((f) => f.id)).toEqual(["r"]);
    const cyclic = tree();
    cyclic.folders.find((f) => f.id === "r")!.parentId = "b";
    expect(folderPath(cyclic, "b").map((f) => f.id)).toEqual(["r", "c", "b"]);
  });

  it("aggregates subtree pages sorted", () => {
    expect(subtreePages(tree(), "c").map((p) => p.id)).toEqual(["b1", "b2", "c1"]);
    expect(subtreeStats(tree(), "c")).toMatchObject({ total: 3, done: 1, failed: 1 });
    expect(subtreeStats(tree(), "r").total).toBe(4);
  });

  it("rejects sibling duplicates but allows same name under other parents", () => {
    const t = tree();
    // "Batch 1" (id b) vs rename "Ch 12" (id c, parent r) menjadi "Batch 1":
    // beda parent (r vs c) → boleh.
    expect(isSiblingNameTaken(t.folders, "c", "Batch 1")).toBe(false);
    // Tambah saudara "Batch 1" di bawah c → rename b jadi "Batch 1" tetap miliknya sendiri.
    expect(isSiblingNameTaken(t.folders, "b", "Batch 1")).toBe(false);
    t.folders.push({ id: "b2", name: "Batch 2", parentId: "c", pages: [] });
    expect(isSiblingNameTaken(t.folders, "b2", "Batch 1")).toBe(true);
    expect(isSiblingNameTaken(t.folders, "b2", "  Batch 1  ")).toBe(true);
    expect(isSiblingNameTaken(t.folders, "b2", "Batch X")).toBe(false);
    expect(isSiblingNameTaken(t.folders, "b2", "   ")).toBe(true);
  });
});

describe("rangeSelect", () => {
  const ids = ["p1", "p2", "p10"];
  it("returns click alone without anchor", () => {
    expect(rangeSelect(ids, null, "p2")).toEqual(["p2"]);
  });
  it("expands forward and backward inclusively", () => {
    expect(rangeSelect(ids, "p1", "p10")).toEqual(ids);
    expect(rangeSelect(ids, "p10", "p1")).toEqual(ids);
    expect(rangeSelect(ids, "p2", "p2")).toEqual(["p2"]);
  });
  it("falls back to click when anchor unknown", () => {
    expect(rangeSelect(ids, "zx", "p2")).toEqual(["p2"]);
  });
});


describe("singleOwnerFolderId", () => {
  const folders: Record<string, string[]> = { f1: ["a", "b"], f2: ["c"] };
  const mk = () =>
    Object.entries(folders).map(([id, ids]) => ({
      id,
      name: id,
      pages: ids.map((x) => pg(x, "idle")),
    }));
  it("returns the folder when all pages live there", () => {
    expect(singleOwnerFolderId(mk(), ["a", "b"])).toBe("f1");
    expect(singleOwnerFolderId(mk(), ["c"])).toBe("f2");
  });
  it("returns null when empty, unknown, or spread across folders", () => {
    expect(singleOwnerFolderId(mk(), [])).toBeNull();
    expect(singleOwnerFolderId(mk(), ["zz"])).toBeNull();
    expect(singleOwnerFolderId(mk(), ["a", "c"])).toBeNull();
  });
});
