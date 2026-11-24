# 04 — Business Rules & Calculation Engine: TernakMatric

**Dokumen**: `TM-DOC-04` · **Status**: Baseline v1.0 · **Tanggal**: 2026-11-24

Semua aturan perhitungan di bawah diimplementasikan dalam crate pure domain **`backend/crates/mod-calculation`** tanpa ketergantungan database, HTTP, maupun jam sistem (*deterministic & pure*). Setiap hasil kalkulasi wajib mengembalikan nilai, satuan eksplisit, `formula_version`, `parameter_version`, dan `assumptions`.

---

## 1. Kapasitas & Geometri Enclosure (`capacity.rs`)

### 1.1 Volume & Luas Enclosure
- **Persegi panjang (kolam/tangki/bak)**:
  $$V = L \times W \times D \quad (\text{m}^3), \qquad A = L \times W \quad (\text{m}^2)$$
- **Silinder/bundar (kolam terpal bundar/tangki RAS)**:
  $$V = \pi \times \left(\frac{d}{2}\right)^2 \times D \quad (\text{m}^3), \qquad A = \pi \times \left(\frac{d}{2}\right)^2 \quad (\text{m}^2)$$
- **Kandang/Cage/Pasture/Tray**: menggunakan `area_m2` atau jumlah kompartemen/kapasitas nominal (`capacity_value` + `capacity_unit`).

### 1.2 Kapasitas & Validasi Okupansi Multi-Batch
- Kapasitas maksimum tidak boleh menggunakan satu angka global. Nilai referensi berasal dari `species_production_profiles.parameters` atau konfigurasi `production_units` (`head`, `kg`, `m3`, `m2`, `tray`, `crate`, `colony`, `hive`).
- Untuk unit yang menampung lebih dari satu batch aktif secara bersamaan:
  $$\text{Total Occupancy} = \sum_{b \in \text{active}(u)} \text{Occupancy}(b) \le \text{MaxCapacity}(u)$$
- Validasi wajib dijalankan di dalam transaksi database dengan mengunci baris `production_units` (`SELECT ... FOR UPDATE`).

---

## 2. Pelacakan Populasi & Tingkat Kelangsungan Hidup (`population.rs`)

### 2.1 Saldo Populasi Aktual
$$P_t = P_0 + S + B + \text{Pur} + T_{\text{in}} - M - C - H_{\text{pop}} - T_{\text{out}} + \text{Adj}$$

Keterangan tipe `stock_movements`:
- Penambah: `initial_stock` ($P_0$), `additional_stock` ($S$), `birth` ($B$), `purchase` ($\text{Pur}$), `transfer_in` ($T_{\text{in}}$).
- Pengurang: `mortality` ($M$), `cull` ($C$), `harvest` / `sale` yang mengurangi ekor hidup ($H_{\text{pop}}$), `transfer_out` ($T_{\text{out}}$).
- Non-populasi: `egg_collection` dan `milk_collection` dicatat di `production_outputs` dan **tidak** mengurangi jumlah ekor hidup (`quantity_delta` tidak mengurangi saldo ekor).
- **Invarian**: $P_t \ge 0$. Setiap transaksi kematian, cull, transfer keluar, atau panen yang membuat $P_t < 0$ wajib ditolak.

### 2.2 Survival Rate (SR) & Mortality Rate
- **Survival Rate**:
  $$SR(\%) = \frac{P_{\text{alive}} + H_{\text{pop}} + T_{\text{out}}}{P_{\text{stocked}}} \times 100 \quad \text{(metode gross, atau } \frac{P_{\text{alive}}}{P_{\text{stocked}} - H_{\text{pop}} - T_{\text{out}}} \times 100 \text{ untuk net)}$$
  dengan $P_{\text{stocked}} = P_0 + S + B + \text{Pur} + T_{\text{in}}$. Engine mendefinisikan mode eksplisit (`NetExistingStock` vs `CumulativeAccounted`) agar panen parsial tidak terbaca sebagai kematian.
- **Mortality Rate**:
  $$MR(\%) = \frac{M}{P_{\text{stocked}}} \times 100$$

---

## 3. Pertumbuhan: ABW, ADG, SGR & Output Non-Daging (`growth.rs`)

### 3.1 Average Body Weight (ABW)
$$\text{ABW (g/ekor)} = \frac{\text{Total Sample Weight (g)}}{\text{Sample Count (ekor)}}$$

### 3.2 Average Daily Gain (ADG) — Umum untuk Unggas, Mamalia, & Ikan
$$\text{ADG (g/hari)} = \frac{\text{ABW}_{\text{akhir}} - \text{ABW}_{\text{awal}}}{\Delta t_{\text{hari}}}$$
*(Untuk ternak besar seperti sapi/babi/kambing dapat dikonversi ke kg/hari: $\text{ADG}_{\text{kg}} = \text{ADG}_{\text{g}} / 1000$).*

### 3.3 Specific Growth Rate (SGR) — Standar Akuakultur & Fase Cepat
$$\text{SGR (\%/hari)} = \frac{\ln(\text{ABW}_{\text{akhir}}) - \ln(\text{ABW}_{\text{awal}})}{\Delta t_{\text{hari}}} \times 100$$
Syarat domain: $\text{ABW}_{\text{awal}} > 0$, $\text{ABW}_{\text{akhir}} > 0$, dan $\Delta t_{\text{hari}} > 0$.

### 3.4 Output Berkelanjutan (Layer & Dairy)
- **Egg Production Rate (Hen-Day / Hen-Housed %)**:
  $$\text{Egg Rate (\%)} = \frac{\text{Total Eggs Collected}}{\text{Laying Population} \times \text{Days}} \times 100$$
- **Milk Yield (liter/ekor/hari)**:
  $$\text{Milk Yield} = \frac{\text{Total Milk (L)}}{\text{Lactating Population} \times \text{Days}}$$

---

## 4. Biomassa & Feed Conversion Ratio (`biomass.rs`, `fcr.rs`)

### 4.1 Estimasi Biomassa
$$\text{Biomass (kg)} = \frac{\text{Estimated Population (ekor)} \times \text{ABW (g)}}{1000}$$
Hasil selalu ditandai sebagai **estimasi** (*estimate*), bukan penimbangan total populasi.

### 4.2 Feed Conversion Ratio (FCR)
Semua input pakan dinormalisasi terlebih dahulu ke satuan kanonik **`kg`** melalui `feed_unit_conversions`.

- **Economic FCR** (berbasis output hidup yang dipanen + sisa biomassa akhir):
  $$\text{FCR}_{\text{econ}} = \frac{\text{Total Feed (kg)}}{\left(\text{Current Biomass} + \text{Harvested Biomass} - \text{Initial Biomass} - \text{Added Stock Biomass}\right)}$$
- **Biological FCR** (memperhitungkan biomassa ternak mati & cull yang sempat mengonsumsi pakan):
  $$\text{FCR}_{\text{bio}} = \frac{\text{Total Feed (kg)}}{\left(\text{Current Biomass} + \text{Harvested Biomass} + \text{Mortality/Cull Biomass} - \text{Initial Biomass} - \text{Added Stock Biomass}\right)}$$
- Jika pertambahan biomassa $\le 0$, engine mengembalikan status domain terdefinisi (`NonPositiveBiomassGain`) alih-alih `NaN` atau pembagian nol.

---

## 5. Harga Pokok Produksi (HPP) & Profitabilitas (`finance.rs`)

### 5.1 HPP per Kg Hidup (Live Weight Cost of Production)
$$\text{Total Cost} = \text{Bibit/Stock Cost} + \text{Feed Cost} + \text{Health/Vaccine Cost} + \text{Allocated Operational Cost}$$
$$\text{HPP per kg hidup} = \frac{\text{Total Cost}}{\text{Total Live Biomass Produced (kg)}}$$
Di mana $\text{Total Live Biomass Produced} = \text{Harvested Live Weight (kg)} + \text{Remaining Active Biomass (kg)}$ (sesuai mode: *realized harvest* vs *running estimate*).

Untuk komoditas non-daging, tersedia varian **HPP per unit output** (mis. Rp/butir atau Rp/kg telur, Rp/liter susu):
$$\text{HPP per unit output} = \frac{\text{Total Cost}}{\text{Total Output Quantity}}$$

### 5.2 Laba / Rugi Batch (P&L)
$$\text{Gross Revenue} = \sum \text{Sale Items Line Total}$$
$$\text{Net Profit} = \text{Gross Revenue} - \text{Total Cost}$$
$$\text{Profit Margin (\%)} = \frac{\text{Net Profit}}{\text{Gross Revenue}} \times 100$$

---

## 6. Aturan Integritas Lintas Domain

1. **Batch Tertutup**: Batch berstatus `completed` atau `cancelled` tidak boleh menerima transaksi operasional baru (`feeding`, `mortality`, `output`) tanpa transaksi pembatalan/penyesuaian yang diaudit.
2. **Ledger Append-Only**: `stock_movements` dan `inventory_transaction_lines` tidak boleh di-`UPDATE` atau di-`DELETE` (dijaga trigger DB). Koreksi wajib memakai `reversal_of`.
3. **Konflik Offline**: Entitas finansial, inventory gudang, output produksi, dan pergerakan stok dilarang memakai strategi `client_wins`.
4. **Snapshot Harga**: Biaya pakan di `feeding_records` dan harga jual di `sale_items` dibekukan saat transaksi terjadi; perubahan master harga pakan di kemudian hari tidak mengubah histori keuangan.
