# TernakMatric Mobile (`apps/mobile`)

Aplikasi mobile *offline-first* berbasis **Flutter + SQLite (`sqflite`)** untuk pencatatan lapangan (pakan, mortalitas, pertumbuhan) dan sinkronisasi *command* ke backend Axum.

## Struktur

- `lib/core/config/app_config.dart` — Konfigurasi environment dari `--dart-define` / `--dart-define-from-file` (`APP_ENV`, `API_URL`).
- `lib/core/api/` — Model sinkronisasi dan `TernakMatricApiClient` yang mengikuti `packages/api-contracts/openapi/ternakmatric.v1.json`.
- `lib/storage/sqlite_schema.dart` — Skema SQLite lokal dan `SqfliteQueueStore`.
- `lib/offline/conflict_policy.dart` — Kebijakan server-authoritative sesuai ADR 002 (tanpa `client_wins` untuk ledger/stok/keuangan).
- `lib/sync/sync_engine.dart` — Siklus `push`/`pull` idempoten dengan `client_operation_id` (UUID v4) dan cursor outbox.
- `lib/screens/field_log_screen.dart` — Form pencatatan pakan & mortalitas offline.

## Perintah

```powershell
cd apps/mobile
flutter pub get
flutter analyze
flutter test
flutter run --dart-define-from-file=.env.example
```

> Jangan menaruh secret pada `--dart-define` atau file `.env*` milik client.
