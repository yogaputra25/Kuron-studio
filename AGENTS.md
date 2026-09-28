# AGENTS.md — Kuron Studio

Aturan kerja untuk AI assistant (Claude Code, OpenCode, dan sejenisnya) yang
mengubah repo ini. Baca sebelum menulis kode.

Dokumen ini selalu terbaca di tiap percakapan. Jaga tetap ringkas. Detail yang
jarang dipakai ditaruh di file terpisah dan ditunjuk dari sini.

---

## 1. Peta repo

```
kuron-studio/            # working directory untuk semua perintah
├── src/                 # Svelte 5 + Tailwind 4 (antarmuka)
│   ├── lib/             # komponen + logika murni (bubble.ts, batch.ts, i18n.ts)
│   ├── lib/ui/          # komponen dasar (Button, Field, Select, Panel, StatusBadge)
│   └── lib/*.test.ts    # test vitest, satu file per topik
├── src-tauri/src/       # Rust (backend)
│   ├── commands/        # #[tauri::command] — lapisan tipis, meneruskan ke lib di bawahnya
│   ├── provider/        # integrasi AI per provider (openai, gemini, cohere)
│   ├── image_ops.rs     # mosaic, compress, chunk webtoon
│   ├── detector.rs      # inferensi ONNX bubble
│   ├── translation.rs   # pipeline penerjemahan
│   └── secrets.rs       # keychain
├── src-tauri/tests/     # test integrasi mock (bukan unit)
└── scripts/             # dev-fresh.mjs
```

**Titik routing:** `src/lib/api.ts` adalah satu-satunya tempat frontend memanggil
backend. Backend dipanggil lewat `invoke("nama_command", {...})`.

---

## 2. Setiap perubahan wajib disertai ini

Tentukan dulu mana yang relevan, lalu jalankan. Semuanya harus hijau.

| Ubah apa | Jalankan |
|---|---|
| File Rust apa pun | `cd kuron-studio/src-tauri && cargo test && cargo clippy --all-targets -- -D warnings` |
| File Svelte/TS apa pun | `cd kuron-studio && pnpm test && pnpm check` |
| `src-tauri/src/commands/*` | Rust gate **dan** `pnpm test` — ada test kontrak yang memverifikasi nama command dan nama argumen |
| `src/lib/api.ts` atau `types.ts` | `pnpm test` (test kontrak invoke) **dan** gate Rust |
| Warna, tema, spacing, tipografi | `pnpm test` — `theme-discipline.test.ts` dan `spacing.test.ts` menangkap regresi token |

`pnpm check` = `svelte-check` dengan TypeScript strict. Error tipe bukan opsi.

Bila test gagal karena perubahan Anda, perbaiki kodenya. Jangan melemahkan test
supaya hijau.

---

## 3. Menaruh logika di tempat yang benar

**Backend (Rust)** bila salah satu berlaku: menyentuh file, model ONNX, gambar,
keychain, database SQLite, jaringan, atau bisa memblokir lebih dari ~100ms.
`image` crate, `ort`, `rusqlite`, `keyring`, `reqwest` semuanya di Rust.

**Frontend (Svelte/TS)** bila soal: state UI, layout, drag, preview, atau
penghitungan geometry yang sudah ada.

Aturan praktis:

- `commands/*.rs` **tidak** berisi logika bisnis. Kalau sebuah command lebih
  dari ~30 baris, pindahkan inti ke modul di `src/` dan biarkan command tipis
  sebagai adapter.
- Logika murni yang bisa diuji tanpa Tauri (`bubble.ts`, `batch.ts`,
  `translation.ts`) tetap di `src/lib/`, bukan di komponen `.svelte`.
- Kontrak nama command dan nama argumen antara Rust dan TypeScript dijaga test.
  Kalau Anda mengganti nama di satu sisi saja, test akan gagal — itu memang
  gunanya. Perbarui kedua sisi, atau lebih baik jangan diganti sama sekali.

---

## 4. Gaya penulisan

### Kode

- **Svelte 5 runes** (`$props()`, `$state`, `$derived`, `$effect`) dan **TypeScript
  strict**. Tidak ada `export let` lagi, tidak ada `any` tanpa alasan tertulis.
- **Rust**: `cargo clippy -D warnings` harus bersih. `unwrap()` dan `expect()`
  hanya di jalur yang secara invariant tidak mungkin gagal, dengan komentar yang
  menyebut invariant-nya.
- **Komentar** menjelaskan alasan, bukan mengulang apa yang dilakukan kode. Kalau
  komentar bisa dihapus tanpa mengubah apa pun yang dibaca, hapus.
- **Komentar `ponytail:`** menandai pemangkasan sadar yang punya plafon
  (`# ponytail: global lock; per-account lock bila throughput jadi masalah`).
  Kalau nanti plafonnya terlampaui, pindahkan ke ledger debt.

### Bahasa

- **Komentar kode dan pesan commit:** Bahasa Indonesia, istilah teknis boleh
  English. Ini mengikuti bahasa repo.
- **Teks antarmuka (UI):** Bahasa Indonesia sebagai default, dengan terjemahan
  English dan Mandarin di `src/lib/i18n.ts`. Kalau menambah string baru,
  tambahkan di ketiga bahasa dalam satu edit yang sama. Ada test
  (`i18n.test.ts`) yang gagal kalau ada kunci yang hilang di salah satu bahasa.

### Git

- Branch: `bugfix/<nomor>-deskripsi` atau `feat/<deskripsi>`. Contoh yang ada:
  `bugfix/1245-bugfix-ui`.
- Commit message Conventional Commits: `feat:`, `fix:`, `refactor:`, `test:`,
  `docs:`, `chore:`. Subjek singkat, tanpa titik di akhir.
- Jangan pernah commit kunci API, `.env`, atau data proyek. `*.onnx` sudah
  di-ignore kecuali `bubble.onnx` yang memang di-bundle.

---

## 5. Aturan desain dan UI

Proyek ini punya skill tersendiri: **`.claude/skills/taste/`**. Baca sebelum
menulis atau mengubah komponen visual, teks antarmuka, atau nama identifier.

Intinya satu: ini aplikasi kerja, bukan halaman pemasaran. Keputusan yang sudah
diambil dan tidak perlu diulang:

- Warna dan spacing datang dari token semantik di `src/app.css` (`--ks-*`).
  Jangan tulis palet mentah seperti `bg-zinc-800` atau `text-emerald-300` —
  itu tidak punya pasangan kontras di light theme. Test
  `theme-discipline.test.ts` akan menandainya.
- Arah visual: netral warm, aksen coral untuk aksi utama, cyan untuk hal
  teknis. Sudah ditentukan, tidak perlu ditawar ulang.
- Ikon bukan emoji. Label tombol menyebut aksi konkret, bukan "Submit".
- Typography: Inter untuk teks, Space Grotesk untuk display, JetBrains Mono
  untuk angka dan koordinat. Ketiganya sudah di-bundle offline.

---

## 6. Rencana kerja (OpenSpec)

Perubahan yang lebih besar dari perbaikan kecil dimulai dari spesifikasi, bukan
dari kode:

| Perintah | Untuk apa |
|---|---|
| `/opsx:propose` | Deskripsi perubahan, buat proposal, design, spec, tasks |
| `/opsx:apply` | Kerjakan task yang sudah ada di `openspec/changes/<nama>/tasks.md` |
| `/opsx:archive` | Tutup perubahan yang sudah selesai |
| `/opsx:explore` | Discussions sebelum memutuskan |

Task di `tasks.md` memakai checkbox `- [ ]` dan penanda prioritas P0, P1, P2,
P3. Selesai berarti checkbox di-centang **dan** gate bagian 2 hijau.

---

## 7. Hal yang perlu diketahui

- **`pnpm dev` bukan cara run yang benar.** Backend Rust tidak ikut jalan.
  Selalu `pnpm tauri dev`. Kalau tampilan visual tidak masuk akal, pakai
  `pnpm dev:fresh`.
- **`keyring` tanpa feature `apple-native` gagal diam-diam** di macOS:
  `set_password` tetap Ok, API key hilang tiap restart. Feature itu dikunci di
  `Cargo.toml` dengan komentar; jangan dihapus. Test `keychain_backend_is_real`
  menjaganya.
- **Konten pengguna-adjacent tidak pernah masuk ke log atau prompt tanpa
  dicrop.** Mosaic dibangun dengan padding 20% supaya konteks sekeliling
  terbaca, dan ada batas ukuran. Jangan longgarkan batas itu.
- **Model ONNX** (`bubble.onnx`, sekitar 12MB) di-bundle, bukan diunduh. Kalau
  `.gitignore`mu memberi tanda negate untuk file itu, biarkan.

---

## 8. Dokumen lain

| Dokumen | Isi |
|---|---|
| `README.md` | Gambaran umum untuk orang yang belum pernah lihat proyek ini |
| `MEMORY.md` | Catatan lintas sesi: keputusan, peta reuse dari Kuron App |
| `kuron-studio/README.md` | Detail app, alur kerja, catatan konfigurasi dan rilis |
| `design/kuron-studio.pen` | File desain (butuh pen.dev) |
| `openspec/config.yaml` | Aturan penulisan spesifikasi |

Issue bug dan permintaan fitur lewat GitHub Issues, template di
`.github/ISSUE_TEMPLATE/`. Jangan buat issue kosong — kedua template itu sudah
menanyakan bagian yang penting.
