import type { ChapterFolder, Page, PageStatus, Project } from "../contracts/types";
import { sortPagesByName } from "./status";

export const DEFAULT_FOLDER_NAME = "default";

/** Kunci sortir grid/list (satu sumber; dulu dideklarasi ulang di App). */
export type SortKey = "name" | "status" | "bubbles";
/** Mode tampilan grid vs list (persist localStorage di App). */
export type PageView = "grid" | "list";
/** Filter kanban + semua. */
export type KanbanFilter = "all" | "todo" | "doing" | "done" | "failed";

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

/** Anak langsung sebuah folder. */
export function childrenOf(project: Project, folderId: string): ChapterFolder[] {
  return project.folders.filter((f) => f.parentId === folderId);
}

/** True bila ada folder bersarang (punya parent). */
export function hasNesting(project: Project): boolean {
  return project.folders.some((f) => f.parentId != null && f.parentId !== "");
}

/** Rantai root → … → parent → folder itu sendiri. Korup (siklus) diputus. */
export function folderPath(project: Project, folderId: string): ChapterFolder[] {
  const byId = new Map(project.folders.map((f) => [f.id, f]));
  const chain: ChapterFolder[] = [];
  const seen = new Set<string>();
  let cur = byId.get(folderId);
  while (cur && !seen.has(cur.id)) {
    seen.add(cur.id);
    chain.unshift(cur);
    cur = cur.parentId ? (byId.get(cur.parentId) ?? undefined) : undefined;
  }
  return chain;
}

/** Semua halaman subtree (folder + keturunan), urut nama file. */
export function subtreePages(project: Project, folderId: string): Page[] {
  const ids = new Set<string>([folderId]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const f of project.folders) {
      if (f.parentId != null && f.parentId !== "" && ids.has(f.parentId) && !ids.has(f.id)) {
        ids.add(f.id);
        grew = true;
      }
    }
  }
  return sortPagesByName(project.folders.filter((f) => ids.has(f.id)).flatMap((f) => f.pages));
}

function statsOf(pages: Page[]): FolderStats {
  const total = pages.length;
  const k = groupKanban(pages);
  return { total, done: k.done.length, failed: k.failed.length, todo: k.todo.length, doing: k.doing.length, progress: total === 0 ? 0 : k.done.length / total };
}

/** Agregat subtree (folder + keturunan). */
export function subtreeStats(project: Project, folderId: string): FolderStats {
  return statsOf(subtreePages(project, folderId));
}

/** True bila `name` sudah dipakai saudara sekandung (parent sama). Cermin aturan backend. */
export function isSiblingNameTaken(folders: ChapterFolder[], folderId: string, name: string): boolean {
  const self = folders.find((f) => f.id === folderId);
  if (!self) return false;
  const want = name.trim();
  if (!want) return true;
  return folders.some((f) => f.id !== folderId && f.parentId === self.parentId && f.name === want);
}

/** Id dalam rentang anchor→klik (inklusif) menurut urutan tampil. Tanpa anchor = klik saja. */
export function rangeSelect(orderedIds: string[], anchorId: string | null, clickedId: string): string[] {
  if (!anchorId) return [clickedId];
  const a = orderedIds.indexOf(anchorId);
  const b = orderedIds.indexOf(clickedId);
  if (a === -1 || b === -1) return [clickedId];
  const [lo, hi] = a < b ? [a, b] : [b, a];
  return orderedIds.slice(lo, hi + 1);
}

/** Folder langsung tempat SEMUA pageIds berada; null bila kosong/tak ketemu/tersebar.
 *  Dipakai dialog pindah: folder asal ditampilkan disabled (tak bisa jadi tujuan). */
export function singleOwnerFolderId(folders: ChapterFolder[], pageIds: string[]): string | null {
  if (pageIds.length === 0) return null;
  const wanted = new Set(pageIds);
  let owner: string | null = null;
  for (const f of folders) {
    if (f.pages.some((p) => wanted.has(p.id))) {
      if (owner !== null) return null;
      owner = f.id;
    }
  }
  return owner;
}

/** Satu baris chip di folder bar: folder + stat + varian tampil. */
export interface FolderChipSpec {
  folder: ChapterFolder;
  stats: FolderStats;
  active: boolean;
  dashed: boolean;
  canDelete: boolean;
}

/** Varian daftar chip: `crumb` (jalur breadcrumb), `children` (anak, gaya dashed), `flat` (tanpa nesting). */
export type FolderChipList = "crumb" | "children" | "flat";

/**
 * Spesifikasi chip per baris untuk `FolderBar`: ganti 3 blok `<FolderChip>`
 * yang dulu ditulis manual (beda `active`/`dashed`/`canDelete`).
 */
export function chipSpecs(
  project: Project,
  kind: FolderChipList,
  folders: ChapterFolder[],
  folderId: string | "all",
): FolderChipSpec[] {
  const total = project.folders.length;
  return folders.map((f, i) => {
    const stats = subtreeStats(project, f.id);
    switch (kind) {
      case "crumb":
        return { folder: f, stats, active: i === folders.length - 1, dashed: false, canDelete: total > 1 };
      case "children":
        return { folder: f, stats, active: false, dashed: true, canDelete: false };
      case "flat":
        return { folder: f, stats, active: f.id === folderId, dashed: false, canDelete: total > 1 };
    }
  });
}
