import { describe, expect, it, vi } from "vitest";
import { createProjectStore } from "./projectStore.svelte.ts";
import type { Project } from "../contracts/types";

vi.mock("../contracts/api", () => ({
  api: {
    listProjects: vi.fn(),
    getProject: vi.fn(),
  },
}));

import { api } from "../contracts/api";

const mk = (id: string, name = id): Project => ({ id, name }) as unknown as Project;

describe("projectStore", () => {
  it("replaceProject hanya menukar entri target", () => {
    const s = createProjectStore();
    s.addProject(mk("a"));
    s.addProject(mk("b"));
    const b2 = mk("b", "baru");
    s.replaceProject("b", b2);
    expect(s.projects.map((p) => p.name)).toEqual(["a", "baru"]);
    expect(s.projects[1]).toBe(b2);
  });

  it("removeProject membuang current dan fallback ke pertama", () => {
    const s = createProjectStore();
    s.addProject(mk("a"));
    s.addProject(mk("b"));
    expect(s.currentId).toBe("b");
    s.removeProject("b");
    expect(s.projects.map((p) => p.id)).toEqual(["a"]);
    expect(s.currentId).toBe("a");
    s.removeProject("a");
    expect(s.projects).toEqual([]);
    expect(s.currentId).toBeNull();
  });

  it("current mengikuti currentId", () => {
    const s = createProjectStore();
    expect(s.current).toBeNull();
    s.addProject(mk("a"));
    s.addProject(mk("b"));
    s.setCurrent("a");
    expect(s.current?.id).toBe("a");
    s.setCurrent("tak-ada");
    expect(s.current).toBeNull();
  });

  it("refresh basi dibuang, generasi terbaru yang menang", async () => {
    let resolveFirst!: (v: Project[]) => void;
    const lama = [mk("lama")];
    const baru = [mk("baru")];
    vi.mocked(api.listProjects)
      .mockImplementationOnce(() => new Promise<Project[]>((r) => (resolveFirst = r)))
      .mockImplementationOnce(async () => baru);
    const s = createProjectStore();
    const first = s.refresh();
    await s.refresh();
    resolveFirst(lama);
    await first;
    expect(s.projects.map((p) => p.id)).toEqual(["baru"]);
    expect(s.loaded).toBe(true);
  });

  it("refreshProject menukar hasil getProject", async () => {
    const updated = mk("a", "diperbarui");
    vi.mocked(api.getProject).mockResolvedValue(updated);
    const s = createProjectStore();
    s.addProject(mk("a"));
    s.addProject(mk("b"));
    await s.refreshProject("a");
    expect(s.projects.map((p) => p.name)).toEqual(["diperbarui", "b"]);
    expect(vi.mocked(api.getProject)).toHaveBeenCalledWith("a");
  });
});
