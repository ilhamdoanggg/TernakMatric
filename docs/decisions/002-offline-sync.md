# ADR 002 — Offline-First Command Synchronization

- Status: Diterima
- Tanggal: 2026-11-24
- Terkait: `note-2.md` (idempotency), `note-3.md` (full offline sync, ledger, conflict resolution), [`note-4-migration-audit.md`](../../notes/note-4-migration-audit.md) (status implementasi dan utang)

## Konteks

Mobile dipakai operator di lapangan/kandang/kolam dan harus tetap bisa mencatat tanpa internet. Dua operator bisa offline bersamaan dan mengubah data yang sama (misalnya mengambil pakan dari gudang yang sama). note-3 melarang `client_wins`/last-write-wins untuk stok ternak, inventory, output produksi (daging, telur, susu, fry), dan keuangan, serta menyebut schema awal (`devices`, `sync_operations`, `sync_conflicts`) masih memerlukan change-feed/cursor, entity versioning, transactional outbox, dan resolusi konflik lengkap.

## Keputusan

1. Mobile mengirim **command** (bukan state akhir) ke endpoint sync.
2. Setiap command memiliki:

   | Field | Fungsi |
   | --- | --- |
   | `client_operation_id` | ID unik dari device, dipakai untuk idempotency |
   | `device_id` | Perangkat pengirim |
   | `entity_id` | UUID entitas yang disentuh |
   | `operation_type` | create / update / delete |
   | `base_version` | versi server terakhir yang diketahui device |
   | `payload` | data transaksi |
   | `device_occurred_at` | waktu kejadian di device |

3. Idempotency dijamin oleh constraint unik `(organization_id, device_id, client_operation_id)`. Retry tidak boleh membuat entitas atau ledger ganda.
4. Server memproses command **dalam satu transaksi** dengan row locking pada entitas yang terlibat, lalu:
   - memvalidasi ulang stok ternak, saldo inventory, kapasitas unit produksi, dan aturan bisnis pada saat apply;
   - menolak transaksi yang menghasilkan populasi, saldo, atau kapasitas negatif/terlampaui;
   - mencatat hasil ke `sync_operations` (`pending`, `applied`, `rejected`, `conflict`).
5. Konflik disimpan di `sync_conflicts` dan dikembalikan ke device; tidak ada silent overwrite. Strategi `client_wins` **dilarang** untuk `stock_movements`/`population_balances`, `inventory_transactions`/`inventory_balances`, `production_outputs`, `expenses`, dan `sales`.
6. Entitas bisnis yang sensitif memakai **entity versioning** (`version` / `base_version`). Update dengan `base_version` stale ditolak atau masuk jalur resolusi.
7. `infra-db::OutboxWriter` menyimpan event perubahan dalam transaksi yang sama dengan commit bisnis. Worker menerbitkan change-feed berkursor agar device lain bisa pull perubahan.
8. `stock_movements`, `inventory_transactions`, dan `production_outputs` bersifat **append-only** sebagai ledger. Koreksi dibuat sebagai transaksi reversal/adjustment, bukan mengubah baris lama. `population_balances` dan `inventory_balances` adalah projection yang diperbarui dalam transaksi yang sama dengan ledger, bukan ledger immutable.
9. Worker bertugas memproses queue sync, rekonsiliasi ledger, dan menghasilkan snapshot/forecast — bukan hanya alert evaluator.

## Konsekuensi

- Saat ini endpoint sync berada di `infra-auth/src/sync.rs` dan dirangkai di `bins/api-server`; outbox berada di `infra-db::OutboxWriter`. Crate `mod-sync`/`infra-outbox` belum ada.
- Migration `2026112407_init_offline_sync.sql` berisi `devices`, `sync_operations`, `sync_conflicts`, indeks `(organization_id, status, received_at)`, serta constraint idempotency.
- `apps/mobile/src/storage` = SQLite lokal; `apps/mobile/src/sync` = queue + retry + cursor; `apps/mobile/src/offline` = UI untuk operasi pending dan konflik.
- CI wajib menguji: retry dua kali tetap satu transaksi, dua command konkuren tidak membuat saldo negatif atau melebihi kapasitas unit, command stale menghasilkan conflict, dan konflik ledger/finance tidak pernah diselesaikan dengan `client_wins`.
- Endpoint sync harus mengikuti kontrak `/api/v1` dan idempotency key terdokumentasi di `08-API-SPECIFICATION.md` / OpenAPI.

### Catatan implementasi (2026-11-26)

Keputusan di atas tetap berlaku dengan penyesuaian berikut:

- Field yang dipakai migrasi adalah `device_occurred_at`, bukan `occurred_at` (`2026112407_init_offline_sync.sql`).
- Daftar entity yang dilarang `client_wins` di constraint `sync_conflicts` belum memuat `population_balances` dan `inventory_balances`; lihat F-05 di `note-4`.
- Worker saat ini menjalankan snapshot biomassa dan reminder vaksin; pemrosesan queue sync/rekonsiliasi ledger belum diimplementasikan.
- Status lengkap dan usulan migrasi korektif ada di [`../../notes/note-4-migration-audit.md`](../../notes/note-4-migration-audit.md).
