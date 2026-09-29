// @vitest-environment jsdom
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { compile } from "svelte/compiler";
import { flushSync, mount, unmount } from "svelte";
import { describe, expect, it } from "vitest";
import FormHarness from "./__fixtures__/FormHarness.svelte";

// BUG SNAPSHOT 2026-09-28: field "Nama" terisi "kimjongun" di layar, tapi
// tombol Simpan tetap melempar "Nama wajib diisi.".
//
// Penyebabnya: Field menerima `value = $bindable()` lalu meneruskannya sebagai
// `{value}` — satu arah. Compiler Svelte menghasilkan
//     $.template_effect(() => $.set_value(input, value));   // tulis saja
// bukan
//     $.bind_value(input, value);                            // baca + tulis
// Jadi teks masuk ke DOM tapi TIDAK pernah kembali ke $state parent.
// Test ini mengetik sungguhan, bukan membaca source file.

function mountHarness() {
  const target = document.createElement("div");
  document.body.appendChild(target);
  const app = mount(FormHarness, { target });
  flushSync();
  return { target, app };
}

function cleanup(h: { target: HTMLElement; app: object }) {
  unmount(h.app);
  h.target.remove();
}

function type(input: HTMLInputElement, text: string) {
  // Meniru input fisik: isi value lalu kirim event `input` (bukan change).
  input.value = text;
  input.dispatchEvent(new Event("input", { bubbles: true }));
  flushSync();
}

function pick(sel: HTMLSelectElement, value: string) {
  sel.value = value;
  sel.dispatchEvent(new Event("change", { bubbles: true }));
  flushSync();
}

describe("Field round-trip: ketikan sampai ke $state parent", () => {
  it("state parent mengikuti ketikan (bukan cuma DOM)", () => {
    const h = mountHarness();
    const target = h.target;
    const input = target.querySelector('[data-testid="name"]') as HTMLInputElement;

    type(input, "kimjongun");

    // DOM sudah berisi — ini yang membuat screenshot terlihat "sudah terisi".
    expect(input.value).toBe("kimjongun");
    // Yang penting: $state di baliknya ikut berubah — bukan cuma DOM.
    expect(
      (target.querySelector('[data-testid="state"]') as HTMLElement).textContent,
      "state parent tidak mengikuti ketikan",
    ).toBe("kimjongun");
    cleanup(h);
  });

  it("reproduksi screenshot: terisi -> Simpan TIDAK error", () => {
    const h = mountHarness();
    const target = h.target;
    const input = target.querySelector('[data-testid="name"]') as HTMLInputElement;

    type(input, "kimjongun");
    (target.querySelector('[data-testid="save"]') as HTMLButtonElement).click();
    flushSync();

    const err = target.querySelector('[data-testid="err"]') as HTMLElement;
    expect(
      err.textContent,
      "bug screenshot terulang: field terisi tapi divonis kosong",
    ).toBe("");
    expect(
      (target.querySelector('[data-testid="out"]') as HTMLElement).textContent,
    ).toContain("kimjongun");
    cleanup(h);
  });

  it("masih menolak nama yang benar-benar kosong", () => {
    const h = mountHarness();
    const target = h.target;
    (target.querySelector('[data-testid="save"]') as HTMLButtonElement).click();
    flushSync();

    expect(
      (target.querySelector('[data-testid="err"]') as HTMLElement).textContent,
    ).toBe("Nama wajib diisi.");
    cleanup(h);
  });

  it("kosongkan lagi lalu isi ulang — error lama tidak menempel", () => {
    const h = mountHarness();
    const target = h.target;
    const input = target.querySelector('[data-testid="name"]') as HTMLInputElement;

    type(input, "");
    (target.querySelector('[data-testid="save"]') as HTMLButtonElement).click();
    flushSync();
    expect(
      (target.querySelector('[data-testid="err"]') as HTMLElement).textContent,
    ).not.toBe("");

    type(input, "tes");
    flushSync();
    expect(
      (target.querySelector('[data-testid="err"]') as HTMLElement).textContent,
    ).toBe("");
    cleanup(h);
  });
});

describe("Select round-trip: pilihan sampai ke $state parent", () => {
  it("memilih opsi mengubah state parent", () => {
    const h = mountHarness();
    const target = h.target;
    const sel = target.querySelectorAll("select")[0] as HTMLSelectElement;

    type(target.querySelector('[data-testid="name"]') as HTMLInputElement, "a");
    pick(sel, "openai");
    (target.querySelector('[data-testid="save"]') as HTMLButtonElement).click();
    flushSync();

    expect(
      (target.querySelector('[data-testid="out"]') as HTMLElement).textContent,
    ).toContain('"providerType":"openai"');
    cleanup(h);
  });
});

describe("mekanisme: compiler harus menghasilkan bind_value", () => {
  it("Field.svelte meng-compile menjadi $.bind_value", () => {
    const src = readFileSync(
      join(process.cwd(), "src/lib/ui/Field.svelte"),
      "utf8",
    );
    const { js } = compile(src, { generate: "client" });
    expect(
      js.code,
      "tanpa bind_value input jadi satu arah — ini akar bug screenshot",
    ).toContain("bind_value");
  });

  it("Select.svelte meng-compile menjadi $.bind_value", () => {
    const src = readFileSync(
      join(process.cwd(), "src/lib/ui/Select.svelte"),
      "utf8",
    );
    const { js } = compile(src, { generate: "client" });
    expect(
      js.code,
      "<select> memakai bind_select_value — tanpa itu pilihan tidak balik ke parent",
    ).toMatch(/bind_select_value|bind_value/);
  });

  it("bukti: {value} TIDAK menghasilkan bind_value (mutasi harus merah)", () => {
    const { js } = compile(
      '<script>let { value = $bindable() } = $props();</script><input {value} />',
      { generate: "client" },
    );
    expect(js.code).not.toContain("bind_value");
    expect(js.code).toContain("set_value");
  });
});
