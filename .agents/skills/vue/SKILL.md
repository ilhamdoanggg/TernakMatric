---
name: Vue
description: Pola frontend TernakMatric apps/web — Vue 3.5 SSR, TypeScript strict, Vue Router 4, Vite 6, Tailwind, dan API client dari @ternakmatric/api-contracts. Gunakan saat membuat view/komponen, memanggil API, atau memperbaiki SSR/hydration.
---

# Skill: Vue (TernakMatric)

Aturan repo lengkap ada di `AGENTS.md`. Skill `ternakmatric` memuat aturan lintas area; skill ini fokus pada `apps/web`.

## Stack dan struktur

Vue 3.5 SSR + TypeScript strict, Vue Router 4, Vite 6, Tailwind CSS 3, server SSR Node tanpa framework (`server.mjs`).

```text
apps/web/
  server.mjs                 # HTTP server; Vite middleware di dev, dist/ di produksi
  src/main.ts                # createSSRApp + factory router (satu instance per request)
  src/entry-client.ts        # hydration di browser
  src/entry-server.ts        # renderToString + preload links
  src/router/index.ts        # memory history saat SSR, web history di browser
  src/lib/api.ts             # createWebApiClient()
  src/components/            # komponen UI reusable
  src/features/<feature>/    # komponen spesifik fitur
  src/views/                 # halaman level route
```

- View baru didaftarkan di `src/router/index.ts`.
- Komponen yang hanya dipakai satu fitur diletakkan di `src/features/<feature>/`, bukan `src/components/`.
- Gunakan SFC `<script setup lang="ts">` + Composition API. Jangan Options API untuk kode baru.
- Jangan menambah UI kit, state library, framework CSS, atau test runner baru tanpa keputusan eksplisit.

## API client

Semua HTTP lewat `createWebApiClient()` di `src/lib/api.ts`. Jangan `fetch` mentah dari komponen dan jangan hardcode base URL.

```ts
import { createWebApiClient } from "@/lib/api";

const api = createWebApiClient(token, organizationId);
```

- Browser: `VITE_API_URL`.
- SSR: `API_INTERNAL_URL` (hanya server, dibaca lewat `process.env` di `import.meta.env.SSR`).
- Fallback lokal: `http://localhost:8080`.
- Token dan `organization_id` dikirim lewat `getToken`/`getOrganizationId` (header `Authorization` dan `X-Organization-Id`), bukan query string.
- Jangan menyimpan token di module scope saat SSR; buat client per request.
- Tipe request/response diambil dari `@ternakmatric/api-contracts`. Jangan definisikan ulang bentuknya secara manual.
- `Decimal` dari backend adalah string. Jangan `parseFloat` untuk aritmetika bisnis dan jangan hitung ulang FCR/ADG/HPP di browser.

## SSR-safe

- Jangan akses `window`, `document`, `localStorage`, atau `navigator` di top-level module atau saat render.
- Guard dengan `import.meta.env.SSR` atau pindahkan ke `onMounted`.
- Jangan memakai `Date.now()` atau nilai acak saat render agar tidak terjadi hydration mismatch.
- Data yang harus tampil di HTML server diambil lewat `onServerPrefetch` atau di `entry-server.ts`, bukan `onMounted`.

Contoh:

```vue
<script setup lang="ts">
import { onMounted, onServerPrefetch, ref } from "vue";
import type { BatchDto } from "@ternakmatric/api-contracts";
import { createWebApiClient } from "@/lib/api";

const batches = ref<BatchDto[]>([]);
const error = ref<string | null>(null);
const api = createWebApiClient();

async function loadBatches() {
  try {
    batches.value = await api.request<BatchDto[]>("/api/v1/batches");
  } catch (e) {
    error.value = e instanceof Error ? e.message : "Gagal memuat data";
  }
}

// Hanya berjalan saat SSR; hasilnya sudah ada di HTML awal.
onServerPrefetch(loadBatches);

// Navigasi client tidak memicu onServerPrefetch.
onMounted(() => {
  if (batches.value.length === 0) void loadBatches();
});
</script>
```

Catatan:

- `TernakMatricApiClient` mengekspos `request<T>(path, init)`; belum ada method typed per endpoint.
- `onServerPrefetch` hanya berjalan saat SSR. Untuk navigasi client, tambahkan `onMounted` atau route guard.
- Bila client butuh token/organization dari request, buat client di `entry-server.ts`/`server.mjs` per request dan sediakan lewat provide/inject atau props — jangan module scope.

## Format dan tampilan

- Format uang, angka, dan tanggal memakai locale `id-ID`, misalnya `new Intl.NumberFormat("id-ID")`. Jangan hardcode pemisah ribuan atau simbol mata uang.
- Tailwind utility-first. Pertahankan tema gelap `slate`/`emerald` yang sudah ada.
- Data mock hanya untuk scaffold. Jangan menambah mock baru bila endpoint sudah tersedia. `DashboardView` dan `SpeciesOverviewTable` masih memakai data contoh dan harus diganti saat fitur di-wire.
- Aksesibilitas: elemen semantik, label form, `alt` pada gambar, tabel dan navigasi bisa dioperasikan keyboard.

## Variabel environment

- Hanya `VITE_*` yang boleh terekspos ke browser.
- Secret server tidak boleh memakai prefix `VITE_*`.
- `VITE_API_URL` untuk browser, `API_INTERNAL_URL` untuk SSR.

## Verifikasi

```sh
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
npm --prefix apps/web run dev
```

- `build` sudah mencakup `vue-tsc --noEmit` dan build client + server.
- Untuk perubahan SSR, jalankan `dev` dan pastikan halaman ter-render dari server (view source atau `curl`), bukan hanya setelah hydration.
- `preview` memakai sintaks env POSIX. Di PowerShell jalankan setelah build:

```powershell
$env:NODE_ENV = "production"
node apps/web/server.mjs
```

Belum ada test runner web; verifikasi minimal adalah `typecheck` dan `build`.
