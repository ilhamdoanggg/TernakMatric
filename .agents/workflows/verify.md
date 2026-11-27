---
description: Verifikasi lengkap workspace TernakMatric (Rust fmt/clippy/test, kontrak OpenAPI/TypeScript, dan web Vue SSR)
---

Jalankan seluruh pemeriksaan kualitas sebelum menyatakan perubahan selesai sesuai `AGENTS.md`.

// turbo
1. Format dan cek formatting backend Rust:
```powershell
cargo fmt --manifest-path backend/Cargo.toml --all -- --check
```
Jika ada selisih format, jalankan `cargo fmt --manifest-path backend/Cargo.toml --all`.

// turbo
2. Jalankan Clippy pada seluruh workspace dengan `-D warnings`:
```powershell
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
```

// turbo
3. Jalankan seluruh unit & contract test Rust:
```powershell
cargo test --manifest-path backend/Cargo.toml --workspace
```

// turbo
4. Cek tipe paket `@ternakmatric/api-contracts`:
```powershell
npm --prefix packages/api-contracts run typecheck
```
Jika handler atau DTO API berubah, jalankan workflow `/regen-contracts` terlebih dahulu sebelum langkah ini.

// turbo
5. Verifikasi typecheck dan build produksi (client + SSR server) untuk `apps/web`:
```powershell
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
```

6. Jika ada perubahan schema/migrasi SQL, pastikan sudah diuji dari database kosong memakai `/db-local`. Jika ada perubahan SSR route/view, jalankan `npm --prefix apps/web run dev` dan pastikan HTML awal ter-render di server.
