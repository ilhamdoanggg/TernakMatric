# 01 — Problem Discovery: TernakMatric

**Dokumen**: `TM-DOC-01` · **Status**: Baseline v1.0 · **Tanggal**: 2026-11-24

## 1. Latar Belakang & Visi Produk

Peternak dan pembudidaya (ikan, udang/kerang, unggas, babi, kambing/domba, sapi/kerbau, kelinci, reptil, dan serangga/maggot) umumnya masih mengandalkan catatan kertas, buku kas, atau spreadsheet terpisah. Akibatnya, keputusan operasional—mulai dari kapasitas kandang/kolam, dosis pakan/ransum, jadwal vaksinasi, hingga waktu panen dan harga pokok produksi (HPP)—sering dibuat berdasarkan perkiraan semata.

**TernakMatric** dirancang sebagai **SaaS Multi-Tenant Smart Animal Production Management Platform** yang menyatukan pencatatan lapangan (offline-first mobile) dan analitik produksi lintas spesies (web dashboard) di atas model data dan *calculation engine* yang konsisten.

### Kategori Spesies yang Didukung

Sesuai desain domain (`species.category`), sistem mendukung 8 kategori tanpa *hardcode* kolom spesifik spesies:

| Kategori | Contoh Spesies | Contoh Unit Produksi | Output Utama |
| --- | --- | --- | --- |
| `fish` | Lele, Nila, Gurame, Patin, Mas, Salmon | Kolam tanah/terpal/beton, Tangki RAS | Daging (`meat`), Benih (`fry`) |
| `crustacean` | Udang Vaname, Udang Windu, Kepiting, Lobster | Tambak, Kolam intensif | Daging (`meat`), Benih (`fry`) |
| `mollusk` | Tiram, Kerang Hijau | Tambak, Rak gantung | Daging (`meat`) |
| `poultry` | Ayam Broiler, Layer, Kampung, Itik, Puyuh | Kandang lantai/litter, Battery cage | Daging (`meat`), Telur (`egg`) |
| `mammal` | Babi, Kambing, Domba, Sapi, Kerbau, Kelinci | Kandang sekat (`pen`), Padang gembala | Daging (`meat`), Susu (`milk`), Anakan (`offspring`) |
| `reptile` | Buaya, Kura-kura, Ular | Enclosure khusus | Anakan (`offspring`), Daging/kulit (`other`) |
| `insect` | Maggot BSF, Jangkrik, Lebah Madu, Ulat Sutra | Nampan (`tray`), Sarang (`hive`) | Larva/biomassa (`meat`/`other`), Madu/kokon (`other`) |
| `other` | Spesies custom milik tenant | Unit produksi custom | Daging, telur, susu, atau output lain |

---

## 2. Identifikasi Masalah Lapangan (Problem Statements)

| ID | Masalah di Lapangan | Dampak Bisnis | Arah Solusi TernakMatric |
| --- | --- | --- | --- |
| **PS-01** | Kesulitan menghitung daya tampung kandang, kolam, cage, atau rak | *Overstocking* memicu stres, penyakit, dan kematian massal; *understocking* menurunkan efisiensi lahan | Kalkulator kapasitas unit berbasis `area_m2`, `volume_m3`, atau kapasitas per ekor/kg dengan validasi multi-batch |
| **PS-02** | Jumlah populasi aktual tidak sinkron (kelahiran, pembelian, mortalitas, cull, panen parsial tidak terrekonsiliasi) | Estimasi kebutuhan pakan meleset; penyusutan/kehilangan stok tidak terdeteksi | Ledger populasi *append-only* (`stock_movements`) + proyeksi saldo (`population_balances`) + dukungan *individual tag* |
| **PS-03** | Pemakaian pakan/ransum dan stok gudang tidak terukur per batch | Pemborosan pakan (60–75% total biaya produksi) dan keterlambatan *restock* | Integrasi pemberian pakan dengan gudang (`inventory_transactions`), konversi unit, dan pelacakan FCR |
| **PS-04** | Pertumbuhan (ABW, ADG, SGR) jarang diukur atau tidak dikaitkan ke histori pakan | Waktu panen tidak akurat dan tren perlambatan pertumbuhan terlambat ditangani | Pencatatan `growth_samples` periodik dengan perhitungan otomatis ABW, ADG, SGR, serta kurva pertumbuhan |
| **PS-05** | Parameter lingkungan (air: DO, pH, amonia; kandang: suhu, kelembapan, NH3, CO2) tidak terdokumentasi | Wabah penyakit dan penurunan nafsu makan tidak terlacak penyebabnya | Pencatatan `environment_readings` manual (MVP) dan kesiapan ingesti sensor IoT (V2) |
| **PS-06** | Jadwal vaksinasi, pengobatan, dan log biosecurity tersebar | Keterlambatan vaksin memicu mortalitas tinggi; kepatuhan biosecurity sulit diaudit | Modul kesehatan terdedikasi (`mod-health`): jadwal vaksin, pengobatan, mortalitas, dan log biosecurity |
| **PS-07** | Biaya operasional tidak teralokasi ke batch; HPP per kg hidup tidak diketahui | Peternak menjual hasil panen tanpa tahu margin bersih sebenarnya | Alokasi biaya per batch/farm, perhitungan HPP per kg hidup/output, dan laporan Laba/Rugi |
| **PS-08** | Lokasi kandang/kolam sering minim sinyal internet | Operator menunda input data ke sore/malam sehingga data rawan lupa atau keliru | Mobile app *offline-first* (SQLite lokal + *command queue* + sinkronisasi idempoten) |

---

## 3. Prinsip Produk & Batasan Penting

1. **Bedakan Aktual, Estimasi, dan Target**:
   - Populasi yang berasal dari *ledger* adalah angka buku terkini, sedangkan biomassa total dan proyeksi panen adalah **estimasi** berbasis sampling berat terakhir dan asumsi *survival rate*. Setiap angka estimasi wajib menyertakan tanggal sampling, versi formula, dan asumsi yang dipakai.
2. **Jangan Hardcode Standar Biologis Universal**:
   - Kepadatan tebar dan *feeding rate* tidak boleh memakai satu nilai global untuk semua spesies. Seluruh target biologis dan ambang lingkungan disimpan sebagai data berversi di `species_production_profiles`.
3. **Auditabilitas dan Konsistensi Ledger**:
   - Pergerakan populasi, stok pakan, dan output produksi bersifat *append-only*. Koreksi dilakukan lewat transaksi pembalik (*reversal*) atau penyesuaian (*adjustment*), bukan menimpa histori.
4. **Satu Platform, Bukan Aplikasi CRUD Terpisah**:
   - Nilai utama TernakMatric ada pada keterhubungan lintas modul: pakan mengurangi gudang dan menghitung FCR; mortalitas mengurangi populasi dan memicu peringatan; panen dan biaya otomatis membentuk HPP dan Laba/Rugi.

---

## 4. Hipotesis yang Perlu Divalidasi pada Pilot

- **H1**: Peternak multi-komoditas bersedia mengganti catatan buku/Excel jika alur input harian di HP selesai dalam < 60 detik dan tetap berjalan saat offline.
- **H2**: Kalkulasi otomatis FCR, ADG/SGR, dan HPP per kg hidup membantu peternak mendeteksi pemborosan pakan lebih awal dibanding rekap akhir siklus.
- **H3**: Pemisahan hak akses (*Owner*, *Farm Manager*, *Operator*, *Finance*) mengurangi kecurangan stok pakan dan selisih populasi antar-shift.
