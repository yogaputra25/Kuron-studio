export type PageStatus =
  | "idle"
  | "detecting"
  | "detected"
  | "noBubbles"
  | "translating"
  | "translated"
  | "failed";

export interface Page {
  id: string;
  path: string;
  width: number;
  height: number;
  status: PageStatus;
}

export interface Project {
  id: string;
  name: string;
  pages: Page[];
}

export interface ImportResult {
  pages: Page[];
  skipped: number;
}
