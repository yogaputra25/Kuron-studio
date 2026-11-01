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

## Rujukan

- `design/prototype/*.html` — mockup per layar; `tokens.css` = primitif.
- `design/references-v1/` — screenshot v1 순서 klik + temuan (Escape tak tutup panel, detect tanpa progres, dst).
- `design/*.figcraft` — sumber desain; `stitch-prompts.md` — generator mockup (System Style = ringkasan arah ini).
