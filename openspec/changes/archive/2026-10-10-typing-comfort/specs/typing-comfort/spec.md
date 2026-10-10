## ADDED Requirements

### Requirement: Mengetik tanpa commit tengah-ketik

Mengetik di textarea kartu (original/reading/translated) SHALL tidak memicu commit/render sampai batas alami: blur, pindah bubble, tombol Save, `compositionend`, atau timer-jaring 10 dtk. Satu kalimat penuh SHALL selesai tanpa interupsi.

#### Scenario: Ketik 1 kalimat penuh tanpa jeda-paksa

- **WHEN** user mengetik 1 kalimat penuh di Terjemahan (ID) dengan jeda mikro alami antar kata (tanpa blur)
- **THEN** tidak ada commit/render yang fire selama mengetik; seluruh huruf tampil lengkap-berurutan, kursor stabil; commit SHALL terjadi sekali saat blur/pindah/Save

#### Scenario: Timer-jaring menyelamatkan sesi ditinggal

- **WHEN** user mengetik lalu meninggalkan kartu tanpa blur selama >10 dtk (tanpa pindah/Save/tutup)
- **THEN** timer-jaring SHALL commit + antre save tepat sekali; ketikan terselamatkan ke SQLite

### Requirement: Indikator dirty dan Save flush eksplisit

Saat cache kotor (ada ketikan belum commit), UI SHALL menunjukkan status belum-tersimpan; tombol Save SHALL commit + save langsung.

#### Scenario: Indikator menyala saat mengetik

- **WHEN** user mengetik (cache kotor, belum commit)
- **THEN** indikator dirty SHALL tampil (konvensi `Simpan*`/badge); setelah commit+save hijau, indikator SHALL padam

#### Scenario: Save flush eksplisit

- **WHEN** user klik Save saat cache kotor
- **THEN** sistem SHALL commit dulu lalu save langsung (tanpa menunggu timer-jaring); teks tersimpan ke SQLite

### Requirement: Sync prop tak menimpa ketikan berjalan

`syncFromPage` cabang `translation` SHALL skip bila cache kotor (`draftBox.cur`) ATAU save terbang (`saveTimer`/`savingTr`); cabang `bubbles` tak berubah. Saat tenang, sync jalan normal (echo/mount/batch/retry lolos; kabar batch tertunda, bukan hilang).

#### Scenario: Ketikan tak balik ke sebelumnya pasca-save

- **WHEN** user mengetik 1 kalimat, commit, lalu mengetik lanjutan saat save terbang (prop echo basi tiba di jendela commit→save)
- **THEN** lanjutan SHALL tidak ditimpa prop basi; save berikutnya SHALL mengirim teks lengkap (baru + lanjutan); kartu tetap tampil kalimat baru

#### Scenario: Hasil batch masuk saat tenang

- **WHEN** batch/retry selesai (`refreshProject`) saat tak ada ketikan kotor dan tak ada save terbang
- **THEN** hasil baru SHALL tampil di kartu/After via sync normal
