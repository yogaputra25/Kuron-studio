## Why

Mengetik 1 kalimat penuh di kartu Terjemahan (ID) terpotong/tertunda: tiap jeda mikro antar kata memicu commit (`draftTimer` 500ms → `editTr` → render ulang kartu) tepat saat jari mengetik kata berikutnya. Cache plain `fix-card-input-lag` menghilangkan render *per huruf* tapi tidak render *per commit*. User memilih kenyamanan: mengetik harus tanpa interupsi render sama sekali.

## What Changes

- Hapus commit-tengah-ketik: `draftTimer` 500ms dihapus; commit hanya via blur / pindah bubble / tombol Save / `compositionend` / timer-jaring panjang (10 dtk, sebagai pengaman crash/lupa, bukan ritme kerja).
- Sync-guard cabang `translation` di `syncFromPage`: tunda timpa dari prop saat cache kotor (`draftBox.cur`) ATAU save terbang (`saveTimer`/`savingTr`) — pelaku "ketikan balik ke sebelumnya" (prop basi menimpa state lebih baru di jendela commit→save). Kabar batch/retry tetap masuk saat kondisi tenang (tak ada data hilang, hanya tertunda).
- Autosave (`queueSave` 500ms → `save_translation`) tetap hidup, hanya fire pasca-commit. Kontrak invoke L1/L2, baseline `ai*`, `saveGen` guard tidak berubah.
- Tombol Save menjadi flush eksplisit (commit + save langsung); indikator "belum tersimpan" bila cache kotor (draft ada tapi belum commit).

## Capabilities

### New Capabilities

- `typing-comfort`: mengetik kartu tanpa render/commit tengah-ketik; commit on blur/pindah/Save/compositionend + timer-jaring 10 dtk; indikator dirty.

### Modified Capabilities

- `responsive-card-input`: ritme commit berubah (debounce-per-jeda dihapus, diganti commit-on-blur + timer-jaring); kontrak commit/blurb/save tetap.

## Scope

- `kuron-studio/src/lib/EditorPanel.svelte` — `draftInput` (tanpa timer 500ms), `commitDraft`, `compEnd`, `setMode`/`selectBubble`/`saveTranslationEdits` (flush eksplisit), tombol Save + indikator dirty, `onDestroy` flush.
- Test: `edit-stuck.test.ts` kontrak diperbarui + kontrak baru timer-jaring/indikator.
- Kait ide/target: workbench edit-cepat tanpa friksi ketik (docs/ide.md: edit-cepat; docs/target.md: provider translate tanpa hambatan).

## Non-goals

- Hapus autosave total (V3 ditolak — beban ingat-save ke user, kehilangan lebih besar).
- Ubah backend Rust / kontrak invoke / baseline `ai*` / `saveGen` guard.
- Ubah editor in-bubble. Virtualisasi `aside`. After-true / ROb.
- Timer 500ms lama dipertahankan berdampingan (satu ritme saja: jaring 10 dtk).

## Risks

- **Crash sebelum blur**: ketikan ≤10 dtk terakhir bisa hilang bila kill paksa sebelum timer-jaring fire. Diterima user (pilih kenyamanan); timer-jaring 10 dtk + flush-on-destroy memitigasi kasus tutup normal.
- **Badge/overlay tertunda**: `edited` + overlay After + autosave baru muncul setelah blur/pindah/Save/10 dtk — teks di bubble tidak live-update mengikuti ketikan. Diterima sebagai harga kenyamanan.
- **Kalimat >10 dtk**: 1 commit tengah (bukan N) — gangguan minimal, jauh di bawah status quo.
- **Test pola-sumber tak membuktikan latensi** — E2E manual terukur wajib (kriteria di tasks).
