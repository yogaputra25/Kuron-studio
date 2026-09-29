## Why

Sidebar panel "translated" hari ini menampilkan N kartu per bubble
(`EditorPanel.svelte` ~438–488: Original JP + Reading + Terjemahan per bubble),
sehingga panel memanjang ke bawah untuk halaman dengan banyak bubble.
Padahal teks terjemahan SUDAH tampil di pane After sebagai overlay Konva
(after-lite). User meminta: hapus list panjang, edit langsung di badan bubble.

Keputusan user terkunci (explore 2026-09-29):
1. Yang diedit di badan bubble = `translated` saja.
2. Sidebar sisakan 1 kartu untuk bubble terpilih (`manualSel`).
3. `Save edits` tetap ada (tombol flush manual + autosave debounce tetap jalan).
4. Mulai edit = double-klik; Esc = batal (standar); commit = blur / Ctrl+Enter.

Kait ke change `reset-translation` (baseline `ai*` + tombol ↺ + cancel opsi B):
jalur save (`editTr` → debounce 500ms → `save_translation`) dipakai ulang
tanpa perubahan — yang berubah hanya *dari mana* `editTr` dipanggil.
Baseline `ai*`, guard ↺ (`isUserEdited && aiTranslated`), dan autosave
tetap utuh.

## What Changes

- Pane After (`CanvasEditor`, `editable={false}`): double-klik bubble →
  `<textarea>` HTML melayang di posisi bubble (koordinat original × scale),
  tema ikut `needsWhitePatch`, font meniru clamp `fs` overlay existing.
- Ketikan ditahan di state lokal textarea; commit sekali saat blur /
  Ctrl+Enter → `editTr(i, "translated", v)` existing; Esc = batal tanpa tulis.
- Klik bubble lain saat editor terbuka = commit otomatis (anti teks hilang).
- Sidebar: `<ol>` N kartu → `{#if selectedBubble}` tunggal berisi
  Original/Reading/Translated + ↺ + badge `edited` + glossary tahan-0.5-dtk
  (semua logika existing pindah rumah, bukan ditulis ulang).
- Header `translated` + `Save edits` + hint overlay tetap (baris ~427–437).
- Split mode: editor teks hanya di After; klik Before = seleksi saja.

## Scope

- `kuron-studio/src/lib/CanvasEditor.svelte` — props kabel edit
  (`onEditTranslation(i, value)?` + sinyal double-klik), textarea melayang,
  posisi × scale, tema `needsWhitePatch`, font ikut `fs` existing.
- `kuron-studio/src/lib/EditorPanel.svelte` — N kartu → 1 kartu terpilih;
  handler commit dari canvas → `editTr(i, "translated", v)`; header tetap.
- Test: vitest pola-sumber (double-klik handler, commit→`editTr`,
  Esc tanpa tulis, 1 kartu terpilih, `Save edits` tetap ada) +
  Rust tidak perlu (tanpa perubahan backend).

## Non-goals

- Edit `original`/`reading` di badan bubble (tetap di kartu tunggal).
- Mengubah jalur save/backend (`save_translation`, baseline `ai*`, autosave
  500ms, cancel) — dipakai ulang apa adanya.
- After-true (render PNG typeset) — tetap after-lite overlay Konva.
- Editor teks di pane Before.
- Menghapus tombol `Save edits` (tetap sebagai flush manual).

## Risks

- **Fokus amblas per keystroke**: effect `translations → redraw()`
  me-rebuild overlay tiap state berubah. Mitigasi: ketikan di state LOKAL
  textarea, `translation.bubbles` baru ditulis sekali saat commit.
- **Scale/posisi meleset**: `scale()` hari ini internal `CanvasEditor`
  (`holder.clientWidth / imgW`); textarea butuh scale yang sama + hitung
  ulang saat resize. Mitigasi: hitung di dalam canvas (sumber scale),
  textarea absolute relatif ke stage container (ikut scroll natural).
- **Font/wrapping beda dengan overlay**: overlay pakai
  `fs = clamp(10..18, w*s / …)`. Mitigasi: textarea pakai rumus `fs` yang
  sama agar WYSIWYG.
- **Bubble mungil, teks panjang**: overlay existing melebar ke bawah
  (`max(b.h*s, txt.height()+6)`). Mitigasi: textarea boleh lebih tinggi
  dari bubble, bukan dipaksa pas.
- **Keyboard virtual / WebView Tauri**: textarea HTML justru lebih aksesibel
  daripada edit Konva murni, tapi wajib tes nyata di `pnpm tauri dev`.
