**Bisa diganti ke Flutter** — tidak ada ketergantungan teknis di backend Rust/PostgreSQL yang mengharuskan memakai React Native.

Saat ini `apps/mobile` baru berupa **scaffold awal** dan memang belum menjadi fokus pengembangan (lihat [AGENTS.md](file:///c:/Users/90188096/project/TernakMatric/AGENTS.md) dan [09-REPOSITORY-STRUCTURE.md](file:///c:/Users/90188096/project/TernakMatric/docs/09-REPOSITORY-STRUCTURE.md)).

### Kondisi `apps/mobile` saat ini
Isi kode yang baru ada hanyalah 4 file TypeScript/TSX dan konfigurasi Expo di [package.json](file:///c:/Users/90188096/project/TernakMatric/apps/mobile/package.json), [app.json](file:///c:/Users/90188096/project/TernakMatric/apps/mobile/app.json), dan [eas.json](file:///c:/Users/90188096/project/TernakMatric/apps/mobile/eas.json):
- [sqliteSchema.ts](file:///c:/Users/90188096/project/TernakMatric/apps/mobile/src/storage/sqliteSchema.ts) — skema SQLite lokal (`local_enclosures`, `local_batches`, `pending_sync_operations`, `local_sync_conflicts`, `sync_state`).
- [syncEngine.ts](file:///c:/Users/90188096/project/TernakMatric/apps/mobile/src/sync/syncEngine.ts) — kerangka `push`/`pull` ke `/api/v1/sync/push` dan `/api/v1/sync/pull`.
- [conflictPolicy.ts](file:///c:/Users/90188096/project/TernakMatric/apps/mobile/src/offline/conflictPolicy.ts) — aturan penolakan `client_wins` untuk entitas ledger/stok/keuangan.
- [FieldLogScreen.tsx](file:///c:/Users/90188096/project/TernakMatric/apps/mobile/src/screens/FieldLogScreen.tsx) — contoh layar form statis.

Karena belum ada fitur produksi yang dalam, biaya berpindah sekarang sangat kecil.

---

### Perbandingan: Tetap React Native (Expo) vs Pindah ke Flutter

#### Jika tetap memakai React Native + Expo
- **Shared contract langsung jalan**: Bisa langsung mengimpor `TernakMatricApiClient` dan tipe OpenAPI dari `@ternakmatric/api-contracts` ([src/index.ts](file:///c:/Users/90188096/project/TernakMatric/packages/api-contracts/src/index.ts)).
- **Satu stack bahasa**: TypeScript seragam dengan `apps/web` dan `packages/api-contracts`.
- **Scaffold sudah siap**: `expo-sqlite` dan `eas.json` sudah terpasang.

#### Jika mengganti ke Flutter
Hal-hal yang tetap harus dijaga agar selaras dengan arsitektur TernakMatric:
1. **Kontrak API (`packages/api-contracts`)**:
   - Flutter tidak bisa memakai package TypeScript `@ternakmatric/api-contracts` secara langsung.
   - Namun sumber kebenarannya tetap file OpenAPI JSON (`packages/api-contracts/openapi/ternakmatric.v1.json`). Anda dapat menambahkan script codegen Dart dari file OpenAPI tersebut (misalnya ke `packages/api-contracts/generated/dart/`).
2. **Tipe `Decimal` dari backend**:
   - Backend mengirim angka uang/berat/laju sebagai **string** JSON (`rust_decimal::Decimal`). Di Dart, jangan ubah ke `double` untuk perhitungan/nilai presisi — gunakan tipe desimal eksak atau simpan sebagai string/decimal.
3. **Kontrak Offline-First ([002-offline-sync.md](file:///c:/Users/90188096/project/TernakMatric/docs/decisions/002-offline-sync.md))**:
   - Harus tetap memakai SQLite lokal (di Flutter misalnya `drift` atau `sqflite`),
   - Mengirim command ke `/api/v1/sync/devices`, `/api/v1/sync/push`, `/api/v1/sync/pull` dengan `client_operation_id` (UUID v4), `base_version`, dan `device_occurred_at`,
   - Mematuhi daftar entitas server-authoritative (dilarang `client_wins` untuk `stock_movements`, `population_balances`, `inventory_transactions`, `inventory_transaction_lines`, `inventory_balances`, `production_outputs`, `expenses`, `sales`, `feeding_records`, `mortality_records`).
4. **Update dokumentasi & keputusan**:
   - Jika diputuskan pindah, [07-SYSTEM-ARCHITECTURE.md](file:///c:/Users/90188096/project/TernakMatric/docs/07-SYSTEM-ARCHITECTURE.md), [09-REPOSITORY-STRUCTURE.md](file:///c:/Users/90188096/project/TernakMatric/docs/09-REPOSITORY-STRUCTURE.md), dan [002-offline-sync.md](file:///c:/Users/90188096/project/TernakMatric/docs/decisions/002-offline-sync.md) perlu diselaraskan karena saat ini masih mencatat `React Native + Expo + SQLite`.

### Kesimpulan praktis
- **Tetap React Native + Expo** jika Anda/tim ingin berbagi kode TypeScript & `@ternakmatric/api-contracts` langsung dengan `apps/web`.
- **Ganti ke Flutter** jika Anda/tim jauh lebih cepat dan nyaman membangun UI Android/iOS dengan Flutter — secara arsitektur backend, perpindahan ini aman dilakukan sekarang selama tetap mengacu pada OpenAPI `ternakmatric.v1.json` dan kontrak sinkronisasi di ADR 002.

Kalau Anda mau pindah ke Flutter, beri tahu saja apakah Flutter SDK sudah terpasang di mesin ini agar bisa saya bantu rancang/siapkan strukturnya.