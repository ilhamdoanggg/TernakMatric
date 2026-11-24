# @ternakmatric/api-contracts

Single source of truth untuk kontrak REST API TernakMatric:

1. Backend Rust (`utoipa`) mengekspor spesifikasi OpenAPI ke `openapi/ternakmatric.v1.json`.
2. `npm run gen:ts` menghasilkan tipe TypeScript ke `generated/typescript/schema.d.ts`.
3. `apps/web` dan `apps/mobile` mengonsumsi client dan tipe dari `src/index.ts`.
