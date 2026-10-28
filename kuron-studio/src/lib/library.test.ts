import { describe, expect, it } from "vitest";
import {
  allPages,
  folderStats,
  groupKanban,
  isSimpleProject,
  kanbanBucket,
  projectStats,
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
