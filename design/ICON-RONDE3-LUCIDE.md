# Ronde 3 — 31 ikon Lucide, sisa glyph dimigrasi (2026-11-07)

Sumber: Lucide v0.469.0 (ISC), pola ronde 2 (vector path pra-skala ×0.5,
stroke 1.5 round caps/joins). Frame 16×16 (vektor 12 di offset 2,2) untuk
ikon tombol; frame 12×12 (vektor penuh) untuk X/±/chip/checkbox agar muat
baris 13px. Warna stroke = fill glyph yang diganti. Eksekusi headless via
`figcraft-cli run` (236 op, semua `ok:true`, `next_id` 1649→1716, 1541→1592
node). Tanpa ubah copy (kecuali buang prefix glyph), layout, token; tanpa
commit. Backup: `/tmp/ronde3-start.figcraft` (pra), `/tmp/ronde3-backup-pre.figcraft`,
`/tmp/kuron-studio-backup-ronde2.figcraft`, `/tmp/kuron-studio-backup-ronde3.figcraft`.

Batasan: port kontrol live FigCraft (9880) mati sejak awal sesi (proses jalan
tanpa listener) — reload live tak mungkin tanpa restart, yang dilarang.
Verifikasi via render headless `figcraft-cli export` (7 PNG @2x dibaca semua)
+ `cmp` biner deploy-vs-review. Aplikasi FigCraft yang terbuka masih
menampilkan state lama — buka ulang file untuk melihat hasil.

## Dimigrasi (glyph lama → Lucide)

| # | Frame | Glyph lama (node, teks) | Ikon baru (frame → vektor, lucide) | Warna |
|---|---|---|---|---|
| 1 | Modal 05 close | 1345/1346 rect-X di 908 XIcon (12×12, posisi sama) | 908 → 1649 lu-x | x, abu |
| 2 | Modal 07 close 1276 BtnClose7 | 1277 `✕` (dihapus) | 1650 IcoX_1276 → 1651 lu-x | x, abu |
| 3 | ZoomOut 425 | 426 `−` (dihapus) | 1652 IcoMinus_425 → 1653 lu-minus | minus, abu |
| 4 | ZoomIn 428 | 429 `+` (dihapus) | 1654 IcoPlus_428 → 1655 lu-plus | plus, abu |
| 5 | Chip 07 | 1293 `[x] Lewati 0-Balon Kata…` → `Lewati…` | 1656 Chip_1293 / 1657 → 1658 lu-x | x, abu |
| 6 | Chip 07 | 1294 `[x] TM Cache Hit…` → tanpa prefix | 1659 Chip_1294 / 1660 → 1661 lu-x | x, hijau |
| 7 | Chip 07 | 1295 `[x] Lewati Selesai…` → tanpa prefix | 1662 Chip_1295 / 1663 → 1664 lu-x | x, redup |
| 8 | Checkbox 05 | 940 `[x] Tambahkan Bleed…` → tanpa prefix | 1665 Row_940 / 1666 → 1667 lu-square-check-big | abu-hijau |
| 9 | Checkbox 05 | 941 `[x] Konversi Separasi…` → tanpa prefix | 1668 Row_941 / 1669 → 1670 lu-square-check-big | abu |
| 10 | Checkbox 05 | 942 `[ ] Sertakan Halaman…` → tanpa prefix | 1671 Row_942 / 1672 → 1673 lu-square | redup |
| 11–14 | Stages 02 | 147/151/155/159 `→` (dihapus) | 1674/1676/1678/1680 IcoArr → 1675/77/79/81 lu-arrow-right | arrow-right, gelap |
| 15–16 | Strip 07 | 1298/1300 `→` (dihapus) | 1682/1684 IcoArr → 1683/85 lu-arrow-right | arrow-right, gelap |
| 17 | BtnNew 29 | 30 `+ Project Baru` → `Project Baru` | 1686 IcoPlus_29 → 1687 lu-plus | plus, PUTIH (coral) |
| 18 | BtnBack 418 | 419 `← Grid` → `Grid` | 1688 IcoBack_418 → 1689 lu-arrow-left | arrow-left, abu |
| 19 | PNext 750 | 751 `Next >` → `Next` (+ikon sesudah) | 1690 IcoNext_750 → 1691 lu-chevron-right | chevron-right, abu |
| 20 | It2 1056 | 1057 `✓ DeepSeek Chat…` → tanpa `✓ ` | 1692 IcoCheck_1056 → 1693 lu-check | check, terang |
| 21 | DropTrig 131 | 132 `ONIMASA Ch. 12 ▾` → tanpa ` ▾` | 1694 IcoChev_131 → 1695 lu-chevron-down | chevron-down, terang |
| 22 | FontSel 530 | 531 `CC Wild Words ▾` → tanpa ` ▾` | 1696 IcoChev_530 → 1697 lu-chevron-down | chevron-down, abu |
| 23 | HL 763 | 765 `ONIMASA Ch. 12 ▾` → tanpa ` ▾` | 1698 IcoChev_765 → 1699 lu-chevron-down | chevron-down, abu |
| 24 | HL 1130 | 1132 `ONIMASA Ch. 12 ▾` → tanpa ` ▾` | 1700 IcoChev_1132 → 1701 lu-chevron-down | chevron-down, abu |
| 25 | CBInp 1050 | 1052 `▾` (dihapus) | 1702 IcoChev_1050 → 1703 lu-chevron-down | chevron-down, redup |
| 26–29 | PBox/MBox/LBox/StyBox | 1308/1311/1315/1318 `▾` (dihapus) | 1704/06/08/10 IcoChev → 1705/07/09/11 lu-chevron-down | chevron-down, redup |
| 30 | BtnCancel7 1338 | (tanpa ikon) + `Batal (Esc)` tetap | 1712 IcoX_1338 → 1713 lu-x | x, abu |
| 31 | BtnCancel 947 (05) | (tanpa ikon, konsistensi dgn 07) + `Batal (Esc)` tetap | 1714 IcoX_947 → 1715 lu-x | x, abu |

Catatan: path `minus` degenerat (tanpa tinggi) — pra-skala ×0.5 manual +
tanpa param ukuran (`width`/`height` ditolak engine utk path 0-ekstensi).
`square-check-big`/`square` memakai arc `A` — engine render arc benar
(diverifikasi zoom PNG).

## Dipertahankan + alasan (tipografi/bukan ikon)

- Kbd-hint: `⌘K` 137, `⌘N` 117, `⌘R` 539, `⌘Enter` 541/1102, `⌘F` 563,
  `⌘D` 1071, `⌘` 1096, `⌫` 1075 (merah, sel hint hapus), `(F3)` 59/139/768/1136,
  `(Enter)` 404/405, `(Esc)` 948/1339/1086/1090, `(G)/(P)/(D)` 477/479/481.
  → teks shortcut mono, bukan affordance ikon.
- Panah inline: `JA → ID` 134 (arah bahasa), `→` 536 (asli→hasil). → tipografi.
- Minus aritmetik: `−13 Dilewati` 1299, `−13` di strip (bilangan negatif). → tipografi.
- Tanda baca/data: `[14:22]`/`[007/182]`/`[X:142 Y:280]`/`(52 hlm)`/`(Muat)`/
  `(Sesak)`/`(118%)`/`…`/`·`/`—`/`•` (butir DS 06) — isi teks, bukan ikon.
- Radio-dot seleksi: dot 145/… di stage pills 02, dot O1LG/O2LG, dot 18
  status — indikator status kecil, setara ronde 2 (TipIcon 1462 circle-i
  dipertahankan).
- `⌘` 1096 di CPInp 06: ilustrasi command-palette di layar dok DS, bukan UI app.
- Ikon package CBZ 1634 (ronde 2): render identik pra/posa (`cmp` PNG) —
  gaya bawaan Lucide `package` di ukuran kecil, di luar lingkup ronde 3.

## Bukti visual

`design/previews-clean/` (7 PNG @2x dari file deploy, `cmp` cocok dgn
yang direview): 01-dashboard, 02-page-grid, 03-workbench, 04-ledger,
05-export, 06-token-guide, 07-batch — deretan tombol akhir 07 semua
berikon Lucide sejajar; chip ×, checkbox, chevron, panah sejajar label;
tanpa copy/layout/token berubah (lebar tombol hug menyesuaikan).
