-- TernakMatric seed data
-- Run after 001_ternakmatric_schema.sql.
-- System catalog rows are global (organization_id IS NULL). The seed is a
-- broad starter catalog, not an exhaustive list; tenants can add custom species.

INSERT INTO unit_types (code, name, habitat) VALUES
('EARTHEN', 'Kolam tanah', 'water'),
('CONCRETE', 'Kolam beton', 'water'),
('TARPAULIN', 'Kolam terpal', 'water'),
('FIBER', 'Kolam fiber', 'water'),
('TANK', 'Tangki', 'water'),
('RAS_TANK', 'Tangki resirkulasi', 'water'),
('COOP', 'Kandang ayam', 'land'),
('BATTERY_CAGE', 'Kandang baterai', 'cage'),
('PEN', 'Kandang umum', 'land'),
('PIG_PEN', 'Kandang babi', 'land'),
('GOAT_PEN', 'Kandang kambing', 'land'),
('CATTLE_PEN', 'Kandang sapi', 'land'),
('PASTURE', 'Padang penggembalaan', 'pasture'),
('HUTCH', 'Kandang kelinci', 'land'),
('BEEHIVE', 'Sarang lebah', 'hive'),
('REPTILE_ENCLOSURE', 'Kandang reptil', 'land'),
('INSECT_TRAY', 'Nampan serangga', 'other')
ON CONFLICT (code) DO NOTHING;

INSERT INTO production_systems (code, name, description) VALUES
('TRADITIONAL', 'Tradisional', 'Input dan kontrol minimal'),
('SEMI_INTENSIVE', 'Semi intensif', 'Input dan kontrol menengah'),
('INTENSIVE', 'Intensif', 'Pengelolaan pakan, kesehatan, dan lingkungan intensif'),
('BIOFLOC', 'Bioflok', 'Sistem berbasis agregat mikroba'),
('RAS', 'Recirculating Aquaculture System', 'Sistem resirkulasi air'),
('ORGANIC', 'Organik', 'Sesuai standar budidaya organik'),
('FREE_RANGE', 'Free range', 'Akses terbuka ke area outdoor'),
('CAGE_SYSTEM', 'Sistem kandang', 'Pemeliharaan dalam kandang/cage'),
('PASTURE_BASED', 'Penggembalaan', 'Pemeliharaan di padang'),
('HOUSING', 'Kandang tertutup', 'Pemeliharaan dalam gedung/kandang'),
('APICULTURE', 'Apiari', 'Pemeliharaan lebah madu')
ON CONFLICT (code) DO NOTHING;

-- Species katalog awal per kategori. Organization_id NULL = baris sistem global.
INSERT INTO species
(organization_id, category, common_name, scientific_name, code, description, is_system)
VALUES
-- Fish
(NULL, 'fish', 'Lele', 'Clarias spp.', 'CATFISH_CLARIAS', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'fish', 'Nila', 'Oreochromis niloticus', 'TILAPIA_NILE', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'fish', 'Gurame', 'Osphronemus goramy', 'GOURAMI_GIANT', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'fish', 'Patin', 'Pangasius spp.', 'PANGASIUS', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'fish', 'Ikan mas', 'Cyprinus carpio', 'CARP_COMMON', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'fish', 'Ikan tawes', 'Barbonymus gonionotus', 'BARB_TAWES', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'fish', 'Bawal air tawar', 'Colossoma macropomum', 'PACU_FRESHWATER', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'fish', 'Salmon Atlantik', 'Salmo salar', 'SALMON_ATLANTIC', 'Katalog awal untuk budidaya air dingin.', true),
(NULL, 'fish', 'Tuna sirip kuning', 'Thunnus albacares', 'TUNA_YELLOWFIN', 'Katalog awal untuk budidaya laut.', true),
-- Crustacean
(NULL, 'crustacean', 'Udang vaname', 'Litopenaeus vannamei', 'SHRIMP_VANNAMEI', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'crustacean', 'Udang windu', 'Penaeus monodon', 'SHRIMP_TIGER', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'crustacean', 'Kepiting bakau', 'Scylla serrata', 'CRAB_MANGROVE', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'crustacean', 'Lobster', 'Panulirus spp.', 'LOBSTER', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
-- Mollusk
(NULL, 'mollusk', 'Tiram', 'Crassostrea spp.', 'OYSTER', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'mollusk', 'Kerang hijau', 'Perna viridis', 'MUSSEL_GREEN', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
-- Poultry
(NULL, 'poultry', 'Ayam broiler', 'Gallus gallus domesticus', 'CHICKEN_BROILER', 'Katalog awal; strain dan sistem menentukan parameter.', true),
(NULL, 'poultry', 'Ayam petelur', 'Gallus gallus domesticus', 'CHICKEN_LAYER', 'Katalog awal; strain dan sistem menentukan parameter.', true),
(NULL, 'poultry', 'Ayam kampung', 'Gallus gallus domesticus', 'CHICKEN_NATIVE', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'poultry', 'Itik', 'Anas platyrhynchos domesticus', 'DUCK', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'poultry', 'Entok', 'Cairina moschata', 'MUSCOVY_DUCK', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'poultry', 'Angsa', 'Anser anser domesticus', 'GOOSE', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'poultry', 'Kalkun', 'Meleagris gallopavo', 'TURKEY', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'poultry', 'Burung puyuh', 'Coturnix coturnix', 'QUAIL', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
-- Mammal
(NULL, 'mammal', 'Babi', 'Sus scrofa domesticus', 'PIG', 'Katalog awal; strain dan sistem menentukan parameter.', true),
(NULL, 'mammal', 'Kambing', 'Capra aegagrus hircus', 'GOAT', 'Katalog awal; strain dan sistem menentukan parameter.', true),
(NULL, 'mammal', 'Domba', 'Ovis aries', 'SHEEP', 'Katalog awal; strain dan sistem menentukan parameter.', true),
(NULL, 'mammal', 'Sapi', 'Bos taurus', 'CATTLE', 'Katalog awal; strain dan sistem menentukan parameter.', true),
(NULL, 'mammal', 'Sapi Bali', 'Bos javanicus', 'CATTLE_BALI', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'mammal', 'Kerbau', 'Bubalus bubalis', 'BUFFALO', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'mammal', 'Kelinci', 'Oryctolagus cuniculus', 'RABBIT', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'mammal', 'Rusa', 'Cervus spp.', 'DEER', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'mammal', 'Kuda', 'Equus ferus caballus', 'HORSE', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
-- Reptile
(NULL, 'reptile', 'Buaya', 'Crocodylus spp.', 'CROCODILE', 'Katalog awal; butuh izin dan parameter khusus.', true),
(NULL, 'reptile', 'Kura-kura', 'Testudines spp.', 'TORTOISE', 'Katalog awal; butuh izin dan parameter khusus.', true),
(NULL, 'reptile', 'Ular', 'Serpentes spp.', 'SNAKE', 'Katalog awal; butuh izin dan parameter khusus.', true),
-- Insect
(NULL, 'insect', 'Lebah madu', 'Apis mellifera', 'HONEYBEE', 'Katalog awal untuk apiari.', true),
(NULL, 'insect', 'Ulat sutra', 'Bombyx mori', 'SILKWORM', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'insect', 'Jangkrik', 'Gryllus spp.', 'CRICKET', 'Katalog awal; validasi populasi lokal sebelum produksi.', true),
(NULL, 'insect', 'Maggot BSF', 'Hermetia illucens', 'BSF_MAGGOT', 'Katalog awal untuk produksi larva pakan.', true)
ON CONFLICT (organization_id, code) DO NOTHING;

INSERT INTO environment_metrics (code, name, default_unit, category, description) VALUES
('WATER_TEMP', 'Suhu air', 'degC', 'water', 'Suhu air kolam/tangki'),
('PH', 'pH', 'pH', 'water', 'Derajat keasaman air'),
('DO', 'Oksigen terlarut', 'mg/L', 'water', 'Dissolved oxygen'),
('AMMONIA_WATER', 'Amonia air', 'mg/L', 'water', 'Amonia/NH3 dalam air'),
('NITRITE', 'Nitrit', 'mg/L', 'water', 'Nitrit/NO2 dalam air'),
('SALINITY', 'Salinitas', 'ppt', 'water', 'Kadar garam'),
('AIR_TEMP', 'Suhu udara', 'degC', 'air', 'Suhu kandang/lapangan'),
('HUMIDITY', 'Kelembapan', '%', 'air', 'Kelembapan relatif'),
('CO2', 'Karbon dioksida', 'ppm', 'air', 'Konsentrasi CO2 kandang'),
('NH3_AIR', 'Amonia udara', 'ppm', 'air', 'Konsentrasi NH3 kandang'),
('LITTER_MOISTURE', 'Kelembapan litter', '%', 'housing', 'Kelembapan alas kandang'),
('SOIL_MOISTURE', 'Kelembapan tanah', '%', 'soil', 'Kelembapan padang penggembalaan'),
('LIGHT_LUX', 'Intensitas cahaya', 'lux', 'housing', 'Pencahayaan kandang'),
('FEED_MOISTURE', 'Kelembapan pakan', '%', 'feed', 'Kelembapan pakan/litter')
ON CONFLICT (code) DO NOTHING;

INSERT INTO plans (code, name, limits, features) VALUES
('FREE', 'Free', '{"users":2,"farms":1,"production_units":5}', '{"reports":true,"iot":false}'),
('GROWTH', 'Growth', '{"users":10,"farms":5,"production_units":50}', '{"reports":true,"iot":false}'),
('BUSINESS', 'Business', '{"users":50,"farms":25,"production_units":500}', '{"reports":true,"iot":true}')
ON CONFLICT (code) DO NOTHING;

INSERT INTO expense_categories (organization_id, code, name, is_system) VALUES
(NULL, 'BREEDING_STOCK', 'Bibit/induk ternak', true),
(NULL, 'FEED', 'Pakan', true),
(NULL, 'MEDICINE', 'Obat dan perlakuan', true),
(NULL, 'VACCINE', 'Vaksin', true),
(NULL, 'LITTER', 'Alas kandang', true),
(NULL, 'ELECTRICITY', 'Listrik', true),
(NULL, 'WATER', 'Air', true),
(NULL, 'LABOR', 'Tenaga kerja', true),
(NULL, 'TRANSPORT', 'Transportasi', true),
(NULL, 'EQUIPMENT', 'Peralatan', true),
(NULL, 'HOUSING', 'Konstruksi kandang/kolam', true),
(NULL, 'PROCESSING', 'Pengolahan hasil', true),
(NULL, 'OTHER', 'Lainnya', true)
ON CONFLICT (organization_id, code) DO NOTHING;

INSERT INTO calculation_versions
(code, version, formula_name, formula_definition, effective_from)
VALUES
('VOLUME_RECTANGLE', 1, 'Rectangular volume',
 '{"formula":"length_m * width_m * average_depth_m","unit":"m3","applies_to":"water units"}',
 '2026-01-01T00:00:00Z'),
('VOLUME_CIRCLE', 1, 'Circular volume',
 '{"formula":"pi * (diameter_m / 2)^2 * average_depth_m","unit":"m3","applies_to":"water units"}',
 '2026-01-01T00:00:00Z'),
('POPULATION_BALANCE', 1, 'Population ledger balance',
 '{"formula":"opening + births + purchases + transfers_in - mortality - culls - harvest - transfers_out - sales + adjustments","unit":"head"}',
 '2026-01-01T00:00:00Z'),
('BIOMASS_ESTIMATE', 1, 'Estimated biomass',
 '{"formula":"estimated_population * average_body_weight_g / 1000","unit":"kg","warning":"Population and ABW are estimates"}',
 '2026-01-01T00:00:00Z'),
('FCR_GENERIC', 1, 'Feed conversion ratio',
 '{"formula":"feed_consumed_kg / weight_gain_kg","unit":"ratio","warning":"Define weight gain consistently, accounting for outputs and mortality"}',
 '2026-01-01T00:00:00Z'),
('SURVIVAL_RATE', 1, 'Survival rate',
 '{"formula":"population_at_end / population_at_start * 100","unit":"percent"}',
 '2026-01-01T00:00:00Z'),
('MORTALITY_RATE', 1, 'Mortality rate',
 '{"formula":"mortality_count / average_population * 100","unit":"percent"}',
 '2026-01-01T00:00:00Z'),
('WEIGHT_GAIN', 1, 'Average daily gain',
 '{"formula":"(final_weight_g - initial_weight_g) / days","unit":"g/day"}',
 '2026-01-01T00:00:00Z'),
('EGG_RATE', 1, 'Egg production rate',
 '{"formula":"egg_count / laying_population * 100","unit":"percent"}',
 '2026-01-01T00:00:00Z'),
('MILK_YIELD', 1, 'Milk yield',
 '{"formula":"milk_liters / lactating_population","unit":"liter/head/day"}',
 '2026-01-01T00:00:00Z'),
('CAPACITY_BY_HEAD', 1, 'Head-based capacity utilization',
 '{"formula":"active_population / unit_capacity_head * 100","unit":"percent"}',
 '2026-01-01T00:00:00Z'),
('CAPACITY_BY_BIOMASS', 1, 'Biomass-based capacity utilization',
 '{"formula":"active_biomass_kg / unit_capacity_kg * 100","unit":"percent"}',
 '2026-01-01T00:00:00Z'),
('HARVEST_FORECAST', 1, 'Output forecast',
 '{"formula":"population * expected_body_weight_g * assumed_survival_rate","unit":"kg","warning":"Depends on latest sampling and assumptions"}',
 '2026-01-01T00:00:00Z'),
('PROFITABILITY', 1, 'Batch profitability',
 '{"formula":"total_revenue - total_feed_cost - total_other_cost","unit":"currency"}',
 '2026-01-01T00:00:00Z')
ON CONFLICT (code, version) DO NOTHING;

-- Production profiles are intentionally NOT seeded with universal density,
-- temperature, humidity, pH, DO, or feeding-rate recommendations. Those values
-- depend on species, strain, life stage, production system, and local expertise.
