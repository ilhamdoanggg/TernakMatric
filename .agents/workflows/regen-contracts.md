---
description: Regenerasi artefak kontrak API OpenAPI dan tipe TypeScript dari anotasi Utoipa Rust
---

Gunakan workflow ini setiap kali ada perubahan handler Axum, route, atau DTO (`ToSchema` / `utoipa::path`). **Jangan mengedit manual** `packages/api-contracts/openapi/ternakmatric.v1.json` atau `packages/api-contracts/generated/typescript/schema.d.ts`.

// turbo
1. Pastikan test kontrak OpenAPI di `api-server` hijau terlebih dahulu:
```powershell
cargo test --manifest-path backend/Cargo.toml -p api-server test_openapi_spec_builds_with_all_module_paths
```

// turbo
2. Export spesifikasi OpenAPI terbaru dari binary `api-server`:
```powershell
npm --prefix packages/api-contracts run gen:openapi
```

// turbo
3. Generate ulang tipe TypeScript dari file OpenAPI:
```powershell
npm --prefix packages/api-contracts run gen:ts
```

// turbo
4. Jalankan typecheck pada `packages/api-contracts` dan `apps/web` untuk memastikan tidak ada breaking change yang terlewat di frontend:
```powershell
npm --prefix packages/api-contracts run typecheck
npm --prefix apps/web run typecheck
```

5. Periksa `git diff -- packages/api-contracts` untuk memverifikasi perubahan path, schema, dan tipe yang dihasilkan sudah sesuai.
