/**
 * Cermin project (FE-2): daftar project + seleksi current + loaded.
 * Diekstrak dari `App.svelte` — satu-satunya pemilik `projects`.
 * Bentuk factory (D1): panggil sekali di komposisi, jangan destructure
 * state reaktifnya (baca lewat getter agar reaktivitas tidak putus).
 */
import { api } from "../contracts/api";
import type { Project } from "../contracts/types";

export interface ProjectStore {
  readonly projects: Project[];
  readonly currentId: string | null;
  readonly loaded: boolean;
  readonly current: Project | null;
  setCurrent(id: string | null): void;
  replaceProject(id: string, patch: Project): void;
  removeProject(id: string): void;
  addProject(p: Project): void;
  refresh(): Promise<void>;
  refreshProject(id: string): Promise<void>;
}

export function createProjectStore(): ProjectStore {
  let projects = $state<Project[]>([]);
  let currentId = $state<string | null>(null);
  let loaded = $state(false);
  // Guard generasi (pola yang sama dengan loadGen/detectGen di App):
  // refresh tumpang-tindih — yang lama selesai belakangan dibuang.
  let gen = 0;

  return {
    get projects() {
      return projects;
    },
    get currentId() {
      return currentId;
    },
    get loaded() {
      return loaded;
    },
    get current() {
      return projects.find((p) => p.id === currentId) ?? null;
    },
    setCurrent(id: string | null) {
      currentId = id;
    },
    replaceProject(id: string, patch: Project) {
      projects = projects.map((p) => (p.id === id ? patch : p));
    },
    removeProject(id: string) {
      projects = projects.filter((p) => p.id !== id);
      if (currentId === id) currentId = projects[0]?.id ?? null;
    },
    addProject(p: Project) {
      projects = [...projects, p];
      currentId = p.id;
    },
    async refresh() {
      const my = ++gen;
      try {
        const list = await api.listProjects();
        if (my !== gen) return;
        projects = list;
        if (!currentId && projects.length > 0) currentId = projects[0].id;
      } finally {
        if (my === gen) loaded = true;
      }
    },
    async refreshProject(id: string) {
      const updated = await api.getProject(id);
      projects = projects.map((p) => (p.id === id ? updated : p));
    },
  };
}
