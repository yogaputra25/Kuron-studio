# Laporan Ikon per Layar — FigCraft icon system (2026-11-07)

Bahasa ikon: frame 16×16 TANPA layout + primitif rect/ellipse, stroke/fill
`--ks-text-2` (#A5A6B0) 1.5px, putih (#FFFFFF) di atas CTA coral, corner membulat,
maks 1 ikon per tombol/baris. File: `design/kuron-studio.figcraft`
(nodes 1466–1610, next_id 1611). Reload akhir: `repairs: []`.

## 01 Dashboard (frame 10)
| Tombol (id) | Ikon frame (id) | Jenis |
|---|---|---|
| BtnDrop 110 Pilih File... | 1482 IcoTrayup_110 | tray-upload |
| BtnOpen1 56 Buka Editor (coral) | 1485 IcoChev_56 | chevron, putih |
| BtnLed1 58 Ledger (F3) | 1489 IcoList_58 | list |
| BtnOpen2 80 Buka Editor | 1492 IcoChev_80 | chevron |
| BtnCBZ 82 Ekspor CBZ | 1497 IcoDl_82 | download |
| BtnDetect 104 Mulai Deteksi (coral) | 1500 IcoScan_104 | scan, putih |

## 02 Page-Grid (frame 118)
| Tombol (id) | Ikon frame (id) | Jenis |
|---|---|---|
| BtnSearch 136 ⌘K Cari | 1503 IcoSearch_136 | search |
| BtnReview 138 Tinjau Ledger (F3) | 1507 IcoList_138 | list |
| BtnTransAll 140 Terjemahkan Sisa 118 (coral) | 1510 IcoBubble_140 | bubble, putih |
| BtnRedetect 402 Deteksi Ulang | 1516 IcoScan_402 | scan |
| BtnGoEdit 404 Buka di Editor (coral) | 1519 IcoChev_404 | chevron, putih |
| SearchBox 562 (field) | 1466 IcoSearch | search |
| BtnDetectPage 1347 | — | SENGAJA tanpa ikon (tombol selebar kartu, +22px meluap) |

## 03 Workbench (frame 406)
| Tombol (id) | Ikon frame (id) | Jenis |
|---|---|---|
| BtnRedet 431 Deteksi Ulang | 1522 IcoScan_431 | scan |
| BtnSavePg 433 Simpan Terjemahan (coral) | 1526 IcoSave_433 | floppy, putih |
| BtnGlossary 476 | 1529 IcoBook_476 | book |
| BtnProvider 478 | 1532 IcoGear_478 | ring+dot |
| BtnDiag 480 | 1535 IcoBang_480 | seru |
| BtnRetranslate 538 Terjemahkan Ulang | 1538 IcoScan_538 | scan |
| BtnApplyNext 540 Terapkan & Lanjut (coral) | 1541 IcoChev_540 | chevron, putih |

## 04 Ledger (frame 542)
| Tombol (id) | Ikon frame (id) | Jenis |
|---|---|---|
| BtnExportCSV 557 | 1546 IcoDl_557 | download |
| BtnApproveAll 559 Loloskan Semua (coral) | 1551 IcoCheck2_559 | cek-ganda, putih |

## 05 Export (frame 752)
| Baris/tombol (id) | Ikon/grup (id) | Jenis |
|---|---|---|
| Opt_PDF 911 | grup O1LG 1477 + 1473 IcoDoc_911 | doc |
| Opt_CBZ 917 | grup O2LG 1579 + ikon box | box |
| Opt_Folder 923 | grup O3LG 1583 + ikon folder | folder |
| BtnPreset 950 Simpan Preset | 1555 IcoSave_950 | floppy |
| BtnDoExport 952 Ekspor (coral) | 1560 IcoDl_952 | download, putih |
| MClose/XIcon 907-908 | — | sudah ada, tak tersentuh |

## 06 Token (frame 954)
| Baris/tombol (id) | Ikon/grup (id) | Jenis |
|---|---|---|
| MI1 1066 Buka di Editor | grup MI1L 1472 + chevron | chevron |
| MI2 1069 Deteksi Ulang | grup MI_1069 1587 + scan | scan |
| MI3 1073 Hapus dari Proyek | grup MI_1073 1591 + trash | trash |
| TipBox 1063 | 1462 TipIcon (parent 1461→1063, circle sw 1.0→1.5) | circle-i, tajam |
| BtnUndo 1083 Urungkan | 1564 IcoUndo_1083 | undo |
| DBtnDel 1091 Hapus (coral) | 1567 IcoTrash_1091 | trash, putih |
| ACH 1100 Terjemahkan Halaman Ini | grup ACH_1100 1595 + bubble | bubble |
| ACH 1105 Tinjau Ledger | grup ACH_1105 1600 + list | list |
| ACH 1110 Ekspor PDF | grup ACH_1110 1606 + download | download |
| ACH 1115 Pengaturan Provider | grup ACH_1115 1610 + gear | gear |

## 07 Batch (frame 1119)
| Tombol (id) | Ikon frame (id) | Jenis |
|---|---|---|
| BtnSimulate 1341 Simulasi Kuota | 1571 IcoChart_1341 | chart |
| BtnStartBatch 1343 Mulai Batch (coral) | 1575 IcoGo_1343 | panah-go, putih |

## Keputusan desain (3)
1. Glyph lama (+, ←, ▾, ⌫, ⌘, [x], ✓, radio/dot) di-grandfather — ikon drawn
   hanya di affordance kosong, nol perubahan teks.
2. Tanpa ikon di header statis dan stat-card batch (tinggi fix 56 meluap) —
   ikon harus memperkuat affordance, bukan dekorasi.
3. Ring elips-stroke diganti rect radius-penuh (7 node: 1463 + 6 ikon) karena
   renderer menggambar elips-stroke sebagai kipas (garis radius).
