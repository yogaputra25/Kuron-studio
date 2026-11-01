# Log sesi Kuron Studio

Append-only: entri baru selalu di **bawah**, tidak pernah edit/sisip di entri lama.
`merge=union` di `.gitattributes` membuat append paralel dari branch berbeda
bergabung otomatis tanpa konflik. Kalau >200 baris, ringkas entri tertua jadi
satu paragraf dan hapus yang diringkas (kompaksi = satu-satunya edit yang boleh
menyentuh baris lama).

## 2026-09-26 — Implementasi awal (rekonstruksi dari git history)

- Kerja: Implementasi awal Kuron Studio (`777aed0`).
- Hasil: Pondasi Tauri v2 + Svelte 5 jalan (import, deteksi ONNX, translate, ekspor).

## 2026-09-29 — Bugfix UI + in-bubble edit (rekonstruksi dari git history)

- Kerja: PR #1 (binding Field/Select, overlay terjemahan, token desain, logging diagnostik) dan PR #2 (design system + diagnostics + in-bubble edit, draft-local, last-write-wins, reset-translation cancel). `9c8263c` tulis panduan AGENTS.md + MEMORY.md pertama.
- Hasil: Keduanya MERGED. Editor bisa edit langsung di bubble.

## 2026-10-10 — Reset-translation dihapus, cancel per-page (rekonstruksi dari git history)

- Kerja: PR #3 — typing comfort + pane switching (`dadda1c`); hapus fitur reset-translation, ganti cancel per-page yang tidak menimpa edit user + jaga backward compatibility (`ea7387a`).
- Hasil: MERGED. Edit user tidak lagi tertimpa baseline AI.
- Keputusan: translasi tidak menimpa edit user; cancel tanpa invoke AI ulang.

## 2026-10-12 — PDF import + iOS picker (rekonstruksi dari git history)

- Kerja: PDF import + feedback drawing tools (`63117c8`); iOS document picker + normalisasi path (`2731c3f`); openspec skills diperkaya (`5740cac`).
- Hasil: Masuk main. Import terima PDF; iOS bisa pilih dokumen.

## 2026-10-18 — Bubble edit + draw gestures + export wrap (rekonstruksi dari git history)

- Kerja: PR #4 bubble bisa diedit (`3573782`); PR #5 isolasi gesture draw-tool + reset tool setelah draw (`9507a7e`); PR #6 selaraskan layout teks canvas vs ekspor + wrapping (`61a0e1b`).
- Hasil: Ketiganya MERGED.

## 2026-10-28 — Gelombang redesign: tablet, perf, folder, onboarding (rekonstruksi dari git history)

- Kerja: PR #9/#10 sidebar resizable + tablet mode (tutup #8); PR #13 docs arsitektur + aset redesign; PR #15/#18 redesign UI + dropdown project + referensi desain; PR #16 antrean thumbnail viewport + cache preview Rust 12/15 (tutup #11); PR #19 nested folders + move dialog + onboarding (sebagian #7); PR #20 busy-state onboarding; PR #21 motion polish.
- Hasil: Semua MERGED ke main hari yang sama. Bukti visual v1 di `design/references-v1/` (19 screenshot + temuan: Escape tak tutup panel, detect tanpa progres, batch disabled).
- Lanjut: issue terbuka #7 (sisa), #12, #14, #17.

## 2026-10-29 — Setup AGENTS.md + MEMORY.md + DESIGN.md

- Kerja: Setup `AGENTS.md` (aturan) + `MEMORY.md` (pekerjaan); tulis `DESIGN.md` (token `--ks-*`, peran coral/cyan, 6 pola layout), tautkan dari `AGENTS.md`.
- Hasil: Selesai, gate tidak tersentuh.
- Keputusan: `AGENTS.md` = aturan, `MEMORY.md` + log = pekerjaan; riwayat awalnya per-file lalu digabung ke log ini + `merge=union`.

## 2026-10-29 — Backlog diluruskan dari GitHub issues

- Kerja: Rekonsiliasi dengan `gh issue list` + `git log`. Koreksi: #8 CLOSED (bukan aktif), #7 OPEN (bukan selesai).
- Hasil: Selesai, kode tidak tersentuh.
- Keputusan: backlog = cermin GitHub issues yang OPEN; issue ditutup → coret di `MEMORY.md`.
- Lanjut: kerjakan issue terbuka #17, #14, #12, sisa #7.

## 2026-10-29 — Aturan no-secret + log jadi satu file

- Kerja: Gabung 9 file `memory/` ke `memory/log.md` + `.gitattributes merge=union` (terbukti auto-merge). Scan docs: bersih, tanpa secret nyata.
- Hasil: Selesai, kode tidak tersentuh.
- Keputusan: secret (API key, password, signing key, isi keychain/`.env`) tidak pernah ke repo termasuk log/rules; temuan di output → `[REDACTED]` (aturan no. 8 di `AGENTS.md`).
- Lanjut: —

## 2026-10-29 — detect-ux-pack 15/17 di atas mainline baru (isu #17)

- Kerja: Implementasi overlay progres Detect (`DetectProgress.svelte` + `reduceDetectProgress` + `listen detect_progress` + guard `detectGen`), backend lanjut-saat-gagal (`collect_batch` + event `ok/message`, test `gagal_tengah_lanjut_dan_urutan_utuh`), 1 tombol Import + menu pola switcher, header dua baris (`icon.svg` + tagline). Base pindah tengah jalan (main merge PR #16/#19–#22 ke `fa26334`); diff diverifikasi tanpa revert + gates diulang di base baru.
- Hasil: `pnpm test` 395/395, `pnpm check` 0 error, `cargo test` 114 lulus, `clippy -D warnings` bersih.
- Keputusan: tombol overlay = "Tutup" (dismiss, bukan cancel — backend tak punya cancel-flag); transport event diperkaya `ok/message` tanpa ubah wire invoke; `move_pages` main yang sempat tertelan resolusi dikembalikan verbatim (compiler menangkap).
- Lanjut: smoke `tauri dev` 25 halaman (1.5) + E2E manual isu #17 (5.4).

## 2026-10-29 — detect-ux-pack 17/17: smoke + E2E manual lulus (isu #17)

- Kerja: Verifikasi manual oleh user (25 halaman, counter + filename + Tutup tengah jalan, 1 file rusak, mobile/sempit).
- Hasil: LULUS. Change 17/17 complete. Siap archive.
- Lanjut: `/opsx:archive` untuk detect-ux-pack.

## 2026-10-29 — Arsip massal 15 change + sync 15 spec baru (opsi 1)

- Kerja: Sync 15 main spec baru (verbatim ADDED + TBD Purpose) + 3 merge MODIFIED ke `kuron-studio` (batch import viewport/cache; 2 skenario manual-edit baru — sisanya sudah tercakup main). Perbaikan normatif: 4 badan before-after-split + 2 preview-transport tanpa SHALL/MUST. Arsip 15 change ke `archive/2026-10-29-*`. Validasi akhir: 20/20 spec strict hijau, tanpa delta header tersisa.
- Hasil: Semua change terarsip. `openspec/changes/` tinggal `archive/` (26 entri). MAIN SPECS bertambah 15.
- Catatan: `kuron-studio-mvp` tanpa delta spec → arsip langsung. Purpose 15 spec baru = TBD, perbarui menyusul.
- Lanjut: commit + push (kode detect-ux-pack + grid-preview-perf masih uncommitted di worktree).

## 2026-10-29 — page-list-view 11/13 di worktree (isu #12, opsi A frontend saja)

- Kerja: Helper sortir (`sortPagesByStatus/Bubbles`, `fileName`, `parentFolder`) + test; `PageGrid` (ekstrak + prefix folder) + `PageList` (sortir, keyboard j/k/Enter/Space, roving focus, skeleton) + `pageView.test`; switcher Grid/List (`tablist`, G/L, pref localStorage, scroll per mode); retarget 4 test pola-sumber yang pindah file (motion, panel-tablet, previewGrid, select-mode).
- Hasil: `pnpm test` 405/405, `pnpm check` 0 error, `cargo test` 114 lulus, clippy bersih. Main maju PR #23 (merge grid-preview-perf, tanpa diff konten) — tanpa konflik.
- Keputusan: kolom ukuran = dimensi `WxH` (tanpa byte, tanpa backend) — opsi A sesuai pilihan user.
- Lanjut: smoke keyboard-only + narrow viewport (3.3), E2E 52/200 halaman (4.3) — butuh GUI + data nyata.

## 2026-10-29 — Arsip grid-sort 9/9 + sync spec (isu #25)

- Kerja: Sync `openspec/specs/grid-sort/spec.md` baru (3 requirement ADDED verbatim; Purpose TBD). Arsip ke `archive/2026-10-29-grid-sort/`.
- Hasil: Valid strict hijau. Semua artifact done, semua task complete.
- Lanjut: —

## 2026-10-29 — export-scope 10/11 (filter folder + ekspor terpilih + hapus tab QA/TM)

- Kerja: Filter folder di Hasil & Ekspor + `folder_id` diteruskan; `page_ids` opsional di `export_project` + `select_pages` tunggal (skip terhitung, urutan input); scope "Semua/Folder/Yang dipilih" + hint; QA/TM dihapus dari panel (backend utuh); i18n `exportScope*` 3 bahasa.
- Hasil: `pnpm test` 405/405, `pnpm check` 0, `cargo test` 116 lulus (termasuk `select_pages_subset_skips_missing` + `pages_in_selection_order_and_skips_missing`), clippy bersih. Kontrak invoke diperbarui (tambah `page_ids`).
- Lanjut: smoke 3.3 (455 baris → filter folder → ekspor folder/terpilih/skip; mobile).

## 2026-10-29 — export-scope: breadcrumb folder ganda (temuan user, 455 hlm)

- Kerja: Filter folder menampilkan breadcrumb (`default / bleach`) untuk folder bersarang via `folderPath()`; berlaku juga di label scope ekspor. Tanpa sentuh data.
- Akar: data user memang berisi 2 folder beda bernama "bleach" (1 root + 1 di dalam default, isi halaman berbeda) — bukan bug hitung. UI lama menampilkan nama datar sehingga terlihat duplikat.
- Hasil: `pnpm test` 405/405, `pnpm check` 0 error.
## 2026-10-29 — custom prompt / instruksi tambahan translate (fokus bahasa output)

- Kerja: `append_custom` di `prompt.rs` (cap 2000 char, char-boundary, format JSON tetap wajib) + test; `custom_prompt` masuk `TranslatePageInput/BatchInput/RetryBubbleInput` → `PrepOpts` → prompt + `CacheKey`; frontend: textarea di `TranslatePanel`, `customPrompt` di `TranslateOpts/BatchOpts/TranslateBatchInput/RetryBubbleInput/TranslatePageArgs`, teruskan di translatePage/BatchPanel/ReviewGrid (retry page+bubble); i18n `customPrompt/customPromptPh` 3 bahasa + test list. Sampingan: hapus import tak terpakai di `panel-tablet.test.ts` (pre-existing, `pnpm check` merah).
- Hasil: `cargo test` 118 lulus, clippy bersih, `pnpm test` 405/405, `pnpm check` 0 error.
- Keputusan: custom prompt ditempel SETELAH glossary (paling dekat ekspektasi akhir) tapi tanpa wewenang melonggarkan format JSON; cache key ikut custom prompt supaya ganti instruksi = translate ulang, bukan cache basi.
- Lanjut: smoke translate sekali pakai custom prompt di Tauri dev (verifikasi prompt terkirim); pertimbangkan preset instruksi bahasa bila user memintanya.

## 2026-10-30 — provider-api-refresh 9/9 di worktree (isu #26 + #27)

- Kerja: Worktree `Kuron-studio-provider-api-refresh` branch `feat/provider-api-refresh` (isolasi dari dirt custom-prompt di checkout utama). 1.1: hapus preset OpenRouter/Zen/OpenCodeGo/ClinePass (Rust+TS), deserializer toleran nama hapus → Custom (`cache.rs` sudah `unwrap_or(Custom)`). 1.2: preset Meta `api.meta.ai/v1` + normalisasi simpanan `api.llama.com` di `effective_base_url` + `provider-preset-contract.test.ts` (TS==Rust via `?raw`). 2.1: `vision_capable(provider,id)` allowlist (Meta muse-spark/wonder; Cohere command-a-vision/aya-vision; pola umum) + blacklist non-vision + fallback lama; `openai::list_models` tambah param provider. 2.2/2.3: mock TcpListener `provider_list_mock` + `provider_chat_mock` (path/auth/body per provider, tanpa dev-dep baru). 3.1: default = vision pertama. 3.2/3.3: checkbox Hanya vision + warning non-vision/tak dikenal (i18n id/en/zh) — smoke browser mock: default muse-spark-1.3, filter sisakan vision saja, warning tampil. 4.1: ARCHITECTURE "5 provider BYOK".
- Hasil: `cargo test` 118+8 mock lulus, clippy bersih, `pnpm test` 410/410, `pnpm check` 0 error. `openspec validate provider-api-refresh` hijau, tasks 10/10.
- Keputusan (Ruling): mock tanpa wiremock/mockito ikut preseden `translate_mock.rs` (TcpListener, tanpa dev-dep). Normalisasi base lama tanpa log (mapping murni). Test lama `models[0]` diganti (bukan di-pin) karena spek mengubah perilakunya. Fix `panel-tablet.test.ts` (import tak terpakai) diadopsi dari dirt checkout utama supaya gate hijau. Deserializer mengembalikan error untuk nama benar-benar asing (bukan silent Custom).
- Lanjut: user review → `/opsx:archive`; merge branch worktree ke main (perhatikan dirt custom-prompt yang belum commit di checkout utama).

## 2026-10-30 — fix TLS: webpki roots (semua HTTPS provider gagal UnknownIssuer)

- Kerja: User lapor `error sending request for url (.../v1beta/models?key=...)` saat Load models Gemini. Host reachable via curl (403 wajar) → bukan jaringan. Akar: `reqwest` feature `rustls-tls-manual-roots` TANPA root cert → semua HTTPS gagal di handshake. Fix: tambah feature `rustls-tls-webpki-roots` + `.tls_built_in_root_certs(true)` di builder `AppState::http` (`lib.rs`). Verifikasi throwaway test HTTPS ke generativelanguage (respons HTTP, bukan transport error), lalu hapus.
- Hasil: `cargo test` hijau, clippy 0 error. Commit di atas merge provider-api-refresh.
- Catatan: key user (`AQ.Ab8...`) BUKAN format Gemini API key (`AIza...`, dari AI Studio) — setelah TLS pulih, key salah akan muncul sebagai error API 400, bukan transport. Sampaikan ke user.
- Lanjut: restart dev server; bila user ganti key valid, Load models Gemini harus hijau.

## 2026-10-30 — fix serde AiProviderType (dispatch Gemini nyasar ke /chat/completions → 404)

- Kerja: Log user tunjukkan POST ke `generativelanguage.../chat/completions` (jalur OpenAI) untuk provider Gemini. Akar: saat hapus preset, `rename_all="camelCase"` ikut terbuang dari `Serialize` → DB tulis `"Gemini"` kapital → Deserialize (lowercase saja) gagal → `unwrap_or(Custom)` → dispatch `_ => openai`. Fix: kembalikan `rename_all`, Deserialize terima dua-duanya (`"gemini"|"Gemini"` dst.) supaya baris yang sempat tersimpan kapital tetap terbaca benar. Test RED (`serialize gemini` → `"Gemini"`) lalu GREEN.
- Hasil: `cargo test` hijau, clippy 0, `pnpm test` 410/410, `pnpm check` 0.
- Catatan: model `gemma-4-31b-it`/`gemini-3.8-flash` tidak ada di katalog — user harus pilih dari Load models. Key 401 sebelumnya urusan key user, bukan app.
- Lanjut: restart dev server; provider Gemini tersimpan otomatis benar tanpa simpan ulang.

## 2026-10-30 — fix label bahasa kartu bubble (ID→EN tampil JP/ID)

- Kerja: User lapor kartu bubble #3 berlabel "Original (JP)"/"Terjemahan (ID)" padahal arah ID→EN. Akar: label hardcoded di `EditorPanel.svelte`. Fix: `trgLang` derived dari `translation.targetLang` (disimpan backend) → `lastOpts.targetLang` → default "id"; label `Original` (sumber tak diketahui app) + `Terjemahan ({trgLabel})` via `TARGET_LANGS`; `createEmptyTranslation` pakai target aktual bukan "id". Test RED (`not.toContain` hardcoded) → GREEN; `edit-in-bubble.test.ts` diperbarui.
- Hasil: `pnpm test` 410/410, `pnpm check` 0 error.
- Catatan: prompt backend kirim kode mentah ("into id") — LLM paham, tapi rapi-nya dipetakan ke nama bahasa; tunda (di luar keluhan).

## 2026-10-29 — openspec mobile-home-drawer untuk isu #14

- Kerja: Buat openspec `mobile-home-drawer` (proposal, design, tasks, specs `home/mobile-nav` + `home/header-split`) dari isi issue #14: drawer slide-over + pecah `HomeHeaderDesktop` vs `HomeHeaderMobile` dengan switch `isNarrow`/`watchViewport`; gating `isMobile` hanya untuk kemampuan desktop-only. `openspec validate mobile-home-drawer` hijau.
- Hasil: 5 file baru di `openspec/changes/mobile-home-drawer/`; belum ada perubahan kode.
- Keputusan: Breakpoint reuse `TABLET_MAX_W=768`; desktop = pindahan verbatim dulu baru bangun mobile (hindari drift); string baru 3 bahasa satu commit; badge error tetap di luar drawer.
- Lanjut: review proposal → `/opsx:apply` (fondasi viewport + ekstrak desktop + drawer + gate penuh + smoke 360px vs ≥768px).

## 2026-10-30 — translate-model-per-run 7/7 (model per-run + sidebar + footer)

- Kerja: 1.1 `effective_model(rec, override)` + test. 1.2 field `model` opsional di TranslatePage/Batch/RetryBubbleInput + `PrepOpts.model`, resolusi tepat setelah load (cache key + `translation.model` ikut benar) + test serde kontrak. 2.1 LOV model sesi-saja di TranslatePanel (auto-load saat ganti provider, default = simpanan provider, suffix vision, tanpa tulis DB) + `TranslateOpts.model`. 2.2 teruskan ke batch/retry + `BatchOpts.model` + type `model?`. 2.3 regroup kartu (Opsi lanjutan details + tombol sticky bottom). 2.4 footer `Kuron Studio vX` (`getVersion`) pin-bottom, ukuran disamakan header (text-sm semibold) per koreksi user. Smoke mock: default model = simpanan, filter vision, warning, details+sticky tampil, footer render (versi kosong di mock= wajar, tanpa backend Tauri).
- Hasil: `cargo test` hijau, clippy 0, `pnpm test` 416/416, `pnpm check` 0. `openspec validate` hijau, tasks 7/7.
- Keputusan: override `None`/kosong = perilaku lama byte-for-byte; cache key pakai model hasil resolusi (hindari cache lintas-model basi); footer versi fallback kosong bila getVersion gagal (web preview).
- Lanjut: user review → `/opsx:archive`.

## 2026-10-30 — openspec page-status-truth untuk bug status Belum/Sudah

- Kerja: Investigasi 3 pertanyaan status (Proses = transient detecting/translating saja; simpan detection timpa translated→detected di `detect.rs:242-245` + `EditorPanel.svelte:322`; klik bubble murni tak ubah status, yang memaksa translated = edit teks/auto-save/auto-push P9/isi-manual/retry 1 bubble). Buat openspec `page-status-truth` (proposal, tasks, specs `library/page-status`) — belum ada perubahan kode.
- Hasil: 4 file baru di `openspec/changes/page-status-truth/`; verdict: dua-duanya bug (overwrite + coarse last-write-wins).
- Keputusan: Done = semua bubble `translated` non-kosong; save_bubbles pertahankan translated bila translation ada; transient gantung dinormalisasi saat load; tanpa string i18n baru.
- Lanjut: user review proposal → `/opsx:apply` (1.1→4.1) → gate penuh → archive.

## 2026-10-30 — kunci keputusan mobile-home-drawer sebelum apply

- Kerja: Tanya jawab 3 keputusan + cek `EditorPanel.svelte:630-699` (sudah viewport-aware via `columnMode`/`isNarrow`). Update openspec: drawer kiri, CTA Detect+Import berdampingan, scope termasuk audit header editor ≤360px (task §4 baru + skenario editor di `header-split`).
- Hasil: `openspec validate mobile-home-drawer` hijau; Open Questions dikosongkan.
- Keputusan: Drawer kiri (konsisten dropdown origin-kiri); 2 CTA tetap satu ketukan; editor tanpa drawer kedua (reuse pola, Save+Detect prioritas terlihat).
- Lanjut: `/opsx:apply` task 1.1 (state `narrow` + test viewport).

## 2026-10-30 — page-status-truth 4.1/4.1 (status jujur turunan data)

- Kerja: Implement penuh openspec `page-status-truth`. Backend: `translation_complete` + `manual_done` di `PageTranslation`; `persist_result` dipakai apply_cached/store_result/persist_translation (sticky flag manual); `save_translation` + `persist_bubbles` guard; command `mark_page_completed`; normalisasi saat `ProjectStore::load`. Refactor `save_translation_inner`/`mark_page_completed_inner` (&AppState) agar unit-testable. Frontend: helper `statusAfterBubbleSave/statusAfterTranslationSave/translationComplete`, 4 mirror `onSaved`, tombol `markDone` + string 3 bahasa + `api.markPageCompleted` + kontrak. `ARCHITECTURE.md` §4 diperbarui.
- Hasil: `cargo test` hijau (lib 136 + integrasi), clippy `-D warnings` bersih, `pnpm check` 0, `pnpm test` 435/435. 12 test Rust baru + `page-status.test.ts`. Test lama: `edit-stuck` D2 count 4→5 (jalur save baru), `markDone` dipindah sebelum `onKey` agar slice-2000-char test error-path tetap valid.
- Keputusan: Done = semua terisi ATAU manual; sebagian tetap Belum (tak ada status baru); data lama dinormalisasi saat load; re-translate tak mencabut flag manual (sticky) — cabut hanya via unmark.
- Lanjut: user review → smoke GUI skenario lapor (translate → simpan detection → tetap Sudah; mark done → Sudah lintas reload) → `/opsx:archive`.
## 2026-10-29 — Merge main (24 commit) ke feat/grid-preview-perf, tanpa fitur hilang

- Kerja: Merge `origin/main` (PR #23–#32: provider rework, translate regroup, editor footer, dsb). Konflik hanya 2 file i18n (key `resultsExport` milik kita + `customPrompt*` milik main — digabung, `review` lama dibuang).
- Verifikasi tidak ada yang hilang: diff vs HEAD hanya penambahan main (customPrompt/model pass-through); seluruh fitur kita utuh (Grid/List, sortir toolbar, scope ekspor, cache preview).
- Hasil: `pnpm test` 417/417, `pnpm check` 0, `cargo test` 125 lulus, clippy bersih.

## 2026-10-30 — amandemen page-status-truth §5 (kanban cermin status)

- Kerja: User usul bucket kanban = status simpanan (idle/terdeteksi/translated/failed) ganti Belum/Proses/Sudah/Gagal. Amandemen ke change terbuka `page-status-truth` (bukan change baru, cegah bentrok): proposal What Changes + capability, spec requirement "Buckets mirror stored status" + 2 skenario, tasks §5 (5.1 bucket+rank, 5.2 key `kanbanDetected`, 5.3 gate ulang + screenshot). `translating` ikut data (lengkap/manual → Sudah, else Terdeteksi). `openspec validate` hijau.
- Hasil: spec + tasks siap review; belum ada perubahan kode (§5 unchecked).
- Keputusan: Bucket Proses dihapus dari kanban; progres operasi tetap di panel progres. Urutan sortir Belum–Terdeteksi–Sudah–Gagal.
- Lanjut: user review §5 → apply 5.1–5.3 → gate → archive.

## 2026-10-30 — providers: Validate aktif setelah simpan + model dipakai

- Kerja: `save()` tidak lagi `startNew()` — tetap mode Edit pakai `saved.id` supaya tombol Validate langsung aktif; tooltip "Simpan provider dulu, baru Validate aktif." saat disabled.
- Hasil: `pnpm test` 437/437, `pnpm check` 0; smoke mock: sebelum simpan DISABLED+tooltip, sesudah simpan ENABLED + header "Edit".
- Keputusan: kolom Model BENAR dipakai — rantai `fModel -> save_provider.model -> ProviderView.model -> default TranslatePanel (loadModelsFor) -> translation.model + cache key`. Field manual + Lov "Model tersedia" komplementer (ketik vs pilih hasil Load models), bukan duplikat.

## 2026-10-30 — pintu-ekspor 6/7 (pintu Hasil & Ekspor per kartu)

- Kerja: Aksi ⧉ per kartu dashboard + `openResults` (aktifkan bila perlu + tunggu + buka; gagal = panel tidak dibuka); busy guard; tanpa state/i18n baru (reuse `resultsExport`); test pola-sumber baru.
- Hasil: `pnpm test` 440/440, `pnpm check` 0, `cargo test` 137 lulus, clippy bersih.
- Lanjut: smoke 2.3 (kartu non-aktif/aktif, mobile 360px).

## 2026-10-29 — bubble-polygon: outline mengikuti bentuk (opsi B, tanpa model)

- Kerja: Modul `outline.rs` baru (`refine_outline`: flood + opening 3x3 + Moore + RDP + cap 64) + kabel di `detect_bubbles` (box tanpa shape dicoba refine, gagal → kotak diam-diam). Aturan leak diperbaiki tengah jalan: border bbox BUKAN bocor (bbox ketat hal biasa), syaratnya menabrak dinding.
- Verifikasi: unit sintetis hijau (elips/penuh/putih/gelap/tiny), smoke sampel nyata `before.jpeg`: detektor 4 kotak → refine 4/4 lolos, rata-rata 14 titik. `cargo test` 142 lulus, `clippy -D warnings` bersih. File spike dibuang (`spike_shape_tmp`, `spike_outline_tmp`).
- Lanjut: mata user menilai overlay hijau-vs-merah (`%TEMP%\kuron-smoke-outline.png`); kalau lolos → archive, kalau gagal → catat batasnya.

## 2026-10-30 - bubble-polygon: pivot ke mask model (output1), flood jadi fallback

- Temuan: `bubble.onnx` = YOLO11n-seg manga109, output1 (32x256x256) + koefisien/anchor selama ini tak dibaca. YOLO26-COCO salah domain + AGPL + jumbo -> ditolak.
- Kerja: `decode_kept` (anchor lolos NMS) + `mask_coeffs` asli + `model_mask_polygon` (sigmoid-dot, crop 256, Moore+RDP, inverse-letterbox) + kabel `detect_bytes` (output1 asing -> diam). Flood-fill tetap sbg fallback lapis-2.
- Bukti: 6 unit test baru hijau; smoke `before.jpeg` 4/4 shape (avg 12.2 vertex); `cargo test` 148 lulus; clippy bersih; spike dihapus.
- Sisa: mata user nilai overlay `%TEMP%\kuron-smoke-mask.png` (2.1); kalau lolos -> archive.

## 2026-10-30 - text-fit-polygon: teks wajib di dalam polygon (1.1-1.5 hijau)

- Aturan: tiap baris dibatasi chord polygon pada pitanya (pilih interval muat-cx else terlebar), blok tengah dalam rentang vertikal polygon, tiap baris digambar tengah-chord. Tanpa shape = jalur lama utuh.
- Dua sisi: TS `textLayout.ts` (kanvas putuskan, simpan di `layout`) + port Rust `export.rs` (fallback + draw + overflow QA). Kabel: `LayoutFor`+shape, `syncGeometryToTranslation`, EditorPanel, CanvasEditor per-baris, `qa.rs`.
- Bukti: TS 16 + Rust 4 test baru; smoke export: 0 piksel teks di luar elips (`%TEMP%\kuron-smoke-textfit.png`); `pnpm test` 445, `check` 0, `cargo test` 152 lib + semua suite, clippy bersih; spike dihapus.
- Sisa: mata user di kanvas (`tauri dev`) + PNG export nyata (2.1); manual fontSize tetap pass-through (QA flag bila luber).

## 2026-10-30 - text-fit-polygon 1.6: manual ikut dipaksa masuk polygon

- Manual `fontSize` = batas atas (bukan nilai mutlak): loop susut yang sama sampai muat chord, floor minimum. Tanpa shape tetap pass-through.
- Test: manual 72 di elips -> susut + tiap baris <= chord; manual 20 yang muat -> tetap. `pnpm test` 447, `check` 0.
- Catatan: layout lama tersimpan (manual luber era sebelum ini) sembuh saat halaman di-save ulang (`syncGeometryToTranslation` hitung ulang tiap save); sebelum itu QA menandainya.

## 2026-10-30 - text-fit-polygon 1.7: ekor keluar dari area teks

- Masalah (screenshot user): bubble berdurasi/ekor -> chord ekor sempit memaksa font mini + pecah per huruf ("H E L P U S ! !" vertikal di ekor).
- Aturan: badan = lajur menyambung sekitar chord terlebar dgn lebar >= 0.35x terlebar (sampling 2px); blok teks tengah di badan saja. Konstanta `BODY_MIN_FRAC` dipin kontrak dua sisi.
- Bukti: TS test ekor, Rust tail-test (span hi<=152, font>min, pita tak masuk ekor, no-overflow); `pnpm test` 448, `check` 0, `cargo test` 153 lib + semua suite, clippy bersih.

## 2026-10-31 - frontend-domain-move: 15/15, 10 commit hijau + docs

- Pure move 77 entri `src/lib/` -> 9 domain (contracts/app/ui/library/editor+editor-text/detection/provider/export/previews). Nol rename/logika; import relatif murni (tanpa alias).
- 10 commit hijau berurutan (df339ea..90c8cb1 + dff18fb docs): tiap folder = git mv + fix import + gate. Deviasi tercatat: test-assertion path string ikut diupdate (mobile-gating, panel-tablet, previewGrid, spacing, lov-dropdown); translate-cancel sementara `../ReviewGrid` lalu final `../export/`; panel-tablet.test ketinggalan di 2.5, menyusul commit sendiri.
- Gates: pnpm test 448, svelte-check 0, cargo test 153 lib + semua suite, clippy bersih. Root lib/ = 9 folder persis; ARCHITECTURE.md §3 sinkron tree.
- Lanjut: /opsx:archive frontend-domain-move (FE-1 selesai; FE-2..FE-11 + XC-2 menunggu).

## 2026-10-31 - app-runes-modules: 8/9 (3.2 smoke manual menunggu user)

- Ekstrak 4 subsistem App.svelte -> runes module: library/projectStore (+5 test), library/selection (+8 test, +setSelectMode), previews/thumbnailQueue (+4 test), detection/detectRun (+3 test).
- Script App 1198->~484 baris; target <350 TAK tercapai (sisa = orkestrasi komposisi, ranah FE-3). Nol $state subsystem, nol assign projects=.
- Gates: pnpm test 468-472, svelte-check 0. Rust tak tersentuh.
- Deviasi tercatat: assertion source-pattern diperbarui ke wiring baru; import .svelte.ts wajib eksplisit; gen-guard refresh ditambah di store (lebih ketat dari App lama).
- Sisa: 3.2 smoke manual (user).

## 2026-10-31 - provider-flow-diagrams: 2 diagram .mmd di root

- Kerja: diagram alur `list_models_draft` (provider belum disimpan) dan `validate_provider` (provider tersimpan) dari kode aktual: ProviderSettings.svelte, commands/provider.rs, provider/{mod,openai,gemini,cohere}.rs, secrets.rs.
- Hasil: `diagrams/provider-list-models-draft.mmd` + `diagrams/provider-validate.mmd`, lolos `mmdc`; versi cetak `*.pdf` satu halaman ukuran natural (`--size 2100`); ARCHITECTURE.md bagian `translate` mereferensikan keduanya.
- Keputusan: taruh di `diagrams/` root, bukan `openspec/specs/.../diagrams/` - `/openspec/` di-gitignore (lokal), jadi referensi dari dokumen tracked akan menggantung.
- Lanjut: spec provider di `openspec/specs/kuron-studio/spec.md` masih drift (9 tipe + `store.json` + `validate_provider(config)`); kandidat change `sync-provider-spec`.

## 2026-10-31 - app-shell-components: 8/9 (3.2 smoke manual menunggu user)

- 7 komponen shell di lib/app/shell/: AlertBanners, ProjectCard (dedup row/card), FolderBar (chipSpecs + satu {#each}), KanbanBar (+SortKey/PageView/KanbanFilter ke library), ProjectSwitcher (self-contained), AppHeader (~19 prop eksplisit, deviasi tercatat), OverlayHost (28 prop eksplisit).
- App.svelte 1198->679 baris; target <350 tak tercapai (sisa script orkestrasi + markup dashboard/grid) -> ranah FE-4/FE-5/FE-11.
- Gates: pnpm test 480, svelte-check 0. Rust tak tersentuh.
- Sisa: 3.2 smoke manual (user).

## 2026-10-31 - editor-panel-split: 8/9 (4.2 smoke manual menunggu user)

- 7 unit editor: TranslationCard, GeometryControls, EditorToolbar (PaneMode pindah ke sini), EditorPanes (shared-spread + bindable refs, CanvasApi), Divider (ui/), hook useDraftText (+5 unit test), useSyncedPage (rev/structurallyEqual +9 unit test, dokumen pemilik bubbles), usePanelResize.
- EditorPanel.svelte 1088->720 baris; target <300 tak tercapai (sisa = orkestrasi save/detect/batch, markup sidebar, 3 view-model $derived komposisi) -> butuh unit lanjutan, di luar scope.
- Gates: pnpm test 500, svelte-check 0. Rust tak tersentuh. Satu commit hijau (per-unit tak direkonstruksi jujur dari satu tree).
- Deviasi tercatat di tasks.md: string-test di-repoint bukan dihapus (unit tak mencakup wiring); PaneMode diimpor dari EditorToolbar; CanvasApi Pick<> untuk refs.
- Sisa: 4.2 smoke manual (user) — ketik/debounce/IME/Esc, resize/collapse/snap, toolbar 12 kontrol, split pane.

## 2026-10-31 — audit-yoga-issues: relevan sudah tutup semua, tanpa close baru

- Kerja: Petakan 40+ commit Yoga (`feat/grid-preview-perf`, PR #24–#75: grid/list + sorting, provider refresh, model-per-run, LOV, page-status, FE-1 domain-move … FE-9 i18n-keys + token ilustrasi) ke GitHub issues via `gh issue list --state all` + verifikasi file (shell 7 komponen, unit editor, translateOpts/storage, PageGrid/PageList, DetectProgress, MoveDialog + parent_id, preset api.meta.ai, key nothingToClean).
- Hasil: Semua issues relevan SUDAH CLOSED dan tutupnya benar (#7, #12, #17, #25–#28, #37, #40–#49). Tidak ada close baru. Sisa OPEN (#14 drawer, #29, #38/#39, #50–#67) di luar scope kerja Yoga — #14 baru spec tanpa implementasi, #50 ReviewGrid tak tersentuh (yang dipecah EditorPanel/CanvasEditor), #60/#61 justru bertambah beban (test ?raw baru).
- Keputusan: Fokus MEMORY.md diluruskan (sebelumnya basi: #7/#12/#17 masih OPEN).
- Lanjut: #14 (implementasi drawer), #50–#67 (backend/batch) menunggu pemilik berikutnya.

## 2026-10-31 — kontrak-json + api-mati (#60, #59)

- Kerja: (A) #60 — buat `kuron-studio/src/lib/contracts/{extensions,layout,providers}.json` sebagai satu sumber kebenaran; Rust (`project.rs` IMAGE_EXTS/is_archive/is_pdf, `export.rs` LAYOUT_* + BODY_MIN_FRAC/SCAN + daftar font per-OS, `config.rs` default_base_url saja) baca via `include_str!` + `OnceLock`; TS (`types.ts`, `textLayout.ts`, `textMeasure.ts`) impor JSON langsung (`resolveJsonModule` sudah aktif); tulis ulang 3 test kontrak (banding JSON lawan pemakaian, tanpa path `src-tauri/`; `invoke-contract` sudah patuh, tak diubah); tambah paragraf aturan kontrak di `ARCHITECTURE.md` + koreksi baris tabel `contracts/`. (B) #59 — hapus `invoke_result` (+`type_name` + testnya) dan `chunk_webtoon` (+testnya); `redacted` dan `font_for` jadi `#[cfg(test)]`. Tak sentuh: `mask_coeffs` (dipakai `detector.rs`), `paint_patch` (dipakai `psd.rs`), `models_url`, dan `src-tauri/src/domain/` (untracked milik lane lain).
- Hasil: Gate A hijau (cargo 170 lolos + clippy bersih; pnpm 561 lolos + svelte-check 0). Gate B hijau (cargo 155 lolos + clippy bersih; pnpm 561 + svelte-check 0). Verifikasi: tak ada test baca `.rs` via `?raw`/`readFileSync`, tak ada path `src-tauri/` di 4 test kontrak (sisa `Cargo.lock` read hanya di `tauri-version-contract`, di luar scope). Semua perubahan uncommitted, tanpa commit/push.
- Keputusan: `BODY_SCAN_STEP` ikut masuk `layout.json` (walau bukan `LAYOUT_*`) agar aturan "konstanta lintas bahasa hanya di JSON" utuh; float JSON diparse sebagai `f32` di Rust (dipin test `*_matches_json_contract`, nilai 0.35 dkk. lolos identik); `ProviderBaseUrls` + `static` ditaruh lokal di `default_base_url()` agar diff minimal.
- Lanjut: Smoke manual + PR oleh pemilik (instruksi: jangan commit/push/tutup issue). Klaim issue #59 soal `models_url`/`paint_patch`/`mask_coeffs` memang kedaluwarsa — ketiganya tetap dipakai dan tak disentuh.

## 2026-11-01 — BE #56 satukan helper provider (lane http)
- Kerja: buat `src-tauri/src/provider/http.rs` (`err_preview`, `preview`, `norm_base`, `data_url`, `b64`, `is_429`, `rate_limited`, `post_with_retry`); `openai/gemini/cohere.rs` pakai helper (URL, auth, body, log, string error identik); `commands/image.rs` 3 titik pakai `data_url`; `config.rs` `models_url` jadi `#[cfg(test)]`. Tak sentuh `translate.rs`, `project.rs`, `export.rs`, `lib.rs`, `domain/`.
- Hasil: `provider_chat_mock` + `provider_list_mock` 8/8 hijau tanpa ubah ekspektasi; `pnpm test` 561 hijau; `pnpm check` 0 error.
- Keputusan: `norm_base(url, strip)` satu definisi dengan daftar sufiks per provider agar perilaku strip `/v1beta` vs `/v2|/v1` tetap beda seperti semula.
- Lanjut: gate Rust penuh masih merah karena `commands/project.rs` lane lain (E0451 `legacy_pages` privat + unused `now_epoch`) — di luar scope, jangan diambil alih.

## 2026-11-01 — BE-layer #54 pindah kode render ke `src-tauri/src/render/`

- Kerja: Pure move verbatim render/gambar ke modul baru `src-tauri/src/render/` (`mod.rs` + `image_ops.rs`, `detect_decode.rs`, `detector.rs`, `outline.rs`, `psd.rs` via `git mv`; `page.rs` baru = cluster teks dari `commands/export.rs`: glyph/bitmap/TTF, layout/bungkus-kata, `paint_patch`, `render_page_png` + 8 test render + `text_position_tests`). `shapes_for_export`/`select_pages`/`collect_pages` tetap di `commands/export.rs` (orkestrasi). Hanya mekanis: path `crate::X::` → `crate::render::X::` (`detector`, `detect_decode`, `psd`, `commands/detect.rs`, `commands/translate.rs`, `commands/batch.rs`, `qa.rs` + doc-nya, `tests/{smoke_detect,e2e_flow_mock,batch_export_mock}.rs`); `lib.rs` ganti 5 `pub mod` jadi `pub mod render;`; `export.rs` impor `render::page::render_page_png`. Tak sentuh: `bubble.rs` (tipe bersama), `preview_cache/`, `provider/config.rs::models_url` (API provider, bukan render), kontrak invoke, frontend.
- Hasil: `cargo test` hijau (164 lib + 6 file integrasi semua lolos), `cargo clippy --all-targets -- -D warnings` bersih, `pnpm test` 561 hijau, `pnpm check` 0 error/warning. `outline.rs` byte-identik; `image_ops.rs` hanya berisi perubahan lane lain yang terbawa utuh via `git mv`. Tanpa commit/push.
- Keputusan: `mask_coeffs` ikut pindah bersama `detect_decode.rs` (pemakai `detector.rs` pindah bersama — sesuai arahan); `paint_patch` pindah ke `render/page.rs` (pemakai `psd.rs` + `render_page_png` diperbarui); `models_url` tetap di provider. Test render ikut kode-nya pindah; test orkestrasi (`shapes_*`, `select_pages`) tetap di `export.rs`.
- Lanjut: Smoke manual + PR oleh pemilik (instruksi: jangan commit/push/tutup issue).

## 2026-11-01 — BE #53 pindah pipeline import ke commands/import (pure move)

- Kerja: Pindah pipeline import dari `src-tauri/src/commands/project.rs` ke modul baru `src-tauri/src/commands/import/mod.rs` — verbatim tanpa perubahan perilaku: `ImportResult`, `is_appledouble`, `is_archive_metadata`, `safe_join`, `is_image_file`, `is_archive`, `is_pdf`, `pdf_image_filter_kind`, `inflate_flate`, `write_flate_png`, `extract_pdf_pages`, `rotate_file_if_needed`, `page_from_file`, `expand_input`, `extract_archive`, `normalize_picker_path`, `stage_ios_pick` (2 varian cfg), command `import_pages`, + 12 test import/pdf (termasuk `pdf_tests`). `project.rs` 1662->860 baris. Wiring: `pub mod import` di `commands/mod.rs`; `lib.rs` impor `import_pages` dari `commands::import`.
- Hasil: Saat tree konsisten: `cargo test --lib` 164 lolos, `clippy --all-targets -- -D warnings` bersih, `pnpm test` 561 lolos, `pnpm check` 0 error. Kontrak invoke tak berubah (nama command + signature sama).
- Keputusan: `image_exts`/`extensions`/`ExtensionContracts` TETAP di `project.rs` (milik lane #60) jadi `pub(crate)` agar modul import bisa pakai; `page_from_file` jadi `pub(crate)` untuk `is_page_readable` (clean). Buang 1 import tak terpakai (`ProjectStore` di modul import). Tanpa `#[allow]` baru.
- Lanjut: Gate penuh (`cargo test` all-targets) saat ini MERAH oleh lane lain yang sedang aktif edit (`batch.rs`/`translate.rs`: `PageWork`, `cache_key`, `client`, `map_text`/`store_result` arity) — di luar scope #53, jangan ambil alih. Tak commit/push sesuai instruksi.

## 2026-11-01 — BE #55 pipeline translate + JobRegistry

- Kerja: (1) Baru `src-tauri/src/jobs.rs` — `JobRegistry` per-batch: `admit` dedup page ganda (lapor busy tanpa claim), `semaphore` batas POST paralel (default 3, sama spt inline lama), `cancel`/`is_cancelled` teruskan flag `AppState` (AppState tak diubah). (2) `commands/translate.rs`: `PagePrep` dipecah provider-agnostik + `ProviderBinding {rec, client, cache_key}` (persiapan #66 failover); `prepare_page` kembalikan tuple; `translate_prep(state, prep, binding)` jadi SATU-SATUNYA orkestrasi (+pre-flight cancel: job antre yang dibatalkan restore tanpa sentuh cache/POST); `bind_provider` gagal → status pulih pra-klaim (sebelumnya provider-tak-ditemukan bikin halaman nyangkut Translating); hapus `PageWork`; `load_provider` jadi satu-satunya resolve provider. (3) `commands/batch.rs`: `translate_batch` pakai registry (admit→prepare→release; worker semaphore→translate_prep→release), hapus `run_work` (duplikat verbatim); `retry_bubble` tak diubah. (4) `commands/provider.rs`: hapus duplikat `record_of`, `list_models`/`validate_provider` pakai `load_provider`. Kontrak invoke tetap; tanpa string UI baru; tanpa `#[allow]` baru. Tak sentuh: domain/, store/, db/, provider/, project/image/export/detect commands.
- Hasil: Test baru 5 lolos — `jobs::` 4 (dedup+release, batas 3, new(0)→1, cancel→flag) + `translate_prep_preflight_cancel_tanpa_post_atau_cache`. Gate hijau: `cargo test` 169+integrasi lolos, `clippy --all-targets -- -D warnings` bersih (1 temuan `manual_inspect` diperbaiki via `inspect_err`), `pnpm test` 561 lolos, `pnpm check` 0 error. Catatan: tree sempat merah oleh lane lain (render/import wiring) lalu hijau sendiri saat lane mereka landing; `pub mod page` di `render/mod.rs` sudah ada file-nya — tak perlu intervensi.
- Keputusan: Menyimpang dari issue #55 langkah 7 (kanonik di `provider::record_of`) karena `provider/` dibekukan lane lain — arah dibalik: provider-command memakai `translate::load_provider`. Langkah besar issue (AppError, derive_status baru, AppState tanpa cancel, retry via pipeline) di luar scope minimal lane ini; `AppState::cancel/prev_status` tetap.
- Lanjut: Smoke manual (translate 1 halaman, batch 3 paralel, cancel tengah jalan, retry 1 bubble) oleh pemilik; tak commit/push sesuai instruksi.

## 2026-11-01 — Verifikasi gelombang #53/#54/#55 (koordinator)

- Kerja: Verifikasi pasca-landing ketiga lane paralel di branch `shirokun20/be-layer` (uncommitted): #53 `commands/import/` (project.rs 1662→860 baris), #54 `src-tauri/src/render/` (image_ops, detector, outline, psd, page teks), #55 `jobs.rs` JobRegistry + refactor translate/batch.
- Hasil: Gate penuh hijau dari koordinator — `cargo test` 169 lib + 8 target integrasi lolos, `clippy --all-targets -- -D warnings` bersih, `pnpm test` 561/561, `pnpm check` 0 error. Merah-sementara saat lane berjalan (laporan #53) terkonfirmasi sembuh sendiri setelah semua landing; tak ada konflik sisa antar-lane.
- Keputusan: Urutan berikut #57 AppError TERAKHIR setelah semua move (migrasi sekali di atas struktur final); #62 butuh #52+#55 (keduanya hijau) → siap; #63 paling akhir; lalu epik #38/#39.
- Lanjut: #57 (AppError thiserror), #62 (translate queue), #63, epik #38/#39. Smoke manual translate + review PR/close issue oleh pemilik; belum commit/push.

## 2026-11-01 — Selesai gelombang #57/#62-followup/#63 + tutup issue (koordinator)

- Kerja: (1) Lane #57 AppError: `src-tauri/src/error/` + migrasi internal, signature command tetap string; sempat merah di `tests/` lalu dibetulkan lane tests. (2) Lane #62-followup: `tests/batch_export_mock.rs:78` → AppError, `tauri/test` pindah ke dev-deps (build produksi terverifikasi bersih via `cargo tree -e no-dev`), test IPC `commands_project/translate/extras` + `common::test_app`; `translate_batch` via IPC infeasible (signature `AppHandle<Wry>` vs `MockRuntime`) — cakup pengganti kontrak JobRegistry + IPC `retry_bubble`. (3) Lane #63: ARCHITECTURE.md §4/§9 + AGENTS.md butir 9-11 + ADR-001..005; saya pindahkan `docs/adr/` → `adr/` root karena `/docs/` di-gitignore (tak ikut PR) + betulkan path di ARCHITECTURE.md.
- Hasil: Gate diverifikasi koordinator — `cargo test` 12 target ok (173 lib), clippy bersih, `pnpm test` 561/561, `pnpm check` 0 error. 12 issue ditutup via `gh issue close` (#51,#52,#53,#54,#55,#56,#57,#58,#59,#60,#62,#63). Epik #38/#39 diberi komentar progres jujur, tetap terbuka: #38 target baris belum tercapai (859/291/942 vs 450/120/250); #39 sisi FE belum diverifikasi.
- Keputusan: Tak commit/push (menunggu instruksi pemilik). Sisa: smoke manual translate, review PR, gelombang peramping #38 (opsional), verifikasi FE #39.

## 2026-11-01 — Sisa epik #38/#39: peramping backend + konversi test FE (koordinator)

- Kerja: Dua slice paralel via subagen di branch `shirokun20/be-layer` (uncommitted). (1) #38: pure-move `project.rs` 859→283 (`render/sample.rs` generator sample, `store/folder_ops.rs` folder/clean/move inners), `export.rs` 291→88 (packer+select ke `render/export.rs`, `shapes_for_export` ke `render/page.rs`), `translate.rs` 942→130 (direktori baru `translate/`: `snapshot.rs`, `provider_bind.rs`, `pipeline.rs`). (2) #39: 33→6 test source-pattern di luar `contract/` — 4 file lint dihapus digabung ke `contracts/lint.test.ts`, 18 file ditulis ulang behavioral (mount+unit, helper `renderWithI18n()` di `test-utils.ts`, mount test EditorPanel/CanvasEditor/ReviewGrid/ProviderSettings/BatchPanel/PageGrid), 6 sisa jujur (i18n key-scan + 5 editor menunggu pecah #50). `provider-preset-contract.test.ts` false-positive grep (komentar sebut `?raw`, isinya baca JSON — sah).
- Hasil: Gate penuh hijau terverifikasi koordinator — `cargo test` 204 lolos (173 lib + 11 target integrasi), `clippy --all-targets -- -D warnings` bersih, `pnpm test` 497/497 (7,3s), `pnpm check` 0 error. DoD baris #38 tercapai semua; DoD #39: ≤6 source-pattern ✓, 6 mount ✓, i18n key-scan diverifikasi gagal bila key fiktif disuntik ✓. Satu edge `commands::detect` di `translate/pipeline.rs` hanya di `#[cfg(test)]`, bukan produksi.
- Keputusan: Tak commit/push/tutup issue (menunggu pemilik). Laporkan angka jujur: pnpm 561→497 karena assertion `toContain` dihapus, bukan cakupan hilang — tiap hapus dipasangi behavioral pengganti.
- Lanjut: Smoke manual translate, review PR, tutup epik #38/#39 oleh pemilik.

## 2026-11-01 — Resolusi konflik PR #78 (be-layer vs main Yoga)

- Kerja: Trial-merge di worktree sekali-pakai → hanya 2 file konflik (Yoga tak sentuh `src-tauri`). (1) `mobile-gating.test.ts` (modify/delete): pertahankan hapus, `contracts/lint.test.ts` dialihkan ke `ExportActions.svelte` (Yoga pindah gating ReviewGrid→ExportActions di d263261). (2) `translate-cancel.test.ts` (content): pakai versi behavioral kita + port 3 assertion baru Yoga tanpa `?raw` — wiring `exportState.retryPage/retryBubble`→konverter (unit via `createExportState` + mock api), urutan Model-sebelum-Target + details + sticky (mount TranslatePanel), ganti-model tanpa `saveProvider` (mount + pilih M2, assert `onOptsChange` + save tak terpanggil). Merge `origin/main` (f888781) ke be-layer, commit lokal `4e51117` (amend sekali karena `lint.test.ts` tertinggal staging), worktree sekali-pakai dihapus. Tanpa push.
- Hasil: `pnpm test` 56 file/524 lolos (naik dari 497: test baru Yoga `BatchTranslateDialog` + `progressStats` + port kita), `pnpm check` 1008 file 0 error, `cargo test` 12 suite ok, `clippy` bersih.
- Keputusan: Resolve = gabung dua intent (konsolidasi lint + struktur baru Yoga; behavioral + assertion baru Yoga), bukan pilih satu sisi. Satu `git add` sempat tak menempel (`UU` persisten) — diverifikasi via `git ls-files -u` + add ulang sebelum commit.
- Lanjut: Push + review PR #78 oleh pemilik; smoke manual translate.

## 2026-11-01 — Tutup epik #38/#39 (verifikasi + sisa DoD di main)

- Kerja: Verifikasi DoD kedua epic di `main`. #38 kurang 2: (1) `commands/import/` → `src/import.rs` top-level (pure move, `git mv`; wiring `lib.rs`/`folder_ops.rs`/`tests/common`, `super::` → path absolut). (2) `cancel`/`prev_status` keluar dari `AppState` → `translate/cancel.rs::CancelState` (manage terpisah); pipeline pakai `Ctx{app,cancel}`; 5 command ambil `State<CancelState>` (JS unchanged); `JobRegistry::cancel/is_cancelled` teruskan CancelState; semua constructor test + mock app diperbarui. Insiden kecil: 1x `#[tauri::command]` terhapus (PUT timpa atribut — dikembalikan), 1x `&Ctx` tanpa lifetime (E0726 — 11 titik), 1x `jobs` mod tertimpa (dikembalikan), 2 screenshot liar ikut `git add -A` (dikeluarkan via amend, file diamankan di `/tmp/kuron-stray/`).
- Hasil: Gate hijau — `cargo test` 175 lib + 11 target ok, `clippy` bersih, `pnpm test` 524/524, `pnpm check` 0 error. Baris: 283/88/134. Commit lokal `d9b25bb` di `main` (BELUM push). Issue #38 + #39 dikomentari bukti + ditutup via `gh` (completed).
- Keputusan: Edge `import.rs` → `commands::project::{extensions,image_exts}` disengaja (konstanta kontrak milik lane #60); satu edge test-only ke `commands::detect` diterima. Push + smoke manual oleh pemilik.
- Lanjut: Push `main`, smoke manual translate, epik #37 FE (milik Yoga: #50 dkk), fitur #65/#66/#67.

## 2026-11-01 — Fix status macet Detected (SFX kosong)

- Kerja: User lapor status TERDETEKSI padahal sudah translate + re-translate tetap. Ground truth dari `projects.json` p015: 11 bubble, 2 kosong (idx SFX `にちゃ♡`/`ぐにっ♡`) dengan `ai_translated` juga kosong → provider yang mengosongkan, bukan bug mapping/merge. `translation_complete` (semua-terisi) lalu menahan status di Detected by design (5503f4f). Bonus temuan: record tersimpan masih `targetLang: en/style: dramatic` (run lama), sedang dropdown kini Indonesia/Standard — After canvas memang render simpanan lama. Fix `e1440b4`: `fallback_translated()` di `map_mosaic`/`map_full_image` — `translated` kosong → isi `original`, `ai_translated` tetap mentah sebagai audit; rank hilang total tetap kosong. +1 unit test. Konsumen (`qa sfxLeak`, overlay, ResultTable, memory search) aman — justru konsisten.
- Hasil: `cargo test` + `clippy` + `pnpm test` 532/532 + `pnpm check` 0 error hijau.
- Keputusan: Fallback data (bukan lemahkan rule status) supaya kanvas/export tak bolong. Data lama tak dimigrasi — user translate ulang halaman.
- Lanjut: Push (masih di tangan pemilik), user verifikasi p015 → Translated.


## 2026-11-01 — Sweep issue terbuka

- Kerja: Cek 6 open issue atas instruksi pemilik (kalau sudah → close). #50: semua AC terverifikasi (ReviewGrid 129 baris, single mapping translateOpts, progressStats 4 kasus, tanpa colspan, mobile gating di lint.test.ts, gate hijau) → komentar verifikasi; close ditolak (sudah ditutup pihak lain). #65: batch-dari-list via BatchTranslateDialog di OverlayHost, tooltip lama hilang → komentar + close. #66 (multi-provider pool: masih `provider_id` tunggal + semaphore global), #67 (double-POST parse-gagal masih ada), #14 (tanpa drawer/isNarrow) → belum, dibiarkan. #29 sebagian (filter default all + tab QA/TM dihapus) tapi keputusan A/B/C belum → dibiarkan tanpa komentar.
- Lanjut: Push menumpuk (`d9b25bb`, `e1440b4`) di tangan pemilik.
## 2026-11-01 — Fix progress card offside + overlay translate ngikut bentuk bubble

## 2026-11-01 — Drawer sempit + About, tutup #29/#14

- Kerja: #14: `AppHeader` pakai snippet aksi tunggal; `narrow` dari `isNarrow()/watchViewport()` (App.svelte); bar ringkas + drawer (Esc/overlay/×, min-44px, divider hidden via arbitrary variant — terbukti di CSS hasil `vite build`); gating desktop-only tetap `isMobile` (lint source-scan hijau). About: `AboutPage.svelte` baru (Panel; versi dari package.json; fitur/privasi/syarat; 16 key × 3 bahasa) + mount OverlayHost + tombol ⓘ header/drawer. Test baru 5 (AppHeader.test, AboutPage.test). #29: opsi A dinyatakan terwujud. Insiden edit: 2x PUT timpa baris tetangga (onToggleReadingDir, showDiagnostics — dikembalikan), 1x tag duplikat, 1x `aside`→`div` (a11y), 1x backdrop vs × label tabrakan (`closeMenu` baru).
- Hasil: `pnpm check` 0/0, `pnpm test` 539/539, `vite build` ok. Commit lokal `2f3e539` (di atas commit paralel pemilik). Issue #29 + #14 dikomentari + closed.
- Keputusan: Satu komponen + snippet (bukan 2 file header) agar tombol tak duplikat; About selalu tersedia (drawer di sempit).
- Lanjut: Push menumpuk di pemilik; smoke manual drawer (resize <768px) + About 3 bahasa.


## 2026-11-01 — Menu bar macOS + shortcut + About legal

- Kerja: (1) Menu About macOS direroute: `src-tauri/src/menu.rs` baru (desktop-only, App/Berkas/Edit/Jendela, semua predefined kecuali About kustom `show-about`) + `on_menu_event` -> emit -> `onShowAbout` (events.ts) -> AboutPage. (2) Shortcut Cmd/Ctrl+,/B/E/D/O (+ hint di tooltip header) via `shortcuts.ts` murni + test; guard editor/input/current/busy. (3) About: tombol Privacy/Terms/License, isi dari repo-root .md via `sync:legal` (pretest/prebuild) ke `src/lib/app/legal/` (vitest menolak ?raw di luar root); teks legal Indonesia authoritatif; kredit Muse (Meta). LICENSE/ PRIVACY.md/TERMS.md proprietary (pilihan owner).
- Hasil: `pnpm test` 543/543, `pnpm check` 0/0, `cargo check/clippy/test` hijau, dev direstart (menu Rust aktif). Commit lokal `b5d15fb` (+amend keluarkan 2 screenshot liar ke /tmp/kuron-stray). Insiden: path `../legal` vs `./legal`, key `about` hilang saat edit, tombol glossary ikut terhapus (dikembalikan), `let` di tengah chain builder (tambah `;`).
- Lanjut: Push menumpuk di pemilik; smoke: menu Tentang Kuron Studio, shortcut 5 tombol, About 3 bahasa + 3 subview.

## 2026-11-01 — About legal rapi + Markdown mini

- Kerja: User: .md tampil mentah jelek + ganti emoji robot dgn logo Meta. `Markdown.svelte` baru (h1/h2/list/bold + escape, tanpa dep) + test; About subview pakai itu. Kredit: loop Meta SVG (stroke=currentColor, `text-info`) ganti emoji. LICENSE/PRIVACY.md/TERMS.md dipoles; sync:legal. Token `--ks-meta` sempat ditambah lalu dibatalkan (pakai text-info) setelah temukan edge-case lint: util `*-ks-*` pertama di repo, test bandingkan tanpa prefix `--` (lapor Yoga/perlu fix test lain waktu).
- Hasil: `pnpm test` 549/549 (61 file), `pnpm check` 0/0. Commit `ff9b7a5`.
- Lanjut: Push menumpuk di pemilik; smoke About 3 subview + 3 bahasa.

## 2026-11-01 — Logo resmi Meta

- Kerja: User protes loop tiruan bukan logo Meta; minta download yang resmi. Ambil `Meta Platforms Inc. logo (cropped).svg` dari Wikimedia Commons (lolos verifikasi render) ke `kuron-studio/src/meta.svg`; About pakai `<img>` + komentar atribusi merek dagang; SVG tiruan dibuang. Jujur ke user: logo resmi = merek dagang Meta, dipakai untuk atribusi.
- Hasil: `pnpm test` 549/549, `pnpm check` 0/0. Commit `9d2deb5`.
- Kerja: (1) `ProjectCard.svelte`: tombol card tak punya `relative` sehingga progress bar `absolute inset-x-0 bottom-0` menempel ke wrapper luar (`div.relative`) — offside dari card. Tambah `relative` di button + `overflow-hidden rounded-b-md` di track. (2) `render/overlay.ts`: background overlay selalu `Konva.Rect` inset (kotak hitam) walau bubble polygon/ellipse — ganti shape-aware samakan export `paint_patch`: polygon → isi `Konva.Line` polygon, `ellipse` → isi `Konva.Ellipse`, rect → isi penuh. Teks + auto-fit (`computeTextLayout`, `wrap:none`, `txt.height(boxH)` + `verticalAlign middle`) tak berubah.
- Hasil: `pnpm test` 56 file/524 lolos, `pnpm check` 0 error. Rust tak tersentuh. Belum smoke visual (butuh `pnpm tauri dev` manual oleh pemilik).
- Keputusan: Tanpa openspec (bugfix kecil, paritas kanvas→export sudah ada). Tanpa commit/push.
- Lanjut: Smoke `pnpm tauri dev`: cek card project + panel After/Split untuk bubble polygon/ellipse/rect; commit oleh pemilik.


## 2026-11-01 — Toolbar ramping + palet tools sidebar (BELUM COMMIT)

- Kerja (atas perintah jangan asal commit — semua di worktree): toolbar editor hanya Grid/nama/mode/«/Detect/Simpan (ikon + label responsif, tanpa scroll, tanpa kepotong); 5 tool gambar pindah ke palet grid 5 kolom di atas sidebar kanan (ikon SVG konsisten + label, aktif coral, disabled di After + alasan). Icon baru: select/rect/ellipse/freeform/tail/search/save. Detect/Simpan ikon + label (>=md). Badge status sembunyi <400px. Test: draw-tools pin pindah ke panel, mount + palet. Revert FAB 9c042ed (ganggu kanvas + split rusak) + perbaiki 2x rusak bawaan revert.
- Hasil: `pnpm check` 0/0, `pnpm test` 553/553. SENGAJA tidak commit — tunggu oke user.
## 2026-11-01 — Font translate kekecilan-kebesaran: fill 85% + margin (lanjut fix bubble)

## 2026-11-01 — Split mobile: scroll independen (BELUM COMMIT)

- Kerja: Saran disetujui. `syncScroll` return awal saat `resize.columnMode` (pane numpuk vertikal scroll sendiri-sendiri); desktop sebelahan tetap sync. Test source pin diperbarui.
- Hasil: `pnpm check` 0, `pnpm test` 553/553. Tidak commit (instruksi pemilik).

## 2026-11-01 — Editor: aksi pindah ke bottom bar (BELUM COMMIT)

- Kerja: Toolbar atas tinggal Grid + nama + badge + engine (ramping, anti-kepotong). Before/Split/After + « + Detect + Simpan pindah ke EditorActionBar baru (bar bawah kanvas, border-t, tengah, label ciut di sempit). Test pin mode/onSetMode dialihkan toolbar->actionbar. PaneMode tetap milik toolbar.
- Hasil: `pnpm check` 0, `pnpm test` 555/555. Tidak commit (instruksi).


## 2026-11-01 — Editor mobile-only dua baris (BELUM COMMIT)

- Kerja: Koreksi sesuai pemilik: dua baris HANYA mobile (columnMode) — slim toolbar + EditorActionBar di bawahnya; desktop satu baris penuh seperti semula. Toolbar dapat prop `slim`. Test pin dikembalikan ke toolbar + tambah assert cabang mobile.
- Hasil: `pnpm check` 0, `pnpm test` 555/555. Tidak commit.
- Kerja: Auto-fit lama cari font TERBESAR yang muat ~100% kotak (margin cuma padding 4px) → teks mepet outline, kelihatan "kegedean". Perubahan: (1) `contracts/layout.json`: `padding` 4→8, tambah `fillFactor: 0.85`. (2) `text/textLayout.ts`: `LAYOUT_FILL` + opts `fillFactor`; auto-fit (kotak + polygon) membungkus/menyusut thd area virtual kotak-dalam×fill; manual user pakai kotak penuh (tak disusutkan). (3) Rust `render/page.rs`: struct `fill_factor` + `layout_from_text` cerminkan rumus yang sama; test `lo <= 4.0` → ikut `layout().padding`. (4) Test: kontrak TS+`layout_matches_json_contract` assert `fillFactor`; `textLayout.test.ts` 3 test baru (margin ≤ fill×dalam, faktor kecil = font ≤, manual abaikan fill); 2 test lama dipin `fillFactor: 1`/komentar.
- Hasil: Gate hijau — `cargo test` ok, `clippy` bersih, `pnpm test` 56 file/527 lolos, `pnpm check` 0 error. Kanvas recompute tiap redraw (langsung terlihat); layout TERSIMPAN ikut update saat save/translate berikutnya.
- Keputusan: Tanpa openspec (tuning fix lanjutan, dua sisi diubah bersama jaga paritas). Tanpa commit/push.
- Lanjut: Smoke `pnpm tauri dev` + save ulang (Ctrl+S) tiap halaman yang sudah diterjemahkan agar layout tersimpan + export PNG ikut mengecil; commit oleh pemilik.

## 2026-11-01 — Fill 70% + bubble manual tak ikut translate (fix)

- Kerja: (1) `fillFactor` 0.85→0.7 di `contracts/layout.json` — auto-fit kini berhenti di 70% area kotak, margin 30% menelan selisih ukur-vs-render (kasus p011: blok 337.5/346px = 0,9% dari luber). Padding tetap 8. (2) Bug bubble manual tak ke-translate: `api.translatePage` hanya kirim pageId, backend `load_snapshot` baca geometri TERSIMPAN — gambar manual/Detect yang belum Simpan (tombol `Simpan*`) diam-diam tak ikut. Fix: `TranslatePanel` prop opsional `onBeforeTranslate` (abort bila false/cancel); `EditorPanel.save()` kembalikan boolean + `saveBeforeTranslate()` simpan dulu bila dirty, dipasang ke TranslatePanel.
- Hasil: Gate hijau — `cargo test` ok (175 lib + integrasi), `clippy` bersih, `pnpm test` 56 file/527 lolos, `pnpm check` 0 error. Rust ikut berubah (struct `fill_factor` + `layout_from_text` + test `lo<=padding`), paritas kanvas-export terjaga.
- Keputusan: Tanpa openspec (tuning + bugfix kecil). Tanpa commit/push.
- Lanjut: Smoke `pnpm tauri dev` — gambar bubble manual → langsung Translate (harus ikut, tanpa Ctrl+S dulu); save ulang halaman lama agar layout 70% tersimpan + export ikut.

## 2026-11-01 — Provider keys pindah keychain → sqlite terenkripsi (AES-GCM)

- Kerja: (1) Openspec `changes/file-encrypted-provider-keys/` (proposal + tasks). (2) `secrets.rs` tulis ulang: format kolom `enc1:base64(nonce12||ct)` AES-256-GCM, DEK 32-byte di `keys/master.key` 0600 (corrupt = error, bukan regenerasi diam-diam); `read_key`/`has_key` untuk `enc1:` murni file+sqlite TANPA sentuh keychain; keychain hanya dibaca baris legacy. (3) `commands/provider.rs`: save selalu enkripsi ke sqlite (edit-tanpa-key pertahankan verbatim), tambah `migrate_provider_keys` idempoten (keychain/plaintext→`enc1:`, hapus entri keychain); `lib.rs` panggil best-effort saat startup + log count. (4) Dep baru `aes-gcm` + `getrandom 0.2`; `keyring` dipertahankan sementara hanya untuk migrasi. Test baru: roundtrip, tamper GCM, legacy passthrough tanpa buat master key, perm 0600, corrupt-DEK ditolak; hapus `keychain_backend_is_real` (malah menyentuh keychain asli).
- Hasil: Gate hijau — `cargo test` 178 lib ok (1 flaky paralel `untranslated_bubble_save_sets_detected` lolos saat rerun, file tak tersentuh), clippy bersih, `pnpm test` 527 lolos, `pnpm check` 0 error. Tanpa commit/push.
- Keputusan: Threat model jujur — enkripsi lindungi dari pengintip kasual/backup tanpa file kunci, bukan dari attacker se-uid; yang dihilangkan popup Keychain, bukan klaim absolut. Tanpa master password (UX lebih buruk). Hapus `keyring` = follow-up setelah satu rilis.
- Lanjut: Smoke `pnpm tauri dev` — provider lama (keychain) harus termigrasi otomatis sekali (log `provider_keys_migrated`), popup Keychain tidak muncul lagi di get_providers/translate; commit oleh pemilik.

## 2026-11-01 — Branding Kuro: logo + maskot loading

- Kerja: (1) Openspec `changes/kuro-mascot-branding/`. (2) `kuron-studio/icon.svg` ditulis ulang → wajah Kuro adaptasi `prototype/kuro-a.svg` nhasixapp (body dicerahkan agar terbaca di bg gelap) — header (`AppHeader` impor file ini) ikut baru; regenerasi `src-tauri/icons/*` via `tauri icon icon.svg`. (3) Komponen baru `src/lib/ui/KuroMascot.svelte`: SVG inline, pink → `var(--ks-accent)` (ikut tema), kedip 4.2s + napas, mati saat `prefers-reduced-motion`, prop `progress` tampilkan ring ala `kuro-splash.svg`, dekoratif `aria-hidden`. (4) Pasang: BatchPanel ring 52px + persen saat running; TranslatePanel mini 18px di tombol Batal; EditorPanel mini 16px di tombol Detect saat batching. Empty state tidak digambar (DESIGN: empty state = aksi, bukan ilustrasi). (5) `HEX_ALLOWLIST` di `lint.test.ts` beri entri KuroMascot (palet ilustrasi brand, bukan chrome UI) — test kontrak token tetap hijau.
- Hasil: `pnpm test` 532 lolos, `pnpm check` 0 error. Rust tak tersentuh. Tanpa commit/push.
- Keputusan: Tanpa Figma/computer-use — aset SVG sudah ada dan portable; icon native dari `icon.svg` (bukan `logo_app.webp`) agar header ↔ OS konsisten. Warna brand maskot dipertahankan, aksen pink ikut token accent.
- Lanjut: Smoke `pnpm tauri dev` — header logo Kuro, jalan batch (ring + persen), translate + detect (mini maskot di tombol busy); commit oleh pemilik (termasuk file biner icons/).

## 2026-11-01 — Header space-between + update error ke toast

- Kerja: (1) UpdateButton tak lagi render `<span>` inline di samping tombol — hasil cek (sukses/error) tampil sebagai toast fixed via `Snackbar` (prop baru `tone: success|error`, badge `!` + `danger-soft/danger`, token lama). String toast pindah ke i18n 3 bahasa (`checkUpdate/updateLatest/updateReady/updateDone`). (2) Baris switcher+toolbar jadi satu baris `justify-between` tanpa `flex-wrap` (setara Flutter spaceBetween): grup kiri (dropdown + `41 pages`, `whitespace-nowrap`) vs toolbar kanan (`ml-auto`, tanpa `flex-1` supaya ruang bebas jatuh di tengah); bar tetap ukur via `barEl`/`overflowAt` untuk ⋯. Test: `UpdateButton.test.ts` baru (4: tanpa teks samping, error→toast, latest, ready+relaunch), `snackbar.test.ts` +1 (tone error), `AppHeader.test.ts` +1 (justify-between, tanpa wrap, pages+RTL sebaris).
- Hasil: `pnpm test` 563/563 (64 file), `pnpm check` 0 error. Rust tak tersentuh. Tanpa commit/push.
- Keputusan: `{#key}` dibuang — outro `fly` bikin dua `role=status` sesaat (flaky `findByRole`); update pesan in-place. Assert dismissal toast tak dipasang di UpdateButton.test (outro tak selesai di jsdom, animate di-mock) — dicover `snackbar.test.ts`.
- Lanjut: Smoke `pnpm tauri dev` — klik ↻ saat offline (toast error bawah, bar tak bergeser) + resize 768–1100px (41 pages … RTL satu baris, ruang di tengah, sisanya ke ⋯); commit oleh pemilik.

## 2026-11-01 — Fix update di menu ⋯: label + toast global

- Kerja: (1) Akar masalah ganda: baris update di menu ⋯ render `UpdateButton` ikon-saja tanpa teks; toast self-contained di dalam menu ter-clip `overflow-hidden` (dan di bar pun salah posisi karena `backdrop-blur` header jadi containing block `fixed`). (2) `UpdateButton` ditulis ulang presentasional: prop `label` (varian baris menu `role=menuitem` ikon+teks, mirip sibling) + `onToast(message, tone)`; tanpa state/UI toast sendiri. (3) Toast diangkat ke `App.updateToast` → `AlertBanners` (di `<main>`, bebas clip) via prop baru `onUpdateToast` di `AppHeader`; `Snackbar` ekspor tipe `SnackbarTone`. Tanpa string/key baru (label pakai `checkUpdate`).
- Hasil: `pnpm test` 566/566, `pnpm check` 0 error. Rust tak tersentuh. Tanpa commit/push.
- Keputusan: Satu kanal toast di `AlertBanners` (pemilik semua banner) ketimbang portal-`@attach` manual — ikut pola repo (state di App, aksi via callback).
- Lanjut: Smoke `pnpm tauri dev` — sempitkan window sampai ↻ masuk ⋯, klik "Cek update" (baris berteks, toast muncul bawah layar); commit oleh pemilik.

## 2026-11-01 — Samakan tinggi baris update di menu ⋯

- Kerja: Baris "Check for update" lebih pendek karena ikonnya glyph teks `↻` (line box ±16px) sementara baris lain pakai `<Icon>` SVG `size-5` (20px). Tambah ikon `refresh` (Feather rotate-cw, stroke currentColor) ke set `Icon.svelte` + daftar `Icon.test.ts`; baris menu `UpdateButton` pakai pola sibling persis (`contents text-ink-3` + Icon default md). Test label diperketat (svg + `size-5`, bukan glyph).
- Hasil: `pnpm test` 566/566, `pnpm check` 0 error. Rust tak tersentuh. Tanpa commit/push.
- Lanjut: Smoke visual menu ⋯ (semua baris sama tinggi); commit oleh pemilik.

## 2026-11-01 — Check-update masuk drawer + hapus Detect ganda

- Kerja: (1) Akar drawer: class `.drow` scoped di `<style>` AppHeader tak berlaku di child `UpdateButton`, jadi dipindah ke `app.css` global (ikut preseden `.tnum`; aman untuk `lint.test.ts` yang hanya parse token `--ks-*`). (2) `UpdateButton` tambah `kind: "drawer"` (render `.drow` + ikon refresh + label) dan `onDone` (tutup drawer/menu saat diklik; cabang menu ⋯ juga panggil `onDone`). (3) Drawer: baris update setelah Providers (gate `!isMobile`), menu ⋯ teruskan `onDone` tutup menu. (4) Bar sempit: tombol "Detect all" dihapus — tinggal ☰; CTA Detect satu-satunya di drawer (baris primer 48px yang sudah ada).
- Hasil: `pnpm test` 568/568, `pnpm check` 0 error/warning (1023 file). Rust tak tersentuh. Tanpa commit/push.
- Keputusan: Detect di drawer cukup (sudah CTA primer penuh); bar sempit minimal ☰ saja.
- Lanjut: Smoke `pnpm tauri dev` mode sempit — drawer berisi Detect sekali + baris "Cek update"; bar hanya ☰; commit oleh pemilik.
