# Indeks Dokumentasi TernakMatric

Dokumentasi produk dan arsitektur TernakMatric. Bahasa dokumen: **Indonesia**. Aturan kerja coding agent ada di [`../AGENTS.md`](../AGENTS.md).

## Dokumen fondasi (01–12)

| Dokumen | Topik |
| --- | --- |
| [`01-PROBLEM-DISCOVERY.md`](01-PROBLEM-DISCOVERY.md) | Masalah peternak dan peluang produk |
| [`02-PRODUCT-REQUIREMENTS.md`](02-PRODUCT-REQUIREMENTS.md) | Requirement produk dan ruang lingkup |
| [`03-USER-STORIES.md`](03-USER-STORIES.md) | User story dan alur pengguna |
| [`04-BUSINESS-RULES.md`](04-BUSINESS-RULES.md) | Aturan bisnis produksi dan keuangan |
| [`05-DOMAIN-MODEL.md`](05-DOMAIN-MODEL.md) | Model domain dan batas konteks |
| [`06-DATABASE-ERD.md`](06-DATABASE-ERD.md) | ERD, daftar migrasi, dan prinsip schema |
| [`07-SYSTEM-ARCHITECTURE.md`](07-SYSTEM-ARCHITECTURE.md) | Arsitektur modular monolith dan dependency rule |
| [`08-API-SPECIFICATION.md`](08-API-SPECIFICATION.md) | Rancangan endpoint API |
| [`09-REPOSITORY-STRUCTURE.md`](09-REPOSITORY-STRUCTURE.md) | Struktur repository aktual dan status implementasi |
| [`10-DATABASE-HARDENING.md`](10-DATABASE-HARDENING.md) | Checklist hardening database dan RLS |
| [`11-MORTALITY-FAILURE-PREVENTION.md`](11-MORTALITY-FAILURE-PREVENTION.md) | Analisis mortalitas dan pencegahan kegagalan produksi |
| [`12-SUPPLY-AND-MARKET-LINKAGE.md`](12-SUPPLY-AND-MARKET-LINKAGE.md) | Keterkaitan pasokan dan pasar |

## Keputusan arsitektur (ADR)

| ADR | Keputusan |
| --- | --- |
| [`decisions/001-tenant-boundary.md`](decisions/001-tenant-boundary.md) | `organization_id` sebagai tenant boundary dan GUC RLS |
| [`decisions/002-offline-sync.md`](decisions/002-offline-sync.md) | Strategi sinkronisasi offline, idempotensi, dan resolusi konflik |
| [`decisions/003-calculation-engine.md`](decisions/003-calculation-engine.md) | Calculation engine murni dan berversi |

## Dokumen terkait di luar `docs/`

| Lokasi | Isi |
| --- | --- |
| [`../notes/`](../notes/) | Catatan perjalanan desain `note-1..3` dan audit migrasi `note-4` |
| [`../data/`](../data/) | Baseline schema SQL, seed referensi, data dictionary, dan ERD |
| [`../backend/migrations/`](../backend/migrations/) | Schema yang benar-benar dipasang |
| [`../packages/api-contracts/`](../packages/api-contracts/) | OpenAPI hasil export → `generated/typescript/schema.d.ts` → client `src/index.ts` |

## Urutan membaca

1. [`01-PROBLEM-DISCOVERY.md`](01-PROBLEM-DISCOVERY.md) → [`02-PRODUCT-REQUIREMENTS.md`](02-PRODUCT-REQUIREMENTS.md) untuk masalah dan requirement.
2. [`05-DOMAIN-MODEL.md`](05-DOMAIN-MODEL.md) → [`06-DATABASE-ERD.md`](06-DATABASE-ERD.md) untuk model dan schema.
3. [`decisions/001`](decisions/001-tenant-boundary.md), [`decisions/002`](decisions/002-offline-sync.md), [`decisions/003`](decisions/003-calculation-engine.md) untuk keputusan yang mengikat.
4. [`07-SYSTEM-ARCHITECTURE.md`](07-SYSTEM-ARCHITECTURE.md) → [`09-REPOSITORY-STRUCTURE.md`](09-REPOSITORY-STRUCTURE.md) untuk implementasi.
5. [`10-DATABASE-HARDENING.md`](10-DATABASE-HARDENING.md) dan [`../notes/note-4-migration-audit.md`](../notes/note-4-migration-audit.md) untuk status production-readiness.

## Sumber kebenaran

| Area | Sumber kanonik |
| --- | --- |
| Produk dan aturan bisnis | Dokumen `01..12` dan `decisions/` |
| Nama tabel/kolom/satuan | `data/001_ternakmatric_schema.sql`, `data/DATA_DICTIONARY.md`, `docs/06-DATABASE-ERD.md` |
| Schema terpasang | `backend/migrations/` |
| Arsitektur dan batas crate | `docs/07-SYSTEM-ARCHITECTURE.md`, `docs/09-REPOSITORY-STRUCTURE.md` |
| Kontrak API | Anotasi Utoipa → `packages/api-contracts/openapi/ternakmatric.v1.json` → TypeScript |

Jika dokumen bertentangan dengan kode, perbarui dokumen setelah memastikan mana yang menjadi sumber kebenaran sesuai tabel di atas. Generated artifact tidak diedit manual.
