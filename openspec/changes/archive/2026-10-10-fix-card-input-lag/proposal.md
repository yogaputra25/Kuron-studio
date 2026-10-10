## Why

Mengetik di textarea kartu Terjemahan (ID) kadang terasa delay: huruf muncul telat setelah tombol ditekan. Jalur ketik hari ini (`draftInput` → `draft[field] = v` per huruf) memicu update `$state` + evaluasi ulang seluruh subtree `aside` tiap keystroke, dan respons `save_translation` (~1 dtk pasca-jeda) mengganti objek `translation` utuh tepat saat user masih mengetik — frame ketik tertabrak render besar. Bukan autosave sebagai penyebab tunggal, tapi rantai draft-per-huruf + render-kartu + respons-save yang memperkuat.

## What Changes

- Ketikan kartu tidak lagi memicu update reactive per huruf: textarea menjadi uncontrolled (nilai DOM milik browser), state draft disinkronkan tanpa me-render ulang kartu.
- Commit tetap via jalur existing (`commitDraft` → `editTr` → debounce 500ms → `save_translation`); ritme autosave, badge `edited`, ↺, glossary tidak berubah.
- Respons `save_translation` tidak me-render ulang textarea yang sedang fokus (nilai DOM tak disentuh selama fokus).
- Guard IME/composition: selama `isComposing`, commit ditunda sampai `compositionend`.
- Test pola-sumber + kriteria E2E angka (latensi ketik).

## Capabilities

### New Capabilities

- `responsive-card-input`: textarea kartu (original/reading/translated) responsif per huruf tanpa jank, commit via jalur existing, aman IME.

### Modified Capabilities

- `edit-draft` (fix-edit-stuck): pola draft-vs-render kartu berubah (draft tak lagi di-`value=` per huruf); kontrak commit/blurb/debounce tetap.

## Scope

- `kuron-studio/src/lib/EditorPanel.svelte` — textarea kartu + `draftInput`/`commitDraft`/render `value=`.
- Test: `edit-stuck.test.ts` / file kontrak baru + E2E manual terukur.
- Kait ide/target: workbench cepat tanpa friksi ketik (docs/ide.md: edit-cepat; docs/target.md: provider translate tanpa hambatan).

## Non-goals

- Ubah jalur save/backend (`save_translation`, debounce 500ms, baseline `ai*`, `saveGen` guard).
- Ubah editor in-bubble (`bind:value` sudah dua arah, tak terdampak).
- Virtualisasi daftar bubble / rewrite `aside` (optimasi render besar-besaran).
- After-true, ResizeObserver, backend Rust.

## Risks

- **Uncontrolled textarea vs Svelte reactivity**: nilai DOM milik browser — commit harus baca dari event/DOM, bukan dari `value=`; risiko draft-vs-DOM drift bila `translation` diganti dari luar (hasil Translate baru, ↺) saat user mengetik. Mitigasi: sinkronisasi eksplisit saat `selectedRow`/bubble berubah + saat respons save tiba (hanya bila tak fokus).
- **Test pola-sumber tak membuktikan latensi nyata** — E2E manual terukur wajib (kriteria ms di tasks).
- **IME**: tanpa guard `isComposing`, commit tengah-komposisi memotong karakter CJK; guard wajib, bukan opsional.
