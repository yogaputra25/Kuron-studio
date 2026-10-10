## ADDED Requirements

### Requirement: Ganti tab meng-commit editor in-bubble yang terbuka

Ganti tab Before|Split|After di editor SHALL meng-commit editor in-bubble yang sedang terbuka di pane After sebelum pane berganti, via jalur `editTr` existing (tanpa yank seleksi).

#### Scenario: Split → After dengan editor terbuka dan ada ketikan

- **WHEN** user membuka editor in-bubble di bubble C saat mode Split, mengetik teks baru, lalu klik tab After
- **THEN** teks ketikan SHALL tersimpan ke bubble C (`translated`, `isUserEdited: true`) dan tidak ada textarea basi yang tertinggal di posisi lama

#### Scenario: Ganti tab tanpa editor terbuka

- **WHEN** user klik tab lain saat tidak ada editor in-bubble yang terbuka
- **THEN** ganti tab SHALL berjalan seperti biasa tanpa save tambahan, tanpa badge `edited` palsu

#### Scenario: Ganti tab dengan editor terbuka tapi teks tak berubah

- **WHEN** user membuka editor lalu langsung ganti tab tanpa mengetik
- **THEN** editor SHALL tertutup tanpa menulis state, tanpa menandai `edited`, tanpa antre save

### Requirement: Kanal commit dari panel ke canvas After

`EditorPanel` SHALL memegang ref ke instance `CanvasEditor` After dan memanggil commit-nya secara synchronous di handler ganti tab, sebelum `mode` berubah.

#### Scenario: Ref After terpasang

- **WHEN** pane After dirender (mode after atau split)
- **THEN** ref commit After SHALL tersedia untuk handler ganti tab; ref Before (`deleteSelected`) SHALL tetap terpisah dan tidak terpengaruh

### Requirement: Canvas mentok pane pasca-ganti-tab

Ganti tab SHALL diiringi redraw canvas (overlay + background satu scale) setelah layout settle, sehingga stage selebar holder.

#### Scenario: Split → After canvas mentok pane

- **WHEN** user pindah tab Split → After (dengan atau tanpa editor terbuka)
- **THEN** stage After SHALL selebar holder (`stage.width ≈ holder.clientWidth`): gambar + overlay bubble mentok pane, tanpa ruang kosong/hitam di kanan; background image dan overlay SHALL dalam scale yang sama (tak ada split-brain)
