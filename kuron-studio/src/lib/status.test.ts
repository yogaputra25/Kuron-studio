import { describe, expect, it } from "vitest";
import { sortPagesByName, statusBadgeClass } from "./status";
import type { Page } from "./types";

const page = (path: string): Page => ({ id: path, path, width: 0, height: 0, status: "idle", bubbles: [] });

describe("statusBadgeClass", () => {
  it("maps every status to a badge", () => {
    for (const s of ["idle", "detecting", "detected", "noBubbles", "translating", "translated", "failed"] as const) {
      expect(statusBadgeClass(s)).toMatch(/^bg-/);
    }
  });
});

describe("sortPagesByName", () => {
  it("orders numerically by filename", () => {
    const out = sortPagesByName([page("p10.jpg"), page("p2.jpg"), page("p1.jpg")]);
    expect(out.map((p) => p.path)).toEqual(["p1.jpg", "p2.jpg", "p10.jpg"]);
  });
});
