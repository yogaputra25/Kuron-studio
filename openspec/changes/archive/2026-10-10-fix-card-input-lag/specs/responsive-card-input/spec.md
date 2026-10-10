## ADDED Requirements

### Requirement: Ketikan kartu responsif tanpa render per huruf

Textarea kartu (`original`/`reading`/`translated`) SHALL menampilkan huruf di frame yang sama saat tombol ditekan, tanpa update reactive per huruf; commit tetap via jalur `editTr` existing (debounce/blur/pindah seleksi).

#### Scenario: Ketik-cepat satu kalimat

- **WHEN** user mengetik satu kalimat cepat di Terjemahan (ID) tanpa jeda
- **THEN** seluruh huruf SHALL tampil lengkap dan berurutan, kursor stabil di posisi ketik, tanpa huruf hilang/tertukar

#### Scenario: Autosave tak mengganggu ketikan berjalan

- **WHEN** user mengetik terus melewati batas debounce (commit + save fire + respons tiba saat masih mengetik)
- **THEN** ketikan SHALL tidak terpotong, kursor tidak melompat, textarea fokus tidak di-render ulang oleh respons save

### Requirement: Komposisi IME tak terpotong commit

Selama komposisi IME aktif (`isComposing`), commit SHALL ditunda sampai `compositionend`; teks komposisi SHALL utuh.

#### Scenario: Ketik dengan IME aktif

- **WHEN** user mengetik memakai IME (komposisi belum di-commit browser) melewati batas 500ms
- **THEN** tidak ada commit yang fire di tengah komposisi; commit SHALL jalan setelah `compositionend` via ritme timer yang sama

### Requirement: Sinkronisasi eksternal tak menimpa fokus

Hasil Translate baru, ↺ reset, dan ganti bubble SHALL mengikuti matriks D3: draft kotor di-commit dulu; DOM textarea yang sedang fokus SHALL tidak ditulis ulang oleh respons save/hasil luar dengan konten identik.

#### Scenario: Hasil Translate tiba saat mengetik

- **WHEN** hasil `translatePage` tiba saat user sedang mengetik di kartu (draft kotor)
- **THEN** draft SHALL di-commit dulu lalu merge (`mergeUserEdits`) melindungi teks yang sedang diketik; tidak ada huruf hilang

#### Scenario: Pindah bubble saat draft kotor

- **WHEN** user klik bubble lain saat draft kotor
- **THEN** draft SHALL di-commit via `editTr` dulu, kartu baru menampilkan teks bubble baru, tanpa huruf tertinggal di kartu lama
