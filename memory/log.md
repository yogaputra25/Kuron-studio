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
