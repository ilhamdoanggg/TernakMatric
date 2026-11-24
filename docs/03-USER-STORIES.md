# 03 — User Stories & Acceptance Criteria: TernakMatric

**Dokumen**: `TM-DOC-03` · **Status**: Baseline v1.0 · **Tanggal**: 2026-11-24

## 1. Tenant, Farm & Enclosure

| ID | User Story | Prioritas | Acceptance Criteria |
| --- | --- | --- | --- |
| **US-001** | Sebagai Owner, saya ingin membuat organisasi baru agar seluruh farm dan tim saya terisolasi dalam satu tenant. | Must | Organisasi terbuat dengan slug unik, mata uang default (`IDR`), zona waktu, dan pembuat otomatis menjadi `Organization Owner`. |
| **US-002** | Sebagai Admin, saya ingin mengundang staf ke organisasi dan menetapkan role per organisasi. | Must | User dapat menjadi bagian dari >1 organisasi dengan role berbeda; akses ke data organisasi lain ditolak oleh RBAC dan RLS. |
| **US-003** | Sebagai Manager, saya ingin mengelola banyak farm dan berbagai enclosure (kolam, kandang, cage, tambak, hive). | Must | Enclosure menyimpan `unit_type`, `production_system`, `dimensions`, `area_m2`, `volume_m3`, dan `capacity_value + capacity_unit`. |
| **US-004** | Sebagai Manager, saya ingin menempatkan dan memindahkan batch antar enclosure tanpa melebihi daya tampung. | Must | Satu batch maksimal memiliki 1 assignment aktif; enclosure dapat menampung >1 batch aktif selama total biomassa/okupansi gabungan ≤ kapasitas enclosure. |

## 2. Spesies, Populasi & Individual Tag

| ID | User Story | Prioritas | Acceptance Criteria |
| --- | --- | --- | --- |
| **US-005** | Sebagai Admin, saya ingin memilih spesies bawaan sistem atau membuat spesies & profil produksi custom. | Should | Sistem mendukung 8 kategori spesies; profil menyimpan `parameters` JSONB berversi (`version`, `valid_from/until`) sehingga batch lama tetap mengacu pada versi saat dibuat. |
| **US-006** | Sebagai Operator, saya ingin mencatat stok awal, pembelian, kelahiran, mutasi masuk/keluar, dan kematian/cull. | Must | Semua perubahan tercatat di `stock_movements` (append-only) dan memperbarui `population_balances` dalam satu transaksi; populasi tidak boleh menjadi negatif. |
| **US-007** | Sebagai Operator peternakan mamalia/indukan, saya ingin memberi tag individu (eartag/RFID/ring) pada ternak tertentu dalam batch. | Should | `individual_tags` mencatat `tag_code` unik per organisasi, jenis kelamin, tanggal lahir/masuk, induk, berat awal, dan status (`active`, `culled`, `deceased`, `sold`). |

## 3. Pakan, Pertumbuhan, Lingkungan & Kesehatan

| ID | User Story | Prioritas | Acceptance Criteria |
| --- | --- | --- | --- |
| **US-008** | Sebagai Operator, saya ingin mencatat pembelian dan pemakaian pakan/ransum dari gudang. | Must | Pemberian pakan (`feeding_records`) otomatis membuat transaksi keluar (`issue`) di `inventory_transactions` dan mengurangi `inventory_balances`; saldo gudang tidak boleh minus. |
| **US-009** | Sebagai Operator, saya ingin mencatat sampling berat/panjang berkala untuk memantau pertumbuhan. | Must | Sistem menghitung ABW, ADG (g/hari), SGR (%/hari), estimasi biomassa, serta FCR biologis & ekonomis dengan metadata versi formula. |
| **US-010** | Sebagai Operator, saya ingin mencatat pengukuran lingkungan kandang/kolam (suhu, pH, DO, NH3, kelembapan). | Should | Pengukuran mengacu pada `environment_metrics` dengan unit eksplisit; nilai di luar rentang profil spesies ditandai untuk evaluasi alert. |
| **US-011** | Sebagai Manager/Operator, saya ingin mengelola jadwal vaksinasi, pengobatan, dan log biosecurity. | Must | Jadwal vaksin memiliki tanggal target dan status (`scheduled`, `completed`, `overdue`, `skipped`); worker menandai pengingat H-1/jatuh tempo; biosecurity log mencatat aktivitas sanitasi/karantina. |

## 4. Output Produksi, Keuangan & Offline Mobile

| ID | User Story | Prioritas | Acceptance Criteria |
| --- | --- | --- | --- |
| **US-012** | Sebagai Operator, saya ingin mencatat panen parsial/total daging/fry maupun koleksi telur dan susu harian. | Must | `production_outputs` mencatat tipe output (`meat`, `egg`, `milk`, `fry`, `cull`, `offspring`, `other`); output yang mengurangi populasi (daging/fry/cull) wajib menulis `stock_movements` dalam transaksi yang sama. |
| **US-013** | Sebagai Finance Staff/Owner, saya ingin mencatat biaya operasional, penjualan output, serta melihat HPP per kg hidup dan Laba/Rugi per batch. | Must | Harga beli dan harga jual dibekukan saat transaksi; sistem menghitung HPP per kg hidup (atau per unit output) dan Laba/Rugi bersih per batch. |
| **US-014** | Sebagai Operator di lapangan tanpa sinyal, saya ingin tetap bisa mencatat pakan, mortalitas, dan sampling dari aplikasi mobile. | Must | Mobile menyimpan ke SQLite lokal, memasukkan command ke antrean sync dengan `client_operation_id` & `base_version`. Retry tidak menggandakan data; konflik stok/ledger tidak di-overwrite oleh `client_wins`. |
