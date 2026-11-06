# Rubric review layar — Kuron Studio

Satu halaman per layar/screenshot, diisi sebelum layar dianggap selesai.
Setiap baris dijawab ya/tidak tanpa debat. Acuan visual: `DESIGN.md`;
sumber token: `kuron-studio/src/app.css` (`--ks-*`).

## A. Hierarki & layout

- [ ] Aksi utama terlihat dalam < 1 detik pandang pertama
- [ ] Maksimal 1 aksi afirmatif dominan per layar
- [ ] Alignment mengikuti grid yang ada (tidak ada 1–2px nyasar)
- [ ] Tidak ada scroll horizontal tak disengaja pada lebar minimum

## B. Tipografi & spasi

- [ ] Hanya skala dari DESIGN.md (tidak ada `text-[13px]`)
- [ ] Angka di badge/koordinat memakai `.tnum`
- [ ] Jarak antar-grup > jarak dalam-grup

## C. Warna & kontras

- [ ] Semua warna dari token `--ks-*` (tanpa hex di luar `app.css`)
- [ ] Pasangan teks/latar AA di **dark dan light**
- [ ] Warna hanya untuk status/aksi — bukan dekorasi
- [ ] Cyan hanya untuk info teknis

## D. State

- [ ] Loading sesuai matriks durasi (DESIGN.md § Perilaku UX)
- [ ] Kosong menawarkan aksi, bukan ilustrasi
- [ ] Error menyebut sebab + jalan keluar; inline bila harus dibaca
- [ ] Disabled terlihat berbeda, bukan hanya tak bisa diklik
- [ ] Terpilih / hover / kursor keyboard: tiga state berbeda dan terlihat

## E. Keyboard & aksesibilitas

- [ ] Semua aksi tercapai tanpa mouse
- [ ] Fokus selalu terlihat (bukan `outline: none` tanpa pengganti)
- [ ] `Esc` menutup lapisan teratas
- [ ] Fokus kembali ke pemicu setelah dialog/panel ditutup
- [ ] Target sentuh ≥ 44px di mobile/tablet

## F. Copy

- [ ] Tidak overclaim (verb hasil hanya bila ada data nyata)
- [ ] Menyebut objek + jumlah ("Terjemahkan 12 halaman", bukan "Lanjutkan")
- [ ] 3 bahasa lengkap (`id/en/zh`)

## G. Kebenaran token FigCraft

Mockup `.figcraft` wajib 1:1 dengan token kanonis di
`tools/figcraft/patterns.md` (cermin `app.css`, blok `:root` dark default).
Ringkas 12 butir checklist UX-feel SKILL FigCraft v2.0:

- [ ] Satu protagonis per layar; ritme gap 4/8/12/20; hierarki 3 sapuan terbaca
- [ ] Satu coral di atas lipatan; cyan hanya data teknis mono; badge tanpa border di atas soft-tint
- [ ] Asli vs Hasil langsung beda bobot; daftar "rapat tapi bernapas"; detail 1px rapi
- [ ] Microcopy nyata (aksi+objek, error+solusi, empty+aksi); angka tabular, progres 3–6px
- [ ] Fokus keyboard jelas + shortcut terlihat; tidak ada elemen di area traffic-light macOS
- [ ] `prefers-reduced-motion` dihormati; semua warna dari `--ks-*` (cek `lint_design` CLEAN)
