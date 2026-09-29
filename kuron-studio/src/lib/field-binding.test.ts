import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("..", import.meta.url));

// Regresi: `Field` menerima `value = $bindable()` tapi tidak mengikatnya dua
// arah ke <input>. Dua kegagalan yang pernah terjadi, dua-duanya bikin input
// "uncontrolled" — ketikan TERLIHAT di layar tapi state parent tetap "":
//   1. tidak menulis apa-apa sama sekali;
//   2. menulis `{value}` — satu arah, compiler menghasilkan set_value tanpa
//      bind_value. Gejalanya persis: "Nama wajib diisi." walau field terisi.
//
// Test ini memastikan `bind:value`; bukti runtime ada di field-roundtrip.test.ts.

describe("ui/Field meneruskan value ke input (controlled)", () => {
  const field = readFileSync(join(SRC, "lib/ui/Field.svelte"), "utf8");

  it("menerima value sebagai $bindable", () => {
    expect(field).toMatch(/value = \$bindable\(\)/);
  });

  it("mengikat bind:value di <input> (dua arah)", () => {
    // Wajib bind, bukan `{value}` — yang terakhir cuma one-way dan tidak
    // pernah mengembalikan ketikan user ke $state.
    const inputTag = field.slice(field.indexOf("<input"));
    expect(
      inputTag.slice(0, 400),
      "value tidak terikat dua arah — field jadi uncontrolled",
    ).toMatch(/bind:value/);
  });

  it("bind:value ditulis SETELAH {...rest} agar tidak ditimpa", () => {
    // PENTING: `indexOf(">")` di SELURUH file akan menemukan `>` dari tag
    // <label> yang muncul lebih dulu — slice jadi kosong. Ambil `>` relatif
    // terhadap posisi <input.
    const from = field.indexOf("<input");
    const inputTag = field.slice(from, field.indexOf("/>", from) + 2);
    const restAt = inputTag.indexOf("{...rest}");
    const valueAt = inputTag.indexOf("bind:value");
    expect(restAt).toBeGreaterThan(-1);
    expect(valueAt, "bind:value harus ada").toBeGreaterThan(-1);
    expect(
      valueAt,
      "bind:value harus setelah {...rest} — kalau sebelum, rest bisa menimpanya",
    ).toBeGreaterThan(restAt);
  });
});

describe("primitif lain juga meneruskan value", () => {
  it("ui/Select mengikat bind:value di <select>", () => {
    const sel = readFileSync(join(SRC, "lib/ui/Select.svelte"), "utf8");
    const tag = sel.slice(sel.indexOf("<select"));
    expect(tag.slice(0, 400)).toMatch(/bind:value/);
  });
});

describe("tidak ada primitif yang menerima bindable tapi membuangnya", () => {
  const files = readdirSync(join(SRC, "lib/ui"));
  it.each(files.filter((f) => f.endsWith(".svelte")))("%s konsisten", (f) => {
    const src = readFileSync(join(SRC, "lib/ui", f), "utf8");
    const acceptsBindable = /(\w+) = \$bindable\(\)/.test(src);
    if (!acceptsBindable) return;
    for (const [, name] of src.matchAll(/(\w+) = \$bindable\(\)/g)) {
      expect(
        src,
        `${f}: menerima ${name} = $bindable() tapi tidak bind:${name} di markup`,
      ).toContain(`bind:${name}`);
    }
  });
});
