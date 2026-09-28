# Kuron Studio

**App desktop untuk penerjemah (translator) manga dan manhwa.**

Import sejumlah halaman, aplikasi mencari balon speech secara otomatis, Anda edit, lalu kirim ke AI untuk diterjemahkan, lalu ekspor hasilnya jadi file siap pakai.

Dibuat dengan **Tauri v2 (Rust)**, antarmuka **Svelte 5**. Semua proses jalan di komputer sendiri. Kunci API disimpan di *keychain* sistem, tidak ada server di tengah.

- **Bundle ID:** `id.kuron.studio`
- **Platform:** Windows, macOS, Linux
- **Status:** MVP berjalan. Fase M0 sampai M4 selesai, M5 sebagian.

> English summary: a desktop workbench for manga and manhwa translation teams.
> Import pages, auto-detect speech bubbles on-device, translate using your own
> AI provider key, export to JSON, PNG, CBZ, or PSD. Not a reader, not a scraper.

---

## Isinya apa saja?

| Fitur | Penjelasan singkat |
|---|---|
| **Import batch** | Pilih satu folder berisi puluhan halaman. Format JPG, PNG, WebP, diurutkan nama file. |
| **Deteksi bubble** | Model ONNX jalan di komputer Anda sendiri, tidak dikirim ke internet. Di bawah 2 detik per halaman. |
| **Editor bubble** | Geser, ubah ukuran, ubah bentuk (persegi, oval, bebas), gambar *tail* arah dialog. |
| **Terjemahkan** | 9 provider AI (OpenAI, Gemini, Cohere, OpenRouter, dan lainnya). Kunci API Anda sendiri (*BYOK*). |
| **Glossary** | Daftar istilah tetap untuk nama karakter dan singkatan. Konsisten di semua chapter. |
| **Batch dan review** | Terjemahkan banyak halaman sekaligus. Satu halaman gagal tidak menggagalkan yang lain. |
| **Ekspor** | JSON (untuk integrasi app lain), PNG dengan teks tertanam, CBZ (zip), PSD (layer per bubble). |

**Bukan** aplikasi baca manga, bukan scraper, bukan hosting model. Fokusnya **peralatan kerja translator**.

---

## Cara menjalankan

### Yang perlu dipasang lebih dulu

| Perlu | Versi | Link |
|---|---|---|
| Node.js | 22 | <https://nodejs.org/> |
| pnpm | terbaru | <https://pnpm.io/installation> |
| Rust | stable | <https://rustup.rs/> |

Linux saja perlu paket sistem tambahan:
`libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

### Jalankan

```bash
cd kuron-studio
pnpm install

pnpm tauri dev      # mode pengembangan
```

> **Penting:** pakai `pnpm tauri dev`, **bukan** `pnpm dev`.
> `pnpm dev` hanya menyalakan antarmuka tanpa backend Rust, jadi tombol-tombol
> yang memanggil backend akan gagal.

Kalau hasilnya terasa aneh karena bundel lama masih tersimpan, bersihkan dulu:

```bash
pnpm dev:fresh        # atau: pnpm tauri dev -- --nuke
```

### Bikin installer

```bash
pnpm tauri build      # menghasilkan .msi, .dmg, atau .deb
```

Detail konfigurasi, catatan rilis, dan alur kerja aplikasi ada di
[`kuron-studio/README.md`](kuron-studio/README.md).

---

## Sebelum push

Semua perintah ini **wajib hijau**, sama persis dengan yang dicek CI:

```bash
cd kuron-studio/src-tauri
cargo test && cargo clippy --all-targets -- -D warnings

cd ../..
pnpm test && pnpm check
```

Cara menjalankan satu test saja:

```bash
cd kuron-studio/src-tauri && cargo test translate_mock       # backend
cd kuron-studio && pnpm vitest run src/lib/batch.test.ts      # frontend
```

---

## Struktur folder

```
Kuron-studio/
├── README.md            # dokumen ini
├── AGENTS.md            # aturan kerja untuk AI assistant
├── MEMORY.md            # catatan lintas sesi
├── kuron-studio/        # kodenya (antarmuka + backend)
├── openspec/            # spesifikasi dan rencana kerja
└── design/              # file desain
```

Dua folder yang paling sering dibuka:

- **`kuron-studio/src/`** — antarmuka Svelte, token warna, teks multibahasa.
- **`kuron-studio/src-tauri/src/`** — logika Rust: deteksi bubble, penerjemah, ekspor.

Kunci API dan data proyek disimpan di folder data aplikasi pada sistem
operasional, **tidak pernah** di dalam repo.

---

## Untuk developer dan AI assistant

Baca [`AGENTS.md`](AGENTS.md) sebelum mengubah kode. Isinya:

- perintah mana yang wajib dijalankan setelah perubahan apa,
- logika harus ditaruh di frontend atau backend,
- skill **`taste`** — aturan agar hasil desain dan UI tidak terlihat seperti
  keluaran generator AI (gradient, emoji sebagai ikon, tombol generik, dan
  sejenisnya).

Spesifikasi perubahan kerja ada di `openspec/`, dibaca dan ditulis lewat
perintah `/opsx:*`.

---

## Melaporkan bug dan meminta fitur

Lewat GitHub Issues, pakai template yang sudah disiapkan:

- **[Laporkan bug](.github/ISSUE_TEMPLATE/bug_report.yml)** — isi template, lampirkan screenshot, tulis langkah reproduksi.
- **[Minta fitur](.github/ISSUE_TEMPLATE/feature_request.yml)** — jelaskan masalah yang Anda hadapi, bukan hanya solusi yang ada di kepala Anda.

Template GitHub bisa diisi langsung lewat browser, jadi tidak perlu menyunting file.

## Kontribusi

Silakan fork dan kirim pull request. Jalankan gate di atas sebelum membuka PR.
Lisensi belum ditetapkan.
