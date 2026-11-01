import { describe, expect, it } from "vitest";
import { matchShortcut } from "./shortcuts";

const mod = { metaKey: true, ctrlKey: false, altKey: false, shiftKey: false };
const ctrl = { metaKey: false, ctrlKey: true, altKey: false, shiftKey: false };

describe("matchShortcut", () => {
  it("Cmd/Ctrl + tombol -> aksi", () => {
    expect(matchShortcut({ ...mod, key: "," })).toBe("providers");
    expect(matchShortcut({ ...ctrl, key: "b" })).toBe("batch");
    expect(matchShortcut({ ...mod, key: "E" })).toBe("results");
    expect(matchShortcut({ ...mod, key: "d" })).toBe("detect");
    expect(matchShortcut({ ...mod, key: "o" })).toBe("import-files");
  });

  it("tanpa mod / dengan Alt/Shift / tombol asing -> null", () => {
    expect(matchShortcut({ ...mod, key: "g", metaKey: false, ctrlKey: false })).toBeNull();
    expect(matchShortcut({ ...mod, key: "," , altKey: true })).toBeNull();
    expect(matchShortcut({ ...mod, key: "b", shiftKey: true })).toBeNull();
    expect(matchShortcut({ ...mod, key: "x" })).toBeNull();
  });
});
