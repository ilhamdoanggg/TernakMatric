Pilihan **TernakMatric** sangat cocok. Saya menyarankan kita membangun TernakMatric sebagai sebuah **Animal Production Management Platform** — bukan sekadar kalkulator kapasitas kolam atau kandang, tetapi sistem untuk membantu peternak mengambil keputusan berdasarkan data produksi ternak. Kategori spesies yang didukung: `fish | crustacean | mollusk | poultry | mammal | reptile | insect | other` (ikan adalah salah satu kategori).

Karena kamu memiliki latar belakang Full-Stack Development, kita bisa merancangnya dengan pendekatan software engineering yang terstruktur: mulai dari problem discovery, requirement, domain modeling, database schema, API, hingga implementasi.

**Saran saya: jangan langsung membuat database schema.** Kita perlu memahami masalah dan proses bisnisnya terlebih dahulu supaya database yang dibuat tidak perlu sering dirombak.

# **1. Roadmap pengembangan TernakMatric**

1. **Problem Discovery**

   Identifikasi masalah peternak, proses produksi ternak, dan kebutuhan sistem.

   Mulai dari sini
2. **Business Requirements**

   Tentukan aktor, use case, business rules, dan fitur MVP.
3. **Domain Modeling**

   Definisikan entitas seperti Farm, ProductionUnit, Batch, AnimalStock, Feeding, dan Output.
4. **Database Design**

   ERD, relational schema, constraints, indexes, dan migration.
5. **System Architecture**

   Backend, frontend, API contract, authentication, dan authorization.
6. **UI/UX Design**

   Dashboard, manajemen unit produksi (kolam, kandang, cage, hive), pencatatan produksi ternak, dan laporan.
7. **Development & Testing**

   Implementasi MVP, pengujian perhitungan, dan validasi dengan pengguna.

# **2. Problem discovery — kumpulkan masalah terlebih dahulu**

Kita perlu mengetahui masalah yang benar-benar dialami peternak.

Berikut daftar awal problem yang bisa kita investigasi.

| Area             | Pertanyaan / Problem                                                     |
| ---------------- | ------------------------------------------------------------------------ |
| Unit produksi    | Bagaimana menghitung kapasitas kolam, kandang, cage, atau hive?           |
| Stok awal        | Berapa jumlah awal ternak dan bagaimana mencatat kelahiran/pembelian?     |
| Kematian         | Bagaimana mencatat kematian ikan, unggas, babi, kambing, sapi, atau serangga? |
| Pakan            | Berapa kebutuhan pakan/ransum harian per spesies dan sistem produksi?     |
| Pertumbuhan      | Bagaimana mengetahui pertumbuhan semua spesies (berat, panjang, telur, susu)? |
| Lingkungan       | Bagaimana monitoring air, udara, tanah, dan kandang (pH, suhu, kelembapan, oksigen)? |
| Output produksi  | Kapan waktu panen daging, fry, telur, atau susu yang optimal?             |
| Keuangan         | Berapa biaya produksi dan keuntungan bersih?                              |
| Prediksi         | Berapa estimasi output berdasarkan kondisi aktual?                        |

Ada satu hal penting: TernakMatric sebaiknya membedakan **data aktual, estimasi, dan target**. Misalnya, berat ternak hasil sampling (ikan, ayam, atau sapi) tidak boleh dianggap sebagai berat seluruh populasi yang sudah pasti.

# **3. Tentukan scope MVP (Minimum Viable Product)**

Jangan membangun semua fitur sekaligus. Saya menyarankan TernakMatric memiliki 6 modul inti untuk versi pertama.

[Macroact - Make Robots Active & Smart](https://images.openai.com/static-rsc-4/2N2xGI44EzfPNX3i07KHspXIm0Bqmm9URm-8EEAtdEL1YBFtOvFXqwdZcehdGYf2OwUKmSNNfCV5HuWHjR5HJ9ZivRVTU3xK3kzwHJ2wWdvR4ANcEFdTva5waZjmGw5A4b0QnEMMBt_2o9WSOECM3jpr94J-BHkW-9D5w4AcdZw?purpose=inline)

**1. Farm & Production Unit Management**

- Data lokasi peternakan.
- Data unit produksi: kolam, kandang, cage, hive, atau unit lainnya.
- Jenis unit (`unit_types`) dan sistem produksi (`production_systems`).
- Dimensi (`dimensions`), luas (`area_m2`), dan volume (`volume_m3`) yang relevan.
- Kapasitas maksimal (`capacity_value` + `capacity_unit`) per unit.

[Contoh historis: manajemen kolam ikan — satu kategori unit produksi di antara kandang dan unit lainnya](https://images.openai.com/static-rsc-4/Zi7ed0H1YZEycUI_QM6dHsJeU9xBeh-zwX5WlV3O1-KqVBOQCNGv0EhUX2_j9YtF2n2rZdEQ-NvSowuvHct-jt0WRSgPggRDOXlJI7wOxvfjU-c2OZm92v8WlvVUv9DJyLQO-8-DbhO0AkE6u7HB4vchVdZZxxV2BzUWlWn5Y6E?purpose=inline)

**2. Animal Stock Management**

- Spesies ternak: fish, crustacean, mollusk, poultry, mammal, reptile, insect, atau other.
- Tanggal mulai batch (`start_date`) dan jumlah awal (`initial_count`).
- Pergerakan stok: stok awal, tambahan stok, kelahiran, pembelian, transfer masuk/keluar.
- Catatan kematian/cull dan populasi tersisa.

[Contoh historis: pakan maggot/serangga — kategori insect, bukan satu-satunya sumber pakan](https://images.openai.com/static-rsc-4/ScpshmByDaPuLMQP_7YHxQk9QerE9KICoHiDc9RDbH9FSY_ntULS3dbLvQcf65RHrOHEzioUZyU0peFwQUa8PjK14eBowBp4LySyjn-0WGTofCXhKJfs7wnvu2yoiuCT20u8pwJPC4W76LQEa5iLFlnn2wWokGUNTkue8H-0tC8?purpose=inline)

**3. Feeding Management**

- Jenis dan harga pakan/ransum (pakan ikan, ransum unggas, hijauan, silase, maggot, dll.).
- Jumlah pakan harian beserta unitnya.
- Feeding schedule.
- Total konsumsi pakan.
- FCR (Feed Conversion Ratio) untuk produksi daging/fry; metrik telur/susu mengikuti profil spesies.

**4. Growth & Health Monitoring**

- Sampling berat dan panjang ternak (ikan, ayam, babi, kambing, sapi, dll.).
- Pencatatan telur/susu sebagai output produksi.
- Average Body Weight (ABW), ADG, dan Specific Growth Rate (SGR).
- Catatan kesehatan dan grafik pertumbuhan.

**5. Output Management**

- Estimasi waktu panen daging/fry/offspring serta koleksi telur/susu.
- Estimasi dan hasil aktual per `output_type` (meat, egg, milk, fry, cull, offspring, other).
- Jumlah dan berat aktual, grade/ukuran.
- Harga jual dan pembeli.

[Contoh historis: model finansial budidaya ikan — satu studi kasus di samping unggas dan mamalia](https://images.openai.com/static-rsc-4/f7NUhyvXJ-lUcqb-uZVz6N-hvvcItT-iQggwpcFAoNa6un4josVBhv5-yHbQd5T5Zr0B6rVsytbMILVfCzyqc7k7WjvbRLp9Lwzbty62_n89M5g-9IQgAhX6HB36Rj89uVPbLuBMvjGKBwJ3v7Ciu-Jcgumw9cGHz-e0wxmQ1uI?purpose=inline)

**6. Financial Management**

- Biaya ternak/bibit.
- Biaya pakan/ransum.
- Biaya operasional (tenaga kerja, energi, air, obat/vaksin, transportasi).
- Pendapatan penjualan daging, telur, susu, fry, atau offspring.
- Profit/loss per siklus.

# **4. Database schema — rancangan awal**

Setelah problem dan fitur ditentukan, kita bisa mulai membuat domain model.

Saya menyarankan menggunakan relational database seperti PostgreSQL.

Berikut rancangan entitas awal TernakMatric:

**Core domain**

**Farm**

id, name, location, owner_id

**Production Unit**

id, farm_id, name, unit_type_id, production_system_id, capacity_value, capacity_unit, dimensions (jsonb), area_m2, volume_m3

**Production Batch**

id, species_id, batch_code, start_date, target_end_date, initial_count, status; relasi ke unit melalui `batch_unit_assignments`

**Stock Movements**

Stok awal, tambahan stok, kelahiran, pembelian, kematian, cull, transfer, panen, koleksi telur/susu

**Feeding Records**

Pakan, kuantitas + unit, biaya, waktu

**Growth & Health Records**

Berat, panjang, jumlah sampel, catatan kesehatan

**Environment Readings**

metric_id, metric_value, unit — untuk air, udara, tanah, dan kandang

**Production Outputs**

output_type (meat, egg, milk, fry, cull, offspring, other), berat/kuantitas, harga jual

**Expense Records**

Ternak/bibit, pakan, listrik, tenaga kerja

Ini adalah logical data model; nama kanonik implementasinya mengikuti schema di `data/` (`unit_types`, `production_systems`, `production_units`, `species_production_profiles`, `environment_metrics`, `environment_readings`, `production_outputs`, dll.).

Beberapa keputusan database yang penting:

- Satu Farm bisa memiliki banyak Production Unit (kolam, kandang, cage, hive).
- Satu Production Unit bisa memiliki banyak Production Batch aktif, tetapi total biomassa/okupansi semua batch aktif harus mengikuti kapasitas unit; satu batch hanya punya satu assignment aktif.
- Semua aktivitas produksi ternak harus terhubung ke Production Batch.
- Data output produksi harus menyimpan hasil aktual (daging, telur, susu, fry), bukan hanya hasil estimasi.
- Perubahan jumlah populasi harus memiliki histori agar bisa diaudit.

## **5. Perhitungan inti yang menjadi keunggulan TernakMatric**

TernakMatric harus memiliki calculation engine yang konsisten untuk semua spesies, bukan hanya ikan.

| Metric              | Rumus dasar                                              |
| ------------------- | -------------------------------------------------------- |
| Capacity unit       | Volume/komoditas area × density atau okupansi per spesies |
| Biomass             | Populasi ternak × ABW                                    |
| Survival Rate       | Ternak hidup ÷ populasi awal × 100%                      |
| FCR                 | Total pakan ÷ pertambahan biomassa                       |
| ABW                 | Total berat sampel ÷ jumlah sampel                       |
| Weight gain / ADG   | (Berat akhir − berat awal) ÷ jumlah hari                 |
| Egg rate / milk yield | Jumlah telur atau liter susu per ekor per periode      |
| Estimasi output     | Populasi estimasi × ABW (daging/fry); laju telur/susu × populasi |
| Profit              | Pendapatan − total biaya                                 |

Catatan: FCR yang benar menggunakan pertambahan biomassa selama periode tertentu, dengan penyesuaian terhadap ternak yang dipanen, mati, atau dicull. Density, feeding rate, weight gain, egg rate, dan milk yield tidak boleh memakai angka universal: sumbernya adalah `species_production_profiles` berversi per spesies dan sistem produksi (lihat ADR 003). Estimasi output juga perlu memperhitungkan survival rate dan sampling terbaru.

# **6. Teknologi yang saya rekomendasikan**

Untuk arsitektur awal:

| Layer          | Teknologi                                                    |
| -------------- | ------------------------------------------------------------ |
| Web            | Next.js + TypeScript                                         |
| UI             | Tailwind CSS + Shadcn UI                                     |
| Mobile         | React Native + Expo + SQLite (offline-first)                 |
| Backend        | Rust + Axum                                                  |
| Kontrak API    | Utoipa → `packages/api-contracts/openapi/ternakmatric.v1.yaml` |
| Query layer    | SQLx (bukan SQLC)                                            |
| Database       | PostgreSQL                                                   |
| Authentication | JWT / Session                                                |
| Chart          | Recharts                                                     |
| Deployment     | Docker                                                       |

Saya akan memilih modular monolith dahulu, bukan microservices. Calculation Engine, Farm & Production Unit Management, Production Management, Inventory, dan Financial Management bisa dipisahkan sebagai modul tanpa menambah kompleksitas deployment. Batas tenant memakai `organization_id` dengan GUC `app.current_organization_id`.

# **7. Dokumen yang perlu kita buat sebelum coding**

Saya menyarankan membuat dokumentasi proyek dengan urutan berikut:

| Dokumen                      | Tujuan                           |
| ---------------------------- | -------------------------------- |
| `01-PROBLEM-DISCOVERY.md`    | Masalah dan kebutuhan peternak   |
| `02-PRODUCT-REQUIREMENTS.md` | Requirement dan scope fitur      |
| `03-USER-STORIES.md`         | Aktivitas dan kebutuhan pengguna |
| `04-BUSINESS-RULES.md`       | Aturan perhitungan produksi ternak per spesies |
| `05-DOMAIN-MODEL.md`         | Entitas dan relasi               |
| `06-DATABASE-ERD.md`         | Diagram database                 |
| `07-SYSTEM-ARCHITECTURE.md`  | Arsitektur aplikasi              |
| `08-API-SPECIFICATION.md`    | Kontrak API                      |

Dokumentasi ini akan menjadi fondasi pengembangan TernakMatric.

# **8. Kita mulai dari menentukan arah produk**

Sebelum membuat PRD dan database schema final, ada beberapa keputusan produk yang perlu kita ambil.

## **TernakMatric — Product Discovery**

Tentukan fondasi produk sebelum masuk ke desain teknis.

**1. Kategori spesies yang akan didukung?**

Spesies kategori fish saja (misalnya lele, nila, gurame, patin)

Hanya poultry (ayam, itik, puyuh)

Multi-kategori: fish | crustacean | mollusk | poultry | mammal | reptile | insect | other

Semua spesies ternak yang pengelolaannya bisa direpresentasikan sebagai unit + batch

**2. Target pengguna?**

Peternak ikan, udang/kerang, unggas, babi, kambing/domba, sapi/kerbau, kelinci, atau serangga

Peternak pribadi dan UMKM

Peternakan menengah dan besar

Semua segmen

**3. Platform aplikasi?**

Web application

Mobile application

Web + Mobile application

**4. Sistem monitoring?**

Manual input dahulu, IoT nanti (air, udara, tanah, kandang)

IoT sensor sejak awal

Manual input + IoT

**5. Model bisnis?**

Personal project

SaaS multi-tenant

Internal farm management

Tetapkan fondasi TernakMatric — dengan kategori spesies fish | crustacean | mollusk | poultry | mammal | reptile | insect | other

**Target akhir tahap ini:** menghasilkan PRD, business rules, dan database ERD yang sudah konsisten sebelum kita mulai menulis kode.

Saya ingin TernakMatric dibangun sebagai produk yang benar-benar bisa digunakan peternak, bukan sekadar aplikasi CRUD untuk mencatat kolam, kandang, dan spesies.
