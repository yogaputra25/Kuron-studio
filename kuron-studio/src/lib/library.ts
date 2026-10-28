import type { ChapterFolder, Page, PageStatus, Project } from "./types";
import { sortPagesByName } from "./status";

export const DEFAULT_FOLDER_NAME = "default";

export interface FolderStats {
  total: number;
  done: number;
  failed: number;
  todo: number;
  doing: number;
  progress: number;
}

export interface Kanban {
  todo: Page[];
  doing: Page[];
  done: Page[];
  failed: Page[];
}

/** Kanban mapping: Belum=idles, Proses=jalan, Sudah=translated, Gagal=failed. */
export function kanbanBucket(s: PageStatus): keyof Kanban {
  switch (s) {
    case "translated":
      return "done";
    case "failed":
      return "failed";
    case "detecting":
    case "translating":
      return "doing";
    default:
      return "todo";
  }
}

export function groupKanban(pages: Page[]): Kanban {
  const out: Kanban = { todo: [], doing: [], done: [], failed: [] };
  for (const pg of sortPagesByName(pages)) out[kanbanBucket(pg.status)].push(pg);
  return out;
}

export function folderStats(folder: ChapterFolder): FolderStats {
  const total = folder.pages.length;
  const k = groupKanban(folder.pages);
  const done = k.done.length;
  const failed = k.failed.length;
  const doing = k.doing.length;
  const todo = k.todo.length;
  return { total, done, failed, todo, doing, progress: total === 0 ? 0 : done / total };
}

/** Semua halaman project lintas folder (kompat: fallback `pages` flat lama). */
export function allPages(project: Project): Page[] {
  if (project.folders.length > 0) return project.folders.flatMap((f) => f.pages);
  return project.pages ?? [];
}

export function projectStats(project: Project): FolderStats {
  const pages = allPages(project);
  const total = pages.length;
  const k = groupKanban(pages);
  const done = k.done.length;
  return {
    total,
    done,
    failed: k.failed.length,
    doing: k.doing.length,
    todo: k.todo.length,
    progress: total === 0 ? 0 : done / total,
  };
}

/** Mode A (simple) = tepat 1 folder `default` → UI boleh skip picker folder. */
export function isSimpleProject(project: Project): boolean {
  return project.folders.length <= 1;
}

export function folderById(project: Project, folderId: string | null): ChapterFolder | null {
  if (!folderId) return project.folders[0] ?? null;
  return project.folders.find((f) => f.id === folderId) ?? null;
}
