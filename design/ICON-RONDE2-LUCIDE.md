# Ronde 2 — 37 ikon Lucide (2026-11-07)

Sumber: Lucide v0.469.0 (ISC), `https://unpkg.com/lucide-static@0.469.0/icons/<nama>.svg`,
diunduh ke /tmp/lucide/ (tidak di repo). Konversi: viewBox 24 → grid 16 (×2/3),
circle/line/polyline → subpath SVG, stroke 1.5px round caps/joins, fill none.
Abu `#A5A6B0`, putih `#FFFFFF` di atas CTA coral. Dukungan path dibuktikan:
`node.create {type vector, path}` ter-render (uji search + reload + baca PNG).
Omit: 0. Reload akhir: `repairs: []`. Backup: /tmp/kuron-studio-backup-ronde2.figcraft.

## Per slot (frame → node vector baru → lucide@0.469.0)

| # | Frame | Vector | Lucide | Warna |
|---|---|---|---|---|
| 1 | 1482 IcoTrayup_110 | 1612 lu-upload | upload | abu |
| 2 | 1485 IcoChev_56 | 1613 lu-chevron-right | chevron-right | PUTIH (coral) |
| 3 | 1489 IcoList_58 | 1614 lu-list | list | abu |
| 4 | 1492 IcoChev_80 | 1615 lu-chevron-right | chevron-right | abu |
| 5 | 1497 IcoDl_82 | 1616 lu-download | download | abu |
| 6 | 1500 IcoScan_104 | 1617 lu-scan-line | scan-line | PUTIH (coral) |
| 7 | 1503 IcoSearch_136 | 1618 lu-search | search | abu |
| 8 | 1507 IcoList_138 | 1619 lu-list | list | abu |
| 9 | 1510 IcoBubble_140 | 1620 lu-message-circle | message-circle | PUTIH (coral) |
| 10 | 1516 IcoScan_402 | 1621 lu-scan-line | scan-line | abu |
| 11 | 1519 IcoChev_404 | 1622 lu-chevron-right | chevron-right | PUTIH (coral) |
| 12 | 1468 IcoSearch (SearchBox) | 1623 lu-search | search | abu |
| 13 | 1522 IcoScan_431 | 1624 lu-scan-line | scan-line | abu |
| 14 | 1526 IcoSave_433 | 1625 lu-save | save | PUTIH (coral) |
| 15 | 1529 IcoBook_476 | 1626 lu-book-open | book-open | abu |
| 16 | 1532 IcoGear_478 | 1627 lu-settings | settings | abu |
| 17 | 1535 IcoBang_480 | 1628 lu-circle-alert | circle-alert | abu |
| 18 | 1538 IcoScan_538 | 1629 lu-scan-line | scan-line | abu |
| 19 | 1541 IcoChev_540 | 1630 lu-chevron-right | chevron-right | PUTIH (coral) |
| 20 | 1546 IcoDl_557 | 1631 lu-download | download | abu |
| 21 | 1551 IcoCheck2_559 | 1632 lu-check-check | check-check | PUTIH (coral) |
| 22 | 1476 IcoDoc (O1LG) | 1633 lu-file-text | file-text | abu |
| 23 | 1578 IcoBox_917 (O2LG) | 1634 lu-package | package | abu |
| 24 | 1582 IcoFolder_923 (O3LG) | 1635 lu-folder | folder | abu |
| 25 | 1555 IcoSave_950 | 1636 lu-save | save | abu |
| 26 | 1560 IcoDl_952 | 1637 lu-download | download | PUTIH (coral) |
| 27 | 1471 IcoOpen (MI1L) | 1638 lu-chevron-right | chevron-right | abu |
| 28 | 1586 IcoScan_1069 | 1639 lu-scan-line | scan-line | abu |
| 29 | 1590 IcoTrash_1073 | 1640 lu-trash-2 | trash-2 | abu |
| 30 | 1564 IcoUndo_1083 | 1641 lu-undo-2 | undo-2 | abu |
| 31 | 1567 IcoTrash_1091 | 1642 lu-trash-2 | trash-2 | PUTIH (coral) |
| 32 | 1594 IcoBubble_1100 | 1643 lu-message-circle | message-circle | abu |
| 33 | 1599 IcoList_1105 | 1644 lu-list | list | abu |
| 34 | 1605 IcoDl_1110 | 1645 lu-download | download | abu |
| 35 | 1609 IcoGear_1115 | 1646 lu-settings | settings | abu |
| 36 | 1571 IcoChart_1341 | 1647 lu-chart-column | chart-column | abu |
| 37 | 1575 IcoGo_1343 | 1648 lu-arrow-right | arrow-right | PUTIH (coral) |

Dipertahankan: 1462 TipIcon (circle-i ronde 1, tajam — setara Lucide info).
Disengaja tanpa ikon: BtnDetectPage 1347 (tetap kosong, anti-meluap).

## Bukti visual

`design/previews-clean/` (7 PNG @2x, dibaca semua): 01-dashboard, 02-page-grid,
03-workbench, 04-ledger, 05-export, 06-token-guide, 07-batch — ikon sejajar
label, tak meluap, putih terbaca di atas coral. Tanpa ubah copy/layout/token;
tanpa commit; FigCraft tetap jalan (port 9880).
