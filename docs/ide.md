# Ide — Kuron Studio

## Problem

Provider translate manga/manhwa saat ini kerja manual:
- Crop bubble satu-satu di Photoshop / manual tool
- Copy-paste ke Google Translate / ChatGPT tanpa konteks reading order
- Tidak ada deteksi bubble otomatis, tidak ada batch, tidak ada glossary konsisten
- Hasil tidak konsisten antar chapter, SFX sering ke-translate salah, honorific hilang

Kuron App sudah punya pipeline translate canggih (ONNX + Mosaic + BYOK LLM) tapi **untuk end-reader**, bukan untuk **provider** yang butuh workbench batch + edit + export.

## Solution

**Kuron Studio** — app desktop Tauri v2 (Rust) yang jadi workbench provider:

```
Import batch (50 halaman) → ONNX detect → Edit bubble di canvas → Translate BYOK → Post-edit → Export
```

Reuse 100% logic Kuron (prompt, mosaic, provider) agar hasil konsisten, tapi dengan UX untuk **produksi**, bukan konsumsi.

## Why Tauri v2 + Rust?

| Alasan | Detail |
|--------|--------|
| **Reuse Rust Kuron** | `kuron_native/rust` sudah punya `image_ops`, `imageproc`, decrypt — port langsung |
| **Performa batch** | Rust + rayon untuk 50 halaman, ONNX native, bukan Dart isolate |
| **Akses FS native** | Import folder, drag-drop, export CBZ/PNG — butuh desktop |
| **Bundle kecil** | Tauri ~10MB vs Electron ~150MB |
| **BYOK aman** | OS keychain (Keychain/Credential Manager/Secret Service) |
| **Cross-platform** | 1 codebase → Windows/macOS/Linux |

> Tauri v2 output: **Desktop stabil** (Win .exe/.msi, macOS .app/.dmg, Linux .deb/.AppImage). Mobile (Android/iOS) masih beta — deferred.

## Core Value Proposition

1. **Deteksi otomatis** — ONNX bubble detection on-device, tanpa upload
2. **Mosaic cerdas** — crop 20% pad + 2x scale + label merah, cap 2MB/1MB hemat token
3. **BYOK fleksibel** — 9 provider (Zen, OpenCode Go, Gemini, OpenAI, OpenRouter, Cohere, Custom) — provider pakai key sendiri
4. **Batch & Glossary** — 50 halaman sekaligus, glossary konsisten antar chapter
5. **Export produksi** — JSON, PNG overlay, CBZ burned-in untuk distribusi

## Target User

- **Primary:** Tim translate manga/manhwa (scanlation, provider komik)
- **Secondary:** Freelance translator yang butuh tool batch
- **Bukan:** End-reader (itu Kuron App)

## Differentiator vs Tool Lain

| Tool | Kuron Studio Advantage |
|------|------------------------|
| Photoshop + ChatGPT manual | Otomatis detect + mosaic + reading order |
| Google Translate | Konteks bubble + style (7 gaya) + glossary |
| Electron app | Bundle 10x lebih kecil, Rust performa |
| Cloud-only tool | ONNX on-device, privacy, offline detect |

## Ide Fitur (Backlog)

- [ ] TM (Translation Memory) — search terjemahan sebelumnya
- [ ] QA checks — untranslated, overflow, SFX leak
- [ ] PSD export — layer per bubble untuk editor
- [ ] Plugin prompt — custom prompt per provider/genre
- [ ] Kolaborasi — project share via git/zip
- [ ] Auto-typeset — render terjemahan langsung ke image (burned-in)

## Non-Goal (V1)

- Bukan reader / scraper source
- Bukan hosting model (BYOK saja)
- Bukan mobile app (desktop dulu)
- Bukan cloud sync (local-first)
