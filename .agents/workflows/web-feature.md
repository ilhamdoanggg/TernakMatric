---
description: Workflow membuat atau mengubah fitur/halaman Vue 3 SSR di apps/web
---

Gunakan workflow ini saat menambah route, view, komponen fitur, atau integrasi API di `apps/web/`. Rujukan: `AGENTS.md`, `.agents/skills/vue/SKILL.md`.

1. **Pastikan kontrak API sudah mutakhir**:
   - Jika endpoint/DTO backend baru ditambahkan atau diubah, jalankan `/regen-contracts` terlebih dahulu.
   - Ambil semua tipe request/response dari `@ternakmatric/api-contracts` (`import type ...`); jangan mendefinisikan ulang tipe API secara manual.

2. **Ikuti struktur & pola Vue 3.5 SSR**:
   - Daftarkan halaman level route di `apps/web/src/router/index.ts` dan letakkan file di `apps/web/src/views/`.
   - Komponen khusus satu fitur diletakkan di `apps/web/src/features/<feature>/`; komponen reusable lintas fitur di `apps/web/src/components/`.
   - Wajib `<script setup lang="ts">` + Composition API, TypeScript strict tanpa `any`.
   - Pertahankan utility-first Tailwind tema gelap `slate`/`emerald`; jangan menambah UI kit, state library, atau test runner baru tanpa keputusan eksplisit.

3. **Panggil API & format data dengan benar**:
   - Semua request HTTP wajib melalui `createWebApiClient()` di `apps/web/src/lib/api.ts` (jangan `fetch` mentah atau hardcode URL).
   - Jangan menyimpan token di module scope saat SSR; buat client per request. Hanya variabel `VITE_*` yang boleh terekspos ke browser (`API_INTERNAL_URL` khusus server SSR).
   - `Decimal` dari backend berupa string: tampilkan dengan `Intl.NumberFormat("id-ID")`, jangan `parseFloat` untuk aritmetika bisnis atau menghitung ulang FCR/ADG/HPP di browser.
   - Jangan menambah data mock baru jika endpoint API sudah tersedia.

4. **Jaga keamanan SSR & aksesibilitas**:
   - Jangan akses `window`, `document`, `localStorage`, atau `navigator` di top-level module atau saat render; guard dengan `import.meta.env.SSR` atau pindahkan ke `onMounted`.
   - Hindari `Date.now()` atau nilai acak saat render agar tidak terjadi hydration mismatch.
   - Data yang harus ada di HTML server diambil lewat `onServerPrefetch` (atau `entry-server.ts`) dan dilengkapi fallback `onMounted` untuk navigasi client.
   - Gunakan elemen semantik, label form, `alt`, dan navigasi keyboard.

// turbo
5. **Verifikasi typecheck & build SSR**:
```powershell
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
```

6. **Verifikasi render SSR langsung** (saat menambah/mengubah route atau logic SSR):
   Jalankan `npm --prefix apps/web run dev`, lalu buka atau periksa respons HTML server (mis. `Invoke-WebRequest http://localhost:3000/<route>`) untuk memastikan konten ter-render di HTML awal, bukan hanya setelah hydration.
   Untuk menjalankan hasil build produksi di PowerShell:
   ```powershell
   $env:NODE_ENV = "production"
   node apps/web/server.mjs
   ```
