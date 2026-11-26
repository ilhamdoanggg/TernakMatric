# note-4 — Audit Migrasi `backend/migrations/`

- **Tanggal**: 2026-11-26
- **Status**: Hasil review statis, belum dijalankan ke PostgreSQL
**Acuan**: `data/001_ternakmatric_schema.sql`, `data/002_ternakmatric_seed.sql`, `docs/06-DATABASE-ERD.md`, `docs/07-SYSTEM-ARCHITECTURE.md`, `docs/09-REPOSITORY-STRUCTURE.md`, `docs/10-DATABASE-HARDENING.md`, `docs/11-MORTALITY-FAILURE-PREVENTION.md`, `docs/decisions/001-003`, `notes/note-3.md`, dan kode di `backend/`. Indeks dokumentasi: [`docs/README.md`](../docs/README.md).

> **Koreksi 2026-11-26**: audit awal keliru menyatakan RLS dipasang pada `users` dan `organizations`. Pemeriksaan ulang menunjukkan kedua tabel hanya masuk loop trigger `set_updated_at`, bukan loop RLS. F-01 di bawah didokumentasikan sebagai koreksi, bukan temuan.

---

## 1. Kesimpulan

**Belum sepenuhnya sesuai / belum production-ready.**

Struktur dasar migrasi sudah sangat dekat dengan schema kanonik dan dokumentasi: nama tabel, tenant boundary, composite FK, partial unique index, ledger append-only untuk sebagian tabel, seed katalog, dan kolom audit sebagian besar sudah ada.

Namun ada:

- **Tidak ada bug yang membuat migrasi gagal** akibat RLS pada `users`/`organizations`; tuduhan awal sudah dikoreksi di F-01.
- **1 gap fungsional**: `role_permissions` tidak pernah di-seed, sehingga role selain `ORG_OWNER`/`ORG_ADMIN` tidak punya izin.
- **1 risiko tinggi**: seed global dijalankan setelah `FORCE ROW LEVEL SECURITY`.
- Beberapa item *hardening* yang belum mengikuti ADR/docs: append-only ledger, daftar `client_wins`, kolom audit, index analisis, GRANT/default privileges, dan sinkronisasi dokumentasi.

Jika migrasi **sudah pernah di-apply** di environment mana pun, jangan mengedit file lama karena checksum SQLx berubah. Buat migrasi korektif baru, misalnya `2026112411_fix_rls_rbac_hardening.sql`.

---

## 2. Ringkasan temuan

| ID | Prioritas | Temuan | File terkait |
| --- | --- | --- | --- |
| F-01 | ✅ Terkoreksi | Bukan bug: `users`/`organizations` hanya di loop trigger `set_updated_at`, bukan loop RLS | `2026112408` |
| F-02 | 🔴 Blocker fungsional | `role_permissions` tidak di-seed | `2026112409` |
| F-03 | 🟠 Tinggi | Seed global `organization_id IS NULL` setelah `FORCE RLS` | `2026112408` → `2026112409` |
| F-04 | 🟠 Menengah | Append-only belum mencakup `inventory_transactions` dan `production_outputs` | `2026112408`, ADR 002 |
| F-05 | 🟠 Menengah | Daftar entity `client_wins` belum memuat `population_balances` dan `inventory_balances` | `2026112407` |
| F-06 | 🟠 Menengah | Kolom audit belum seragam pada tabel sync/outbox | `2026112410` |
| F-07 | 🟡 Rendah | Index mortality/environment per batch belum ada | `2026112403`, `2026112405` |
| F-08 | 🟠 Menengah | Dua runner migrasi & GRANT tidak konsisten | `deploy/`, `infra-db` |
| F-09 | 🟡 Rendah | Dokumentasi tertinggal dari migrasi aktual | `docs/09`, `data/001`, ADR 003 |
| F-10 | 🟡 Rendah | `audit_logs` dengan `organization_id NULL` tidak bisa dibaca via RLS | `2026112406`, `2026112408` |

---

## 3. Yang sudah sesuai

- Nama tabel kanonik mengikuti `data/`: `production_units`, `batch_unit_assignments`, `environment_readings`, `production_outputs`, `calculation_runs.unit_id`, dll.
- Composite FK `(id, organization_id)` sudah konsisten di hampir seluruh relasi tenant.
- `uq_one_active_assignment_per_batch` sudah ada di `2026112403`.
- Trigger append-only sudah ada untuk `stock_movements`, `inventory_transaction_lines`, dan `audit_logs`.
- Seed katalog sudah mencakup `unit_types`, `production_systems`, species multi-kategori, `environment_metrics`, `plans`, `expense_categories`, dan `calculation_versions` termasuk `SGR` serta `HPP_LIVE_WEIGHT`.
- Kolom audit `created_by`, `updated_at`, `updated_by` sebagian besar sudah ditambahkan di `2026112410`.
- Tabel tambahan yang disebut di `docs/06` sudah ada: `individual_tags`, `vaccination_schedules`, `biosecurity_logs`.
- Kolom tambahan yang dibutuhkan dokumen lanjutan sudah ada: `environment_readings.source`, `health_records.withdrawal_days`, tipe kalkulasi `sgr` dan `hpp`.

---

## 4. Temuan detail dan perbaikan

### F-01 — `users` dan `organizations` tidak masuk RLS (koreksi, bukan temuan)

Audit awal menyatakan `2026112408` memasang policy `tenant_isolation` pada `users` dan `organizations`, sehingga migrasi akan gagal karena kedua tabel tidak punya `organization_id`. **Pernyataan itu salah.**

Pemeriksaan ulang terhadap `backend/migrations/2026112408_rls_triggers_hardening.sql`:

- Baris 40–54 adalah loop **trigger `set_updated_at`**. Di sinilah `'users','organizations'` muncul (baris 44). Trigger ini valid: kedua tabel memang memiliki kolom `updated_at` dari `2026112410`.
- Baris 56–79 adalah loop **RLS**. Daftar di sini dimulai dari `subscriptions`, `organization_members`, `member_roles`, `farms`, dan seterusnya; **`users` dan `organizations` tidak termasuk**.
- Tidak ada `CREATE POLICY ... ON users` maupun `... ON organizations` di migrasi mana pun.

Hal ini sesuai dengan `data/001_ternakmatric_schema.sql` dan `data/README.md`: kedua tabel sengaja global dan diamankan lewat GRANT serta pengecekan membership/RBAC di backend. Kode `register`/`login` mengakses `users` tanpa tenant context, dan worker membaca `organizations` sebelum context organisasi tertentu — memaksa RLS di sini akan merusak alur tersebut.

**Tindakan**: tidak ada yang dihapus dari array trigger. Jangan mengikuti saran lama untuk membuang `'users'` dan `'organizations'` dari loop `set_updated_at`.

Jika di masa depan ingin policy berbasis identitas, itu adalah keputusan baru: tambahkan policy eksplisit berbasis `current_user_id()` dan membership, bukan `organization_id = current_organization_id()`. Hal itu tidak termasuk perbaikan migrasi saat ini.

---

### F-02 — `role_permissions` tidak di-seed

**Lokasi**: `2026112409_seed_catalogs.sql`.

Migration mengisi `roles` dan `permissions`, tetapi tidak mengisi tabel junction `role_permissions`. `AuthPrincipal::require()` di `infra-auth/src/rbac.rs` membypass permission hanya untuk `ORG_OWNER` dan `ORG_ADMIN`. Role lain akan selalu mendapat `403 missing permission ...`.

Tambahkan seed:

```sql
WITH mapping(role_code, perm_codes) AS (
  VALUES
    ('ORG_OWNER', ARRAY[
      'org.manage','farm.manage','species.manage','production.write',
      'feeding.write','environment.write','health.write','finance.write','analytics.read'
    ]),
    ('ORG_ADMIN', ARRAY[
      'org.manage','farm.manage','species.manage','production.write',
      'feeding.write','environment.write','health.write','finance.write','analytics.read'
    ]),
    ('FARM_MANAGER', ARRAY[
      'farm.manage','production.write','feeding.write','environment.write',
      'health.write','analytics.read'
    ]),
    ('FARM_OPERATOR', ARRAY[
      'production.write','feeding.write','environment.write','health.write'
    ]),
    ('FINANCE_STAFF', ARRAY['finance.write','analytics.read']),
    ('VIEWER', ARRAY['analytics.read'])
)
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM mapping m
JOIN roles r
  ON r.organization_id IS NULL AND r.code = m.role_code
JOIN permissions p
  ON p.code = ANY (m.perm_codes)
ON CONFLICT DO NOTHING;
```

Karena `role_permissions` kena `FORCE RLS` dengan policy write khusus role tenant, seed ini harus dijalankan **sebelum hardening** atau dengan RLS dinonaktifkan sementara (lihat F-03).

---

### F-03 — Seed global setelah `FORCE RLS`

**Urutan saat ini**:

1. `2026112408` mengaktifkan `FORCE ROW LEVEL SECURITY` + policy write `organization_id = current_organization_id()`.
2. `2026112409` menginsert baris global `organization_id IS NULL`.

Terdampak: `roles`, `species`, `expense_categories`, dan (setelah F-02) `role_permissions`.

Jika role yang menjalankan migrasi bukan superuser / tidak punya `BYPASSRLS`, insert akan ditolak. Saat ini biasanya lolos karena `POSTGRES_USER` Docker default adalah superuser, tetapi ini tidak boleh dijadikan jaminan.

**Pilihan perbaikan**:

1. Tukar urutan file: seed sebelum hardening.
2. Atau di seed migration:

```sql
ALTER TABLE roles DISABLE ROW LEVEL SECURITY;
ALTER TABLE species DISABLE ROW LEVEL SECURITY;
ALTER TABLE expense_categories DISABLE ROW LEVEL SECURITY;
ALTER TABLE role_permissions DISABLE ROW LEVEL SECURITY;

-- ... INSERT seed global ...

ALTER TABLE roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE species ENABLE ROW LEVEL SECURITY;
ALTER TABLE expense_categories ENABLE ROW LEVEL SECURITY;
ALTER TABLE role_permissions ENABLE ROW LEVEL SECURITY;
```

`FORCE` tetap berlaku setelah `ENABLE`.

---

### F-04 — Append-only belum lengkap

`docs/06` menyebut `stock_movements`, `inventory_transaction_lines`, `audit_logs` — ini sudah ada.

Tetapi `docs/decisions/002-offline-sync.md` poin 8 juga menyebut `inventory_transactions` dan `production_outputs` sebagai append-only. `population_balances` adalah projection yang memang harus di-update, jadi tidak di-immutable.

Tambahkan:

```sql
CREATE TRIGGER trg_inventory_transactions_immutable
  BEFORE UPDATE OR DELETE ON inventory_transactions
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();

CREATE TRIGGER trg_production_outputs_immutable
  BEFORE UPDATE OR DELETE ON production_outputs
  FOR EACH ROW EXECUTE FUNCTION deny_ledger_mutation();
```

Jika transisi `draft → posted` pada `inventory_transactions` masih diperlukan, gunakan trigger yang mengizinkan hanya transisi itu, bukan `deny_ledger_mutation()` penuh. Perlu keputusan produk/domain sebelum diterapkan.

---

### F-05 — Daftar `client_wins` belum lengkap

`2026112407` melarang `client_wins` untuk sebagian entity, tetapi belum memasukkan `population_balances` dan `inventory_balances`, padahal ADR 002 poin 5 menyebut keduanya.

```sql
ALTER TABLE sync_conflicts
  DROP CONSTRAINT IF EXISTS sync_conflicts_resolution_check;

ALTER TABLE sync_conflicts
  ADD CONSTRAINT sync_conflicts_resolution_check CHECK (
    resolution IS NULL
    OR resolution <> 'client_wins'
    OR entity_type NOT IN (
      'stock_movements','population_balances',
      'inventory_transactions','inventory_transaction_lines','inventory_balances',
      'production_outputs','expenses','sales','feeding_records','mortality_records'
    )
  );
```

---

### F-06 — Kolom audit belum benar-benar seragam

`docs/06` mengklaim `2026112410` menambahkan audit columns secara seragam. Faktanya tabel dengan transisi status belum masuk daftar:

- `sync_operations`: `pending → applied/rejected/conflict`
- `sync_conflicts`: `resolution`, `resolved_at`
- `outbox_events`: `published_at`
- `sale_items`: jika pernah dimodifikasi

Tambahkan minimal:

```sql
ALTER TABLE sync_operations
  ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now(),
  ADD COLUMN IF NOT EXISTS updated_by uuid REFERENCES users(id);

ALTER TABLE sync_conflicts
  ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now(),
  ADD COLUMN IF NOT EXISTS updated_by uuid REFERENCES users(id);

ALTER TABLE outbox_events
  ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now();

CREATE TRIGGER sync_operations_set_updated_audit
  BEFORE UPDATE ON sync_operations
  FOR EACH ROW EXECUTE FUNCTION set_updated_audit();

CREATE TRIGGER sync_conflicts_set_updated_audit
  BEFORE UPDATE ON sync_conflicts
  FOR EACH ROW EXECUTE FUNCTION set_updated_audit();
```

Catatan: `set_updated_audit()` hanya mengisi `updated_by` jika `app.current_user_id` terset. Worker yang hanya memanggil `with_organization` akan mempertahankan `updated_by` lama. Pertimbangkan untuk men-set `NEW.updated_by = NULL` jika context kosong agar tidak menyesatkan.

---

### F-07 — Index untuk analisis mortalitas belum ada

`notes/note-3.md` dan `docs/11` membutuhkan overlay `environment_readings` terhadap `mortality_records` per batch. Saat ini index environment hanya `(organization_id, unit_id, measured_at)`.

```sql
CREATE INDEX IF NOT EXISTS idx_environment_readings_batch_time
  ON environment_readings(organization_id, batch_id, measured_at DESC);

CREATE INDEX IF NOT EXISTS idx_mortality_records_batch_time
  ON mortality_records(organization_id, batch_id, occurred_at DESC);
```

---

### F-08 — Runner migrasi dan GRANT belum konsisten

Ada dua jalur migrasi:

- `deploy/db/run-migrations.sh` → tabel `public.schema_migrations`
- `infra-db::run_migrations()` → `sqlx::migrate!("../../migrations")` → `_sqlx_migrations`

Saat ini `run_migrations()` belum dipanggil, jadi belum error, tetapi dokumen menyebut SQLx migrations/CI. Pilih satu runner atau samakan tracking-nya. Jika memakai SQLx, perbarui `init-app-role.sql` juga untuk me-revoke `_sqlx_migrations`.

`init-app-role.sql` memberi GRANT setelah semua migrasi. Migrasi baru setelah provisioning tidak otomatis ter-grant. Tambahkan default privileges:

```sql
ALTER DEFAULT PRIVILEGES FOR ROLE <migrator_role> IN SCHEMA public
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO <app_role>;
```

---

### F-09 — Dokumentasi tertinggal

1. `docs/09-REPOSITORY-STRUCTURE.md` ± baris 71–79 masih memakai nama migrasi lama:

   - `2026112402_init_farm_units_species.sql` → aktual `2026112402_init_farm_enclosure_species.sql`
   - `2026112403_init_batches_ledgers.sql` → aktual `2026112403_init_production_ledger.sql`
   - `2026112404_init_inventory_ledger.sql` → aktual `2026112404_init_feeding_inventory.sql`
   - `2026112405_init_finance.sql`, `2026112406_init_calculation_catalog.sql`, `2026112499_rls_and_hardening.sql` → tidak ada

   `docs/06` sudah benar; `docs/09` telah diselaraskan pada 2026-11-26 dan kini memakai nama file aktual. Nama lama di atas hanya catatan historis.

2. `data/001_ternakmatric_schema.sql` belum memuat hasil pengembangan di `backend/migrations/`:

   - `individual_tags`
   - `vaccination_schedules`
   - `biosecurity_logs`
   - `environment_readings.source`
   - `health_records.withdrawal_days`
   - tipe `sgr`, `hpp` di `calculation_runs`
   - kolom audit dari `2026112410`

   Status baseline-vs-ekstensi sudah ditegaskan di `data/README.md`, `docs/06`, dan `docs/09` pada 2026-11-26.

3. ADR 003 konsekuensi meminta seed `species_production_profiles` berversi. Migrasi saat ini tidak mengisinya. ADR 003 sudah diberi catatan implementasi: profil dibuat tenant, seed hanya katalog spesies dan metadata formula.

4. Kode kalkulasi memakai kode formula `AREA_ONLY`, `CAPACITY_UTILIZATION`, `ENCLOSURE_BIOMASS_ESTIMATE`, `FEEDING_VARIANCE`, `HPP_OUTPUT_UNIT`, dan `EGG_OUTPUT_FORECAST`, tetapi kode-kode itu tidak ada di seed `data/002` maupun `2026112409`. Sebaliknya `CAPACITY_BY_HEAD`/`CAPACITY_BY_BIOMASS` ada di seed tetapi tidak dipakai kode. Perlu keputusan mana yang kanonik sebelum `calculation_versions` dipakai sebagai lookup runtime.

---

### F-10 — `audit_logs` global tersembunyi oleh RLS

`audit_logs.organization_id` nullable, tetapi `2026112408` memberi policy ketat `organization_id = current_organization_id()`. Baris dengan `organization_id IS NULL` tidak bisa dibaca/ditulis oleh app role.

Saat ini semua pemanggilan `AuditLogger::record()` yang terlihat memakai `Some(org_id)`, jadi dampaknya masih latent. Jika audit global memang diperlukan, tambahkan policy khusus untuk peran/admin; jika tidak, ubah kolom menjadi `NOT NULL` dan pisahkan audit identitas ke tabel lain.

---

## 5. Catatan minor

- `production_units.area_m2` dan `volume_m3` mengizinkan `0` (`>= 0`), sementara `docs/10` poin 11 menyebut “dimensi/area > 0”. Perlu keputusan apakah `0` valid.
- Seed species di `2026112409` lebih sempit dari `data/002_ternakmatric_seed.sql` (misalnya tidak ada salmon, tuna, kepiting, lobster, tiram). Jika katalog broad diinginkan, samakan seed-nya.
- `species` seed belum punya kategori `other`; data baseline juga demikian, jadi ini konsisten tetapi perlu dicatat jika kategori `other` diharapkan punya contoh master.

---

## 6. Rencana perbaikan yang disarankan

### Jika database masih fresh

1. Tambahkan seed `role_permissions` (F-02).
2. Atur agar seed global tidak bergantung pada bypass RLS (F-03).
3. Tambah trigger append-only `inventory_transactions` dan `production_outputs` jika disetujui (F-04).
4. Perbarui CHECK `sync_conflicts` (F-05).
5. Tambah `updated_at`/`updated_by` ke tabel sync/outbox (F-06).
6. Tambah index mortality/environment (F-07).
7. Samakan runner migrasi + default privileges (F-08).
8. Selaraskan dokumentasi dan status baseline-vs-ekstensi (F-09).
9. Putuskan kebijakan audit global `organization_id IS NULL` (F-10).

Nama file di bawah **hanya ilustrasi untuk database baru yang belum pernah menjalankan migrasi**. Jangan mengubah nama atau isi file yang sudah di-apply; gunakan migrasi korektif baru seperti pada sub-bab berikutnya.

```text
2026112401_init_identity_organizations.sql
2026112402_init_farm_enclosure_species.sql
2026112403_init_production_ledger.sql
2026112404_init_feeding_inventory.sql
2026112405_init_environment_health.sql
2026112406_init_finance_calculation.sql
2026112407_init_offline_sync.sql
2026112408_seed_catalogs.sql
2026112409_rls_triggers_hardening.sql
2026112410_audit_actor_columns.sql
```

### Jika migrasi sudah pernah di-apply

Jangan edit file lama. Buat satu migrasi korektif:

```text
2026112411_fix_rls_rbac_hardening.sql
```

berisi: seed `role_permissions`, penanganan seed global terhadap RLS, trigger append-only tambahan, update CHECK `sync_conflicts`, kolom/trigger audit tambahan, dan index tambahan. **Jangan** menyentuh trigger `set_updated_at` pada `users`/`organizations` (lihat F-01).

---

## 7. Status definition-of-done

Sebelum `docs/10-DATABASE-HARDENING.md` ditandai hijau semua, checklist ini sebaiknya ditambahkan:

- [ ] Migrasi bisa dijalankan dari database kosong dengan role non-superuser.
- [ ] Seed global tidak bergantung pada bypass RLS.
- [ ] Role non-owner memiliki permission yang sesuai dari `role_permissions`.
- [ ] Query sebagai organisasi A tidak melihat data organisasi B.
- [ ] `UPDATE`/`DELETE` pada ledger yang ditentukan ADR 002 gagal.
- [ ] `client_wins` ditolak untuk semua entity ledger/finance/projection.
- [ ] Retry `sync_operations` dua kali tidak membuat duplikat.
- [ ] Worker bisa menulis `calculation_runs` dan `outbox_events` tanpa `updated_by` palsu.
- [ ] CI menjalankan migrasi + tenant isolation test.
