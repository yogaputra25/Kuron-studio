# DESIGN.md — Kuron Studio

Aturan visual + layout. Pasangan: `ARCHITECTURE.md` (sistem), `AGENTS.md` (kerja).
Sumber nilai: `kuron-studio/src/app.css` — jangan karang warna di luar token `--ks-*`.

## Arah

Dense desktop workbench untuk translator, bukan landing page. Quiet, tight rows, grup jelas.

## Token

- Single source: `--ks-*` di `app.css` (`@theme inline` → utility Tailwind `bg-surface`, `text-ink`, dst). Dark default + light. Pasangan teks/latar WCAG AA.
- Peran warna: **coral = aksi utama saja**; **cyan = info teknis** (koordinat, ID, jumlah bubble); success/warn/danger/info = status. Tanpa gradient, shadow, emoji sebagai ikon.
- Font (bundel offline via fontsource, bukan CDN): Inter = UI, Space Grotesk = brand/heading, JetBrains Mono = angka + nama file. Angka di badge/koordinat pakai `.tnum` (tabular-nums).
- Kontrol ikut tabel Skala ukuran di bawah (tinggi, radius, ikon, touch target); separator garis 1px.
- Skala ukuran (satu-satunya yang boleh dipakai; jangan hardcode angka lain):
  | Elemen | sm | md (default) | lg | Konstanta |
  |---|---|---|---|---|
  | Button | h-7 (28px) | h-8 (32px) | h-10 (40px) | `size` di `ui/Button.svelte` (`SIZE`) |
  | Field / input / Lov trigger | — | h-9 (36px) | — | `h-9` di `ui/Field.svelte` + `ui/Lov.svelte` (pengecualian compact h-8 hanya di dalam dropdown) |
  | Segmented group | h-7, ikut Button sm | h-8, ikut Button md | — | samakan dengan `size` Button di sebelahnya |
  | Icon | 16px (`sm`) | 20px (`md`) | 24px (`lg`) | `size` di `ui/Icon.svelte` (`ICON_SIZE`), `stroke=currentColor` selalu |
  | Radius | `rounded-md` (6px) kontrol | `rounded-card` token kartu/panel/modal | `rounded-full` pil/badge | token `--radius-card` di `app.css` + default Tailwind |
  | Touch target | min 44px (mobile/tablet; drawer, FAB, gagang sheet) | — | — | `min-h-[44px]` / `size-11`+ |

## Primitif

Ikuti `design/prototype/tokens.css` (identik dengan `app.css`): `.btn` (pri/def/gho/dgr/sm/on/disabled), `.field`, `.badge` (ok/run/warn/err/cy/mute), `.dot`. Button menamai aksi + objek ("Terjemahkan 12 halaman"). Error menyebut sebab + jalan keluar. Empty state menawarkan aksi, bukan ilustrasi.

## Layout

| Layar | Pola |
|---|---|
| Dashboard | Kartu project 300px: cover, judul, counter mono, progress bar tipis, stat, badge mode |
| Daftar halaman | Pipeline bar 5 tahap (Import→Deteksi→Terjemah→Review→Ekspor) + grid kartu (thumb, chip jumlah bubble cyan, nama file mono, badge status; selected = outline coral) |
| Editor | 3 kolom 196px / 1fr / 268px: list halaman, canvas gelap + outline bubble coral + badge nomor, inspector kartu bubble (Asli/Baca/Hasil + fit bar) |
| Ledger | Tabel dense: # \| Hlm \| Asli \| Terjemahan \| Muat \| Dicek; filter segmented + search; footer ringkasan |
| Tablet 834 | Rail ikon 64pt + kartu 3 kolom + bottom sheet 40% (grab handle, aksen cyan) |
| Mobile 390 | Grid 2 kolom + tab bar 56pt |

String UI Indonesia default; tambah key baru = 3 bahasa di `i18n.ts` satu commit. Fokus keyboard selalu terlihat; hormati `prefers-reduced-motion`.

## Peran → token

Satu peran = satu token. Tint selalu `color-mix(in srgb, var(--token) N%, transparent)`, tidak pernah hex baru → parity dark/light otomatis.

| Peran | Token | Catatan |
|---|---|---|
| Kanvas aplikasi | `--ks-bg` | latar window terdalam |
| Panel di atas kanvas | `--ks-surface` | latar halaman kerja |
| Panel bertumpuk | `--ks-surface-2` | header, toolbar, rail |
| Kartu / modal terangkat | `--ks-raised` | + elevasi level 2–3, bukan shadow bebas |
| Hover baris list | `--ks-hover` | sementara; hilang saat pointer pergi |
| Baris terpilih persisten | `--ks-selected` | opt-in `[data-current="true"]`; dilarang mengarang warna selected per layar |
| Kursor keyboard (sementara) | tint + inset ring, tanpa token sendiri | `color-mix(in srgb, var(--ks-text) 12%, transparent)` + `box-shadow: inset 0 0 0 1px var(--ks-ring)`; dilarang sekadar `--ks-hover` (nyaris tak terlihat di light) |
| Garis rambut | `--ks-line` | semua pembatas struktural 1px |
| Garis kuat / divider drag | `--ks-line-strong` | kontur jendela luar, border dialog |
| Cincin fokus | `--ks-ring` | `:focus-visible` + halo seleksi; bukan dekorasi permanen |
| Aksi utama | `--ks-accent` | satu-satunya aksi afirmatif per alur; bukan untuk hover, dekorasi, atau aksi sekunder |
| Info teknis | `--ks-cyan` | hanya koordinat, ID, jumlah bubble — selalu mono + `.tnum`; teks sekunder biasa pakai `--ks-text-3` |
| Status | `--ks-success/warn/danger/info` (+ `-soft` untuk latar badge) | badge tanpa border, duduk di atas `-soft` |

## Elevasi (3 level, tidak ada level keempat)

| Level | Komposisi | Pakai untuk |
|---|---|---|
| 1 | border hairline `--ks-line`, tanpa shadow | default — hampir semua hal |
| 2 | border + `shadow-xs` | kontrol outline, kartu tertanam |
| 3 | border `--ks-line-strong` + `shadow-floating` | popover / dialog yang lepas dari kanvas |

Butuh lebih dari level 3 biasanya berarti yang dicari adalah `--ks-ring` fokus, bukan shadow. Level keempat dilarang.

## Baris list

| State | Tampilan |
|---|---|
| idle | transparan |
| hover | `--ks-hover` |
| kursor keyboard | tint foreground + inset `--ks-ring` (lihat tabel peran) |
| aktif persisten | `--ks-selected` + `[data-current="true"]` |

Tetap tiga state yang berbeda dan terlihat (hover / kursor / terpilih tidak boleh saling menyamar). Dilarang warna selected custom per layar.

## Perilaku UX

Aturan perilaku, bukan selera. Berlaku untuk Deteksi bubble, Terjemah batch, Ekspor, Import PDF, dan semua alur async lain.

| Durasi | Yang ditampilkan |
|---|---|
| 0–100 ms | tidak ada |
| 100 ms–1 s | disable kontrol saja |
| 1–3 s | disable + spinner |
| 3 s+ / multi-tahap | label progres per tahap |

- `disabled` di-bind segera; spinner baru muncul setelah ~200 ms (operasi lokal tidak berkedip).
- Ruang label yang berganti dipesan (`width`, bukan `min-width`) supaya layout tidak melompat.
- Copy tidak overclaim: verb hasil ("terhapus", "terverifikasi", "ditemukan") hanya bila ada data hasil nyata; selama proses pakai bahasa netral ("Menerjemahkan 4/12 halaman…"); tanpa pengisi ("silakan", "cukup", "tinggal").
- Error menyebut sebab + jalan keluar — berlaku untuk toast juga, bukan hanya empty state.
- Jalur mundur sunyi (Batal / Tutup / Buang): gaya `ghost`, tanpa warna aksen, tanpa chip shortcut, tanpa animasi pengalih perhatian; `Esc` selalu menutup lapisan teratas.
- Shortcut: label ikut binding nyata (`⌘`/`⇧` di macOS, `Ctrl+`/`Shift+` di lainnya); chip shortcut hanya lewat satu komponen `ui/Shortcut.svelte`, bukan `<kbd>` ad-hoc.
- Motion: `prefers-reduced-motion` dihormati per elemen (`motion-reduce:`), bukan satu switch global; spinner tetap "hidup" saat reduced motion; scroll yang dipicu skrip memakai perilaku aman-reduced-motion.
- Kosong vs error vs loading: data kosong → aksi untuk mendapatkannya, bukan ilustrasi; toast hanya untuk hasil sesaat; error yang harus dibaca/di-retry/copy → inline, persisten.

## Rujukan rubric

Review layar sebelum merge memakai `design/ux-review-rubric.md` (satu halaman per layar/screenshot, jawab ya/tidak tanpa debat). Rubric mengikat untuk perubahan UI; perubahan non-UI dikecualikan.

## Rujukan

- `design/prototype/*.html` — mockup per layar; `tokens.css` = primitif.
- `design/references-v1/` — screenshot v1 순서 klik + temuan (Escape tak tutup panel, detect tanpa progres, dst).
- `design/*.figcraft` — sumber desain; `stitch-prompts.md` — generator mockup (System Style = ringkasan arah ini).
