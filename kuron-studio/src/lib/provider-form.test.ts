import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const SRC = fileURLToPath(new URL("..", import.meta.url));
const panel = readFileSync(join(SRC, "lib/ProviderSettings.svelte"), "utf8");

// Dua bug yang-reported user:
//
// 1. "Nama wajib diisi." nempel di banner atas_panel dan tidak pernah hilang —
//    user sudah memperbaiki/k menutup form tapi messagenya masih di sana.
// 2. "Model tidak langsung show" — list_models butuh record di DB (id),
//    jadi provider baru tidak bisa melihat pilihannya sebelum disimpan.
//    Chicken-and-egg: simpan dulu dengan model kosong, baru bisa load.

describe("provider form: error tidak lagi global yang nempel", () => {
  it("pesan nama wajib pakai konstanta yang sama di writer dan reader", () => {
    expect(panel).toContain('const NAME_REQUIRED = "Nama wajib diisi.";');
    // Writer (save) dan reader (nameError) harus referensi konstanta yang
    // sama — kalau ada string literal terpisah, mereka bisa melenceng diam-diam.
    // Hitung di kode saja, bukan di komentar.
    const code = panel
      .split("\n")
      .filter((l) => !l.trimStart().startsWith("//") && !l.trimStart().startsWith("*"))
      .join("\n");
    const literal = [...code.matchAll(/"Nama wajib diisi\."/g)];
    expect(
      literal.length,
      "string harus muncul sekali di kode (hanya di deklarasi konstanta)",
    ).toBe(1);
  });

  it("error validasi jadi per-field (nameError), bukan hanya banner", () => {
    expect(panel).toContain('const nameError = $derived(error === NAME_REQUIRED');
    expect(panel).toMatch(/label="Nama"[\s\S]*?error=\{nameError\}/);
  });

  it("error dibersihkan saat user mengetik di field", () => {
    const n = [...panel.matchAll(/oninput=\{\(\) => \(error = ""\)\}/g)].length;
    expect(n, "field tanpa oninput error-clearing").toBeGreaterThanOrEqual(3);
  });

  it("startNew() dan startEdit() membersihkan error", () => {
    for (const fn of ["function startNew", "function startEdit"]) {
      const body = panel.slice(panel.indexOf(fn), panel.indexOf(fn) + 400);
      expect(body, `${fn} harus clear error`).toContain('error = ""');
    }
  });

  it("banner global tidak lagi menampilkan error validasi field", () => {
    // Kalau banner masih menampilkannya, user lihat pesan yang sama 2x.
    const banner = panel.slice(panel.indexOf('role="alert"') - 300, panel.indexOf('role="alert"') + 300);
    expect(banner).toContain("error");
    expect(
      banner,
      "banner global jangan render error validasi per-field (duplikat)",
    ).not.toMatch(/\{error\}\s*<\/p>\s*<\/div>\s*<div[^>]*role="alert"/);
  });
});

describe("provider form: model bisa dipilih sebelum disimpan", () => {
  it("loadModels punya jalur draft, bukan hanya id", () => {
    const fn = panel.slice(panel.indexOf("async function loadModels"));
    expect(fn).toContain("api.listModelsDraft(");
    expect(fn).toContain("api.listModels(editId)");
  });

  it("tidak lagi menolak load models hanya karena belum ada editId", () => {
    const fn = panel.slice(panel.indexOf("async function loadModels"));
    expect(
      fn,
      "pesan 'simpan dulu' = chicken-and-egg yang dikeluhkan user",
    ).not.toContain("Simpan provider dulu sebelum load models");
  });

  it("gatanya基于 key, bukan berdasarkan sudah-tersimpan", () => {
    expect(panel).toContain("const canListModels = $derived(fKey.trim().length > 0 || !!editId);");
  });

  it("memberi alasan saat tombol disabled", () => {
    expect(panel).toMatch(/disabled=\{busy \|\| !canListModels\}/);
    expect(panel).toContain('title={canListModels ?');
    expect(panel).toContain("Isi API key dulu");
  });

  it("otomatis pilih model pertama agar field tidak kosong", () => {
    const fn = panel.slice(panel.indexOf("async function loadModels"));
    expect(fn).toContain("if (models.length > 0 && !fModel) fModel = models[0].id;");
  });

  it("select model Spiegelkan fModel, bukan selalu value=\"\"", () => {
    // value="" selalu membuat select возвращаться ke placeholder walau
    // user sudah memilih dari daftar.
    expect(panel).toMatch(/label="Model tersedia"[\s\S]*?value=\{fModel\}/);
  });

  it("model yang diketik manual tidak dihapus saat key berubah", () => {
    // oninput API key sengaja hanya mereset `models`, bukan `fModel`.
    const key = panel.slice(panel.indexOf('label="API key"'));
    const input = key.slice(0, key.indexOf("/>") + 2);
    expect(input).toContain("models = []");
    expect(input, "fModel tidak boleh di-reset").not.toContain("fModel = ");
  });
});

describe("provider form: key tersimpan ditampilkan tanpa membocorkan", () => {
  it("pakai flag hasStoredKey, bukan isi key", () => {
    expect(panel).toContain("let hasStoredKey = $state(false);");
    expect(panel).toContain("hasStoredKey = p.hasKey;");
    // Rust hanya menyimpan boolean hasKey — fKey tidak boleh di-backfill.
    const startEdit = panel.slice(panel.indexOf("function startEdit"), panel.indexOf("function startEdit") + 400);
    expect(startEdit).toMatch(/fKey = "";/);
  });

  it("hint menjelaskan bedanya 'tersimpan' vs 'kosong'", () => {
    expect(panel).toContain("Ada key tersimpan");
  });
});

describe("provider form: tombol Simpan tidak mati tanpa sebab", () => {
  it("tidak disabled hanya karena Nama kosong", () => {
    // Gejala yang dilaporkan: "Tombol Simpan mati/kelabu". Penyebabnya
    // `disabled={!fName.trim()}` — user tidak pernah diberi tahu alasannya.
    expect(panel, "tombol Simpan jangan disabled karena Nama kosong").not.toMatch(
      /variant="primary"[\s\S]{0,160}disabled=\{busy \|\| !fName\.trim\(\)\}/,
    );
  });

  it("tetap menolak submit kosong dengan pesan yang jelas", () => {
    const save = panel.slice(panel.indexOf("async function save"));
    expect(save).toContain("if (!fName.trim()) { error = NAME_REQUIRED; return; }");
  });

  it("memberi tooltip saat Nama kosong", () => {
    expect(panel).toContain('title={fName.trim() ? "" : "Isi nama provider dulu."}');
  });
});

describe("provider form: base_url tidak ditimpa diam-diam", () => {
  it("pickType hanya menimpa base_url yang masih pristine", () => {
    const fn = panel.slice(panel.indexOf("function pickType"));
    expect(fn, "harus cek apakah base_url masih nilai default").toMatch(
      /const wasPristine =/,
    );
    expect(fn).toContain("DEFAULT_BASE_URLS[fType]");
  });

  it("ganti tipe juga membersihkan model lama", () => {
    const fn = panel.slice(panel.indexOf("function pickType"));
    expect(fn, "model lama tidak berlaku untuk provider baru").toMatch(/fModel = ""/);
  });

  it("custom dikosongkan, preset diisi", () => {
    const fn = panel.slice(panel.indexOf("function pickType"));
    expect(fn).toContain('if (t === "custom")');
    expect(fn).toContain("fBaseUrl = DEFAULT_BASE_URLS[t];");
  });
});
