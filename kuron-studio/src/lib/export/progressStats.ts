import type { Project } from "../contracts/types";
import { allPages } from "../library/library";

/** Beda vs library.projectStats (kanban-level per folder): ini page+bubble-level lintas folder untuk header progres export. */
export function computeStats(project: Project): {
  done: number;
  failed: number;
  pending: number;
  total: number;
  pct: number;
  bubblePct: number;
} {
  const pages = allPages(project);
  const done = pages.filter((p) => p.status === "translated").length;
  const failed = pages.filter((p) => p.status === "failed").length;
  const pending = pages.length - done - failed;
  const totalBubbles = pages.reduce((n, p) => n + p.bubbles.length, 0);
  const translatedBubbles = pages.reduce(
    (n, p) => n + (p.translation?.bubbles.filter((b) => b.translated).length ?? 0),
    0,
  );
  return {
    done,
    failed,
    pending,
    total: pages.length,
    pct: pages.length ? Math.round((done / pages.length) * 100) : 0,
    bubblePct: totalBubbles ? Math.round((translatedBubbles / totalBubbles) * 100) : 0,
  };
}
