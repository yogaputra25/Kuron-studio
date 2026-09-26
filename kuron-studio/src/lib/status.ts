import type { Page, PageStatus } from "./types";

const badge: Record<PageStatus, string> = {
  idle: "bg-zinc-500",
  detecting: "bg-amber-500",
  detected: "bg-sky-500",
  noBubbles: "bg-slate-500",
  translating: "bg-violet-500",
  translated: "bg-emerald-600",
  failed: "bg-rose-600",
};

export const statusBadgeClass = (s: PageStatus): string => badge[s];

/** Filename order, natural-ish via localeCompare numeric. */
export function sortPagesByName(pages: Page[]): Page[] {
  return [...pages].sort((a, b) =>
    a.path.localeCompare(b.path, undefined, { numeric: true }),
  );
}
