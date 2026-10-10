## Why

Editor in-bubble (textarea melayang di After) basi saat user pindah tab Before|Split|After selagi editor terbuka: posisi textarea di-snapshot sekali di `openEditor` dan tak pernah dihitung ulang, sementara lebar pane berubah → textarea "tetap di posisi yang sama". Redraw Konva yang tertunda (via save debounce 500ms) lalu memecah konsistensi: bubble melompat ke posisi baru, textarea tertinggal. Keputusan desain yang diambil saat explore: **editor tidak boleh tetap terbuka saat ganti tab — commit otomatis** (kutub defensif, opsi C bila perlu).

## What Changes

- Ganti tab (`mode` berubah di `EditorPanel`) → commit editor in-bubble yang terbuka di instance After sebelum pane berganti (ketikan selamat via jalur `editTr` existing, tanpa yank — seleksi sudah dikunci fix sebelumnya).
- `CanvasEditor` expose fungsi commit agar `EditorPanel` bisa memanggilnya via ref (instance After hari ini tidak di-bind; hanya Before yang punya `editorRef`).
- Redraw canvas After setelah lebar settle (batasan: tanpa `ResizeObserver` bila commit-cukup; ROb hanya bila diputuskan perlu saat desain).
- Test pola-sumber: ganti tab → commit dipanggil; textarea tak basi (kontrak), + skenario E2E manual.

## Capabilities

### New Capabilities

- `pane-switch-commit`: ganti tab Before|Split|After meng-commit editor in-bubble yang terbuka (ketikan selamat, tak ada textarea basi).

### Modified Capabilities

- `in-bubble-edit`: perilaku editor saat konteks pane berubah (sebelumnya tak didefinisikan).

## Scope

- `kuron-studio/src/lib/EditorPanel.svelte` — panggil commit After saat `mode` berubah + bind ref instance After.
- `kuron-studio/src/lib/CanvasEditor.svelte` — expose `commitEditor` (export function) bila belum.
- Test: `edit-in-bubble.test.ts` / file baru pola-sumber + E2E manual.
- Kait ide/target: alur edit-cepat tanpa kejutan posisi (docs/ide.md: workbench cepat; docs/target.md: provider translate tanpa friksi).

## Non-goals

- After-true (PNG typeset) — tetap after-lite overlay Konva.
- `ResizeObserver` penuh + reposisi textarea live — hanya bila commit-cukup diprotes (ponytail).
- Ubah jalur save/backend (`save_translation`, debounce 500ms, baseline `ai*`).
- Editor di Before.

## Risks

- **Ketikan <500ms hilang?** Tidak — commit memanggil jalur `editTr` yang sama dengan blur, jadi ketikan justru diselamatkan, bukan dibuang. Risiko sisa: commit memicu save → yank-check (sudah diperbaiki fix sebelumnya).
- **Ref After belum ada**: `bind:this` kedua perlu nama berbeda (`afterRef`); pastikan tak bentrok `editorRef` (Before, dipakai `deleteSelected`).
- **Test pola-sumber tak membuktikan posisi nyata** — E2E manual wajib (skenario di tasks).
