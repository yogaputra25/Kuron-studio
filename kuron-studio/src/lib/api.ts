import { invoke } from "@tauri-apps/api/core";
import type { ImportResult, Project } from "./types";

// NOTE: invoke keys must match Rust param names exactly (snake_case).
export const api = {
  listProjects: () => invoke<Project[]>("list_projects"),
  createProject: (name: string) => invoke<Project>("create_project", { name }),
  getProject: (project_id: string) => invoke<Project>("get_project", { project_id }),
  importPages: (project_id: string, paths: string[]) =>
    invoke<ImportResult>("import_pages", { project_id, paths }),
  getImagePreview: (path: string, max_side = 512) =>
    invoke<string>("get_image_preview", { path, max_side }),
};
