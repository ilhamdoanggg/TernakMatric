# ADR 001 — Tenant Boundary dan Penamaan Konteks Organisasi

- Status: Diterima
- Tanggal: 2026-11-24
- Terkait: `note-2.md` (organization_members, RBAC per tenant), `note-3.md` (RLS, composite FK)

## Konteks

TernakMatric adalah SaaS multi-tenant untuk produksi semua ternak yang dikelola — ikan, udang/kerang, unggas, babi, kambing/domba, sapi/kerbau, kelinci, reptil, serangga, dan spesies lain dalam kategori `fish | crustacean | mollusk | poultry | mammal | reptile | insect | other`. Satu user bisa menjadi anggota banyak organisasi dengan role berbeda. note-3 menetapkan `organization_id` sebagai tenant boundary dan memperingatkan dua lapisan proteksi: composite foreign key serta Row-Level Security. Struktur yang dikirimkan masih memakai istilah `tenant_id` dan setting `app.tenant_id`, sehingga berbeda dengan contoh di note-3 (`app.current_organization_id`). ADR ini mengusulkan penamaan `app.current_organization_id` agar konsisten dengan boundary `organization_id`; keputusan final perlu dikonfirmasi sebelum migration ditulis.

## Keputusan

1. Nama kanonik untuk boundary adalah **`organization_id`** di schema dan `OrganizationId` di domain (`shared-kernel`).
2. Setting sesi Postgres yang dipakai RLS adalah **`app.current_organization_id`**.
3. Nilai setting diambil dari **membership terverifikasi** di backend, bukan dari body/header request client.
4. Setting dilakukan dengan **`SET LOCAL`** di dalam transaksi yang sama dengan query bisnis:

   ```sql
   SELECT set_config('app.current_organization_id', $1, true);
   ```

   Di SQLx:

   ```rust
   sqlx::query("SELECT set_config('app.current_organization_id', $1, true)")
       .bind(org_id.as_str())
       .execute(&mut tx)
       .await?;
   ```

5. Semua tabel data bisnis memiliki `organization_id`, langsung atau melalui parent; relasi antar-tabel memakai **composite foreign key** `(organization_id, id)` agar tidak bisa cross-tenant.
6. RLS diaktifkan pada tabel bisnis dan tabel yang berisi membership/role. Role database aplikasi **tidak boleh** `superuser`, table owner, atau memiliki `BYPASSRLS`.
7. Istilah `tenant` boleh dipakai dalam narasi produk, tetapi bukan nama kolom/setting. Modul `mod-tenant` mengelola `organizations` + `organization_members`, bukan tabel bernama `tenants`.

## Konsekuensi

- Migration `2026112401_init_identity_organizations.sql` menggantikan `init_tenants_users.sql`.
- `infra-db::TenantContext` (lebih tepat: `OrganizationContext`) menjadi satu-satunya tempat set setting RLS.
- Semua repository wajib menerima context, bukan `organization_id` bebas dari request.
- Dibutuhkan test otomatis: query dengan organisasi A tidak boleh melihat data organisasi B, termasuk lewat relasi turunan.
- Jika ada perubahan nama, lakukan migrasi rename, bukan menambah setting baru yang paralel.
