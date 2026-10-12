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

Semua perintah di bawah dijalankan dari folder `kuron-studio/` (yang berisi `package.json` dan `src-tauri/`), kecuali ditulis lain.

### 1. Pasang prasyarat (sekali saja)

| Perlu | Versi | Link |
|---|---|---|
| Node.js | 22 | <https://nodejs.org/> |
| pnpm | 11 (CI pakai 11.9.0) | <https://pnpm.io/installation> |
| Rust | stable | <https://rustup.rs/> |

Linux saja, pasang paket sistem dulu (sesuai CI, Ubuntu 24.04):

```bash
sudo apt-get update && sudo apt-get install -y pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev libayatana-appindicator3-dev patchelf
```

> Jangan pakai `libappindicator3-dev` (nama lama, sudah tidak ada di Ubuntu 24.04).

macOS: Xcode penuh sudah cukup (CLT ikut di dalam). Kalau belum ada:

```bash
xcode-select --install
```

### 1b. iOS (opsional, hanya kalau target iOS)

Butuh Xcode penuh dari App Store (bukan CLT saja), lalu:

```bash
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
brew install cocoapods
```

Xcode tidak membaca PATH nvm/corepack, jadi symlink `pnpm-native` ke folder yang sudah ada di PATH Xcode (`~/.local/bin`, tanpa sudo):

```bash
ln -sf ~/.cache/node/corepack/v1/pnpm/*/pnpm-native ~/.local/bin/pnpm-native
pnpm-native --version
```

Jalanin pertama kali:

```bash
cd kuron-studio
pnpm ios:init        # alias: pnpm tauri ios init (generate src-tauri/gen/apple/)
```

### Cara running di iOS Simulator:

> **Kenapa `pnpm tauri ios dev` error `ARCHIVE FAILED` / `No Account for Team`?**
> CLI Tauri default mencoba melakukan `xcodebuild archive` ke device fisik (`-sdk iphoneos`), yang membutuhkan akun Apple Developer berbayar dan provisioning profile. Di Simulator, archiving ini tidak diperlukan dan pasti gagal jika akun belum disetel di Xcode.

Gunakan salah satu cara ini untuk Simulator:

**Cara A (1 perintah dari terminal — langsung build, install, & buka di simulator):**
```bash
cd kuron-studio
pnpm ios:run:sim
```
*Perintah ini mem-build bundle simulator tanpa code signing, lalu otomatis meng-install dan me-launch app ke simulator yang sedang aktif.*

**Cara B (Development dengan hot-reload via Xcode):**
```bash
cd kuron-studio
pnpm ios:open        # menyalakan dev server dan membuka project di Xcode
```
Di Xcode: pilih simulator tujuan di toolbar atas (mis. **iPhone 17**), lalu klik **Run (Cmd+R)**.

### Cara running di iPhone fisik:
1. Buka Xcode → **Settings > Accounts** → login Apple ID Anda.
2. Buka project di Xcode (`pnpm ios:open`), pilih target **kuron-studio_iOS** → tab **Signing & Capabilities**.
3. Centang **Automatically manage signing** dan pilih **Team** akun Anda (Personal Team gratis bisa dipakai).
4. Colokkan iPhone, pilih device Anda di Xcode, lalu klik **Run (Cmd+R)**.

Catatan iOS di repo ini:
- `src-tauri/tauri.ios.conf.json`: `minimumSystemVersion 15.0` (default CLI 14.0 ditolak Xcode 27+).
- `vite.config.ts`: bind `0.0.0.0:1420` agar dev server bisa diakses perangkat di LAN.
- Capability `default` dibatasi `macOS/windows/linux`, capability `mobile` untuk `iOS/android` (tanpa plugin updater/process yang desktop-only).
- `Cargo.toml`: dependencies aplikasi bersifat umum untuk semua target.

### Kalau `tauri dev` gagal `cargo metadata: No such file or directory`

Artinya `cargo` tidak di-PATH. Cek shell dulu, lalu pasang permanen:

```bash
echo $SHELL            # /bin/zsh atau /bin/bash
cargo --version        # error = belum di-PATH
```

zsh (default macOS):

```bash
grep -q 'cargo/env' ~/.zshrc || echo '. "$HOME/.cargo/env"' >> ~/.zshrc
. "$HOME/.cargo/env"
cargo --version
```

bash:

```bash
grep -q 'cargo/env' ~/.bashrc || echo '. "$HOME/.cargo/env"' >> ~/.bashrc
. "$HOME/.cargo/env"
cargo --version
```

> Installer Rust kadang nulis ke shell yang salah (mis. `.bashrc` padahal shell `zsh`). Terminal baru baca otomatis, terminal lama butuh source manual sekali.

### 2. Install dependency (sekali saja, atau tiap pull)

```bash
cd kuron-studio
pnpm install
```

### 3. Jalanin app (tiap hari)

```bash
cd kuron-studio
pnpm tauri dev
```

Ini membuka window desktop + frontend di `http://localhost:1420`. `pnpm tauri dev` otomatis menyalakan Vite (`beforeDevCommand: pnpm dev`), jadi jangan jalankan manual.

> Pakai `pnpm tauri dev`, **bukan** `pnpm dev`.
> `pnpm dev` (Vite saja) tidak menyalakan backend Rust, jadi semua tombol yang manggil backend gagal.

### 4. Kalau tampilan ngaco / cache nyangkut

```bash
cd kuron-studio
pnpm dev:fresh            # hapus cache Vite (node_modules/.vite), lalu jalanin tauri dev
pnpm dev:fresh -- --nuke  # sama + reset profil WebView (state webview hilang, data proyek aman)
```

`dev:fresh` abort kalau port 1420 kepakai (matikan Vite zombie dulu). Setelah webview naik, tekan `Ctrl+Shift+R` (hard reload).

### 5. Bikin installer

```bash
cd kuron-studio
pnpm tauri build      # hasil: .msi (Windows), .dmg (macOS), .deb (Linux) di src-tauri/target/release/bundle/
```

Detail konfigurasi, catatan rilis, dan alur kerja aplikasi ada di
[`kuron-studio/README.md`](kuron-studio/README.md).

---

## Sebelum push

Semua perintah ini **wajib hijau**, sama persis dengan yang dicek CI:

```bash
cd kuron-studio/src-tauri
cargo test && cargo clippy --all-targets -- -D warnings

cd ..
pnpm test && pnpm check
```

Cara menjalankan satu test saja:

```bash
cd kuron-studio/src-tauri && cargo test drops_low_confidence   # backend (nama test asli, mis. bubble::tests::drops_low_confidence)
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
