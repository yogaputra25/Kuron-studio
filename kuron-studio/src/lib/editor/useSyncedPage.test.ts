// Unit keputusan sinkron prop->lokal (FE-4, task 2.2). Murni, tanpa DOM.
import { describe, expect, it } from "vitest";
import { rev, shouldApplyBubbles, shouldApplyTranslation, structurallyEqual } from "./useSyncedPage.svelte";

describe("rev/structurallyEqual", () => {
  it("rev stabil untuk konten identik beda referensi", () => {
    expect(rev({ a: [1, 2] })).toBe(rev({ a: [1, 2] }));
    expect(structurallyEqual({ a: [1] }, { a: [1] })).toBe(true);
    expect(structurallyEqual({ a: [1] }, { a: [2] })).toBe(false);
  });

  it("rev aman untuk undefined/fungsi (tanpa lempar)", () => {
    expect(rev(undefined)).toBe("");
    expect(structurallyEqual(undefined, null)).toBe(false);
  });
});

describe("shouldApplyBubbles", () => {
  it("save identik tak reset dirty: konten sama + tenang = jangan terapkan", () => {
    const bubbles = [{ x: 1 }];
    // Echo onSaved: objek baru, konten identik, halaman sama, tak sibuk.
    expect(
      shouldApplyBubbles({ samePage: true, localBusy: false, incoming: [{ x: 1 }], current: bubbles }),
    ).toBe(false);
  });

  it("ganti halaman selalu resync walau sibuk", () => {
    expect(
      shouldApplyBubbles({ samePage: false, localBusy: true, incoming: [], current: [{ x: 1 }] }),
    ).toBe(true);
  });

  it("sibuk lokal menahan timpaan konten beda", () => {
    expect(
      shouldApplyBubbles({ samePage: true, localBusy: true, incoming: [{ x: 2 }], current: [{ x: 1 }] }),
    ).toBe(false);
  });

  it("tenang + beda struktur = terapkan", () => {
    expect(
      shouldApplyBubbles({ samePage: true, localBusy: false, incoming: [{ x: 2 }], current: [{ x: 1 }] }),
    ).toBe(true);
  });
});

describe("shouldApplyTranslation", () => {
  it("prop basi tak timpa lokal yang lebih baru saat mengetik", () => {
    const current = { bubbles: [{ translated: "baru" }] };
    expect(
      shouldApplyTranslation({ busyTyping: true, incomingRev: rev({ bubbles: [{ translated: "lama" }] }), current }),
    ).toBe(false);
  });

  it("rev kosong (tanpa terjemahan masuk) = jangan terapkan", () => {
    expect(shouldApplyTranslation({ busyTyping: false, incomingRev: "", current: null })).toBe(false);
  });

  it("tenang + beda rev = terapkan", () => {
    const incoming = { bubbles: [{ translated: "AI" }] };
    expect(
      shouldApplyTranslation({ busyTyping: false, incomingRev: rev(incoming), current: null }),
    ).toBe(true);
  });
});
