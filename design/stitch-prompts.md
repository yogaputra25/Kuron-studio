# Paket Prompt Google Stitch — Kuron Studio Redesign

Cara pakai: buka [Stitch](https://stitch.withgoogle.com) → paste **System Style** sekali
(atau taruh di custom instructions), lalu paste **satu prompt per layar**.
Bahasa prompt Inggris (Stitch paling nurut), label UI tetap Indonesia.
Referensi visual: `design/references-v1/` (screenshot app asli).

---

## SYSTEM STYLE (paste sekali)

Dark dense desktop workbench for manga translators, not a landing page.
Colors: bg #0D0D10, surface #141418, surface-2 #1A1A20, raised #22222A,
line #26262E, text #ECECF1, muted text #A5A6B0 / #71727D,
accent coral #FF6B5A (primary actions only), cyan #35E7F5 (technical info),
success #5FD99A, warn #FFC15E, danger #FF5C5C.
Fonts: Inter for UI, Space Grotesk for brand/headings, JetBrains Mono for numbers.
Rules: 28px control height, 6px radius, 1px line separators (no shadows,
no gradients, no emojis). Tight rows, clear groups, tabular numbers.
Every button names action + object ("Translate 12 Pages"). Errors name
cause + way out. Empty states offer the action, never illustrations.
UI strings in Indonesian.

---

## 1. DASHBOARD (desktop 1280)

"Desktop dashboard for a manga translation workbench, dark theme (#0D0D10).
Top bar: app name 'Kuron Studio', view title 'Library', primary coral button
'+ Project baru'. Stats line: '3 project · 41 halaman tersisa · 2 gagal'.
Three project cards in a row, each 300px: cover thumbnail area, title
('ONIMASA Ch.12'), mono counter ('007/182 · 35%'), thin progress bar,
stats line ('118 todo · 2 gagal · dibuka 2 mnt lalu'), small mode badge
('Mode A' cyan / 'Mode B'). Card 2 full success green bar ('52/52'),
card 3 empty warn state. Footer hint: 'Klik kartu = buka project'.
Dense, quiet, no decoration."

## 2. PAGE LIST + PIPELINE (desktop 1280, layar utama)

"Desktop page-grid screen, dark theme. Slim header: brand, project dropdown
'Chapter 12', mono counter '007/182', theme + language buttons, ⌘K button.
Below header, a 5-stage pipeline bar: Import 182 (done green) → Deteksi 164
(done) → Terjemah 064 (active blue) → Review 012 → Ekspor, each stage a dot
+ label + mono count, connected by thin lines; right side badge '64/182'
and coral button 'Terjemahkan sisa 118'. Page cards grid 4 columns: manga
page thumbnail with cyan bubble-count chip overlay (e.g. '5'), filename
mono ('p007.jpg'), status badge (Done green / Translating blue / Idle gray /
No bubbles amber / Failed red). Selected card has coral outline."

## 3. EDITOR WORKSPACE (desktop 1280, layar tersibuk)

"Three-column desktop workspace, dark theme. Left sidebar 196px: scrollable
page list, each row 26px thumbnail + mono filename + status dot + bubble
count ('5 bubble', 'belum deteksi', 'gagal deteksi'); active row highlighted
with coral left border. Center: dark canvas with one manga page, two oval
speech-bubble selections with coral outline and number badges 1, 2; bottom
zoom bar (minus, 100%, plus, RTL toggle, '5 bubble'). Right inspector 268px:
bubble cards (#2 active with cyan badge, #3 warn 'Perlu review', #4 danger
'Kelebihan'); each card shows rows Asli (Japanese source) / Baca (romaji) /
Hasil (Indonesian translation) plus a fit bar (green 'Muat 9/12' or amber
'Sesak 18/11'). Footer: coral button 'Terjemahkan #3' + 'Simpan'."

## 4. BUBBLE LEDGER (desktop 1280)

"Desktop translation-management table, dark theme. Header: brand, project
dropdown, counter, buttons 'Terjemahkan tersaring' and 'Ekspor'. Filter bar:
segmented tabs (Semua 87 active, Belum terjemah 23, Belum dicek 64,
Muat gagal 4), search field 'Cari teks sumber…', keyboard hint.
Dense table, columns: # | Hlm | Asli (Japanese) | Terjemahan (Indonesian) |
Muat (badge: Muat green / Sesak amber / Kelebihan red / — gray) | Dicek
(badge: Sudah green / Belum amber / — gray). Highlighted active row with
coral left border. Footer bar: summary '87 bubble · 23 belum terjemah ·
4 gagal muat', buttons 'Tandai 64 dicek' and coral 'Terjemahkan 23'."

## 5. TABLET (834 wide)

"Tablet layout 834pt wide, dark theme, touch targets min 44pt. Top bar:
hamburger, brand, chapter label, Detect + Review buttons (44pt tall).
Body: left icon rail 64pt wide (Library, Grid active coral, Chapters,
Settings icons with tiny labels); main area: note text, 3 page cards in a
row, bottom sheet editor with drag handle, title, and two buttons
(44pt tall), snapped at 40% height with cyan top border accent."

## 6. MOBILE (390 wide)

"Mobile layout 390pt wide, dark theme, touch targets min 44pt. Compact
header (brand + counter), search field 'Cari project…', 2-column card grid
(page thumbnail, filename, status dot + label: Menerjemahkan cyan /
Selesai green / Gagal red). Bottom tab bar 56pt: Library (coral active),
Editor, Review, Settings."
