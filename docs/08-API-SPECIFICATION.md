# 08 — API Specification: TernakMatric

**Dokumen**: `TM-DOC-08` · **Status**: Baseline v1.0 · **Tanggal**: 2026-11-24

Seluruh endpoint REST berada di bawah prefix `/api/v1` (kecuali `/health` dan `/api-docs/openapi.json`). Spesifikasi OpenAPI mesin-terbaca dihasilkan oleh Utoipa di `packages/api-contracts/openapi/`.

---

## 1. Konvensi Umum

- **Autentikasi**: `Authorization: Bearer <jwt_token>`
- **Konteks Tenant**: `X-Organization-Id: <uuid>` (diverifikasi ulang terhadap `organization_members` di backend; tidak pernah dipercaya mentah tanpa verifikasi membership)
- **Idempotency**: Header `Idempotency-Key: <uuid>` wajib untuk operasi mutasi ledger/keuangan dan `sync_operations.client_operation_id`
- **Format Error Standar**:
  ```json
  {
    "code": "CAPACITY_EXCEEDED",
    "message": "Total occupancy exceeds enclosure maximum capacity",
    "details": { "enclosure_id": "...", "requested": 1200, "available": 800 }
  }
  ```

---

## 2. Daftar Endpoint per Modul

### Health & OpenAPI
| Method | Path | Deskripsi |
| --- | --- | --- |
| `GET` | `/health` | Health check API & koneksi pool DB |
| `GET` | `/api-docs/openapi.json` | Dokumen OpenAPI 3.1 hasil generate Utoipa |

### `mod-tenant` — Auth, Organisasi, Staf & Farm
| Method | Path | Deskripsi |
| --- | --- | --- |
| `POST` | `/api/v1/auth/register` | Registrasi akun user baru |
| `POST` | `/api/v1/auth/login` | Login & dapatkan access/refresh JWT |
| `GET` | `/api/v1/organizations` | Daftar organisasi yang diikuti user |
| `POST` | `/api/v1/organizations` | Buat organisasi tenant baru |
| `GET` | `/api/v1/organizations/{id}/members` | Daftar anggota & role organisasi |
| `POST` | `/api/v1/organizations/{id}/members` | Undang / tambah staf & tetapkan role |
| `GET` | `/api/v1/farms` | Daftar farm pada organisasi aktif |
| `POST` | `/api/v1/farms` | Buat farm baru |
| `PATCH` | `/api/v1/farms/{id}` | Perbarui data farm (mengisi `updated_at`/`updated_by` & `audit_logs`) |
| `GET` | `/api/v1/audit-logs` | Riwayat perubahan entitas (`actor_user_id`, `before_data`, `after_data`, `occurred_at`) |

### `mod-enclosure` — Kandang, Kolam & Kapasitas
| Method | Path | Deskripsi |
| --- | --- | --- |
| `GET` | `/api/v1/unit-types` | Katalog tipe unit produksi (`water`, `land`, `cage`, `pasture`, `hive`, `other`) |
| `GET` | `/api/v1/production-systems` | Katalog sistem produksi (`TRADITIONAL`, `INTENSIVE`, `BIOFLOC`, `RAS`, dll.) |
| `GET` | `/api/v1/enclosures` | Daftar enclosure (`production_units`) per farm |
| `POST` | `/api/v1/enclosures` | Buat enclosure (kolam/kandang/cage/hive) beserta dimensi/kapasitas |
| `GET` | `/api/v1/enclosures/{id}` | Detail enclosure + okupansi batch aktif saat ini |
| `PATCH` | `/api/v1/enclosures/{id}` | Perbarui data/status enclosure (mengisi `updated_at`/`updated_by` & `audit_logs`) |
| `POST` | `/api/v1/enclosures/{id}/calculate-capacity` | Hitung kapasitas & persentase okupansi dengan profil spesies |

### `mod-species` — Katalog Spesies & Profil Produksi Berversi
| Method | Path | Deskripsi |
| --- | --- | --- |
| `GET` | `/api/v1/species` | Daftar spesies global + custom tenant (filter `category`) |
| `POST` | `/api/v1/species` | Tambah spesies custom organisasi |
| `GET` | `/api/v1/species/{id}/profiles` | Daftar profil produksi berversi (`species_production_profiles`) |
| `POST` | `/api/v1/species/{id}/profiles` | Buat versi baru profil produksi + tahapan pertumbuhan |

### `mod-production` — Batch/Cohort, Populasi, Tag Individu & Output
| Method | Path | Deskripsi |
| --- | --- | --- |
| `GET` | `/api/v1/batches` | Daftar batch/cohort produksi |
| `GET` | `/api/v1/batches/summary` | Daftar ringkasan batch (enclosure aktif, populasi, ABW, ADG, SGR, estimasi biomassa) |
| `POST` | `/api/v1/batches` | Buat batch baru + assignment enclosure awal (validasi kapasitas) |
| `GET` | `/api/v1/batches/{id}` | Detail batch beserta populasi, pertumbuhan & estimasi biomassa terkini |
| `PATCH` | `/api/v1/batches/{id}` | Perbarui target/status/catatan batch (mengisi `updated_at`/`updated_by` & `audit_logs`) |
| `POST` | `/api/v1/batches/{id}/transfer` | Pindahkan batch ke enclosure lain dalam satu transaksi *(belum terpasang)* |
| `POST` | `/api/v1/batches/{id}/stock-movements` | Catat pergerakan populasi (`additional_stock`, `birth`, `purchase`, `cull`, `adjustment`) |
| `GET` | `/api/v1/batches/{id}/tags` | Daftar tag individu (`individual_tags`) pada batch |
| `POST` | `/api/v1/batches/{id}/tags` | Daftarkan tag individu baru (eartag/RFID/ring) |
| `GET` | `/api/v1/batches/{id}/growth-samples` | Daftar riwayat sampling pertumbuhan batch |
| `POST` | `/api/v1/batches/{id}/growth-samples` | Input sampling pertumbuhan (hitung ABW, ADG, SGR) |
| `GET` | `/api/v1/batches/{id}/outputs` | Daftar riwayat output produksi batch |
| `POST` | `/api/v1/batches/{id}/outputs` | Catat output (`meat`, `egg`, `milk`, `fry`, `cull`, `offspring`, `other`) |

### `mod-feeding` — Pakan, Gudang & Kalkulator FCR
| Method | Path | Deskripsi |
| --- | --- | --- |
| `GET` | `/api/v1/warehouses` | Daftar gudang organisasi |
| `POST` | `/api/v1/warehouses` | Buat gudang baru |
| `GET` | `/api/v1/feed-products` | Daftar SKU pakan/ransum & konversi satuan |
| `POST` | `/api/v1/feed-products` | Tambah SKU pakan/ransum |
| `GET` | `/api/v1/inventory/balances` | Saldo stok gudang per produk & `lot_code` (`inventory_balances`) |
| `POST` | `/api/v1/inventory/transactions` | Catat transaksi gudang (`receipt`, `issue`, `transfer`, `adjustment`, `return`) |
| `POST` | `/api/v1/feeding` | Catat pemberian pakan ke batch (atomik memotong gudang + hitung FCR) |
| `GET` | `/api/v1/batches/{id}/fcr` | Hitung Biological FCR & Economic FCR batch |

### `mod-environment` — Kualitas Air & Lingkungan Kandang
| Method | Path | Deskripsi |
| --- | --- | --- |
| `GET` | `/api/v1/environment/metrics` | Daftar katalog metrik lingkungan (`water`, `air`, `soil`, `housing`, `feed`) |
| `GET` | `/api/v1/enclosures/{id}/environment` | Histori pembacaan lingkungan pada enclosure |
| `POST` | `/api/v1/environment/readings` | Catat pembacaan manual |
| `POST` | `/api/v1/environment/iot-ingest` | Endpoint webhook ingesti sensor IoT (V2-ready) |

### `mod-health` — Vaksin, Pengobatan, Mortalitas & Biosecurity
| Method | Path | Deskripsi |
| --- | --- | --- |
| `GET` | `/api/v1/batches/{id}/vaccinations` | Daftar jadwal vaksinasi batch |
| `POST` | `/api/v1/batches/{id}/vaccinations` | Buat jadwal vaksinasi baru |
| `PATCH` | `/api/v1/vaccinations/{id}` | Perbarui status vaksinasi (mengisi `updated_at`/`updated_by` & `audit_logs`) |
| `GET` | `/api/v1/batches/{id}/health-records` | Daftar riwayat observasi penyakit & pengobatan batch |
| `POST` | `/api/v1/batches/{id}/health-records` | Catat observasi penyakit atau pengobatan |
| `GET` | `/api/v1/batches/{id}/mortalities` | Daftar riwayat mortalitas batch |
| `POST` | `/api/v1/batches/{id}/mortalities` | Catat mortalitas (atomik mengurangi `population_balances`) |
| `POST` | `/api/v1/farms/{id}/biosecurity-logs` | Catat log biosecurity (sanitasi, karantina, tamu/kendaraan) |

### `mod-finance` — Biaya, Penjualan, HPP & P&L
| Method | Path | Deskripsi |
| --- | --- | --- |
| `GET` | `/api/v1/finance/expense-categories` | Katalog kategori biaya sistem & custom organisasi |
| `GET` | `/api/v1/finance/expenses` | Daftar biaya operasional/langsung (filter `batch_id`, `farm_id`, `category_id`) |
| `POST` | `/api/v1/finance/expenses` | Catat biaya operasional/langsung ke batch atau farm |
| `GET` | `/api/v1/finance/sales` | Daftar transaksi penjualan beserta item line-nya |
| `POST` | `/api/v1/finance/sales` | Catat transaksi penjualan terkait `production_outputs` |
| `GET` | `/api/v1/batches/{id}/financial-summary` | Hitung total biaya, pendapatan, HPP per kg hidup/output, dan Laba/Rugi |

### Offline Sync (`infra-auth`)
| Method | Path | Deskripsi |
| --- | --- | --- |
| `POST` | `/api/v1/sync/devices` | Registrasi perangkat mobile/web |
| `POST` | `/api/v1/sync/push` | Kirim batch *sync operations* dari mobile SQLite (idempoten) |
| `GET` | `/api/v1/sync/pull` | Tarik delta perubahan sejak `cursor` terakhir dari `outbox_events` |
| `GET` | `/api/v1/sync/conflicts` | Daftar konflik yang perlu resolusi manual |
