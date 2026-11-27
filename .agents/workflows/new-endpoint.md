---
description: Checklist end-to-end menambah atau mengubah endpoint HTTP Axum di backend TernakMatric
---

Ikuti urutan ini saat menambah atau mengubah endpoint di `backend/` (rujukan: `AGENTS.md`, `.agents/skills/axum-endpoint/SKILL.md`, `.agents/skills/rust/SKILL.md`).

1. **Tentukan lokasi & baca konteks domain**:
   - Endpoint domain diletakkan di `backend/crates/mod-<domain>/src/lib.rs`.
   - Endpoint lintas modul/sistem (health, sync) diletakkan di `backend/bins/api-server/src/main.rs`.
   - Jangan pernah menaruh handler HTTP atau SQL di `shared-kernel` atau `mod-calculation`.
   - Pastikan nama tabel/kolom merujuk pada `backend/migrations/` dan `docs/data/DATA_DICTIONARY.md`.

2. **Definisikan DTO Request & Response**:
   - Request: `#[derive(Debug, Serialize, Deserialize, ToSchema)]`.
   - Response DTO: `ToSchema` (tambahkan `sqlx::FromRow` hanya jika dibaca langsung dari query).
   - Gunakan tipe newtype ID dari `shared-kernel` di domain, `Decimal` untuk angka bisnis (bukan `f64`/`f32`), dan wajib sertakan `idempotency_key: Uuid` pada command mutasi stok/ledger/saldo.

3. **Implementasikan handler aman-tenant**:
   - Handler generik over `S: AppBackendState`, menerima `AuthPrincipal`, dan langsung panggil `auth.require(Permission::...)` sebelum bekerja.
   - Jangan percaya header `X-Organization-Id` secara langsung.
   - Validasi input **sebelum** membuka transaksi (`AppError::Validation`).
   - Buka transaksi tenant: `let mut tx = auth.context().begin_tx(state.pool()).await.map_err(ApiError)?;`.
   - Tetap bind `auth.organization_id.0` secara eksplisit di setiap query meskipun RLS aktif.
   - Gunakan `FOR UPDATE` pada baris induk saat memeriksa kapasitas, saldo, atau uniqueness.
   - Mapping error SQLx hanya via `infra_db::map_sqlx_err`. Tidak boleh `unwrap`, `expect`, `panic!`, `println!`, atau `dbg!` di handler/service.
   - Tulis `AuditLogger::record` dan `OutboxWriter::append` di transaksi yang sama, lalu `tx.commit().await`.

4. **Daftarkan route & OpenAPI**:
   - Pakai sintaks path Axum 0.8 `{id}` (bukan `:id`) di `router<S>()` dan `#[utoipa::path(...)]`.
   - Daftarkan handler dan schema di `#[derive(OpenApi)]` modul.
   - Jika modul atau path top-level baru, merge router dan `nest(...)` `ApiDoc` di `backend/bins/api-server/src/main.rs`, lalu tambah assertion di `test_openapi_spec_builds_with_all_module_paths`.

// turbo
5. **Verifikasi backend & regenerasi kontrak API**:
```powershell
cargo fmt --manifest-path backend/Cargo.toml --all
cargo clippy --manifest-path backend/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml --workspace
npm --prefix packages/api-contracts run gen:openapi
npm --prefix packages/api-contracts run gen:ts
npm --prefix packages/api-contracts run typecheck
npm --prefix apps/web run typecheck
```
