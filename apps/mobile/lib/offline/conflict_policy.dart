/// Kebijakan resolusi konflik offline sesuai ADR 002 dan `infra-auth/src/sync.rs`.
const Set<String> serverAuthoritativeEntities = <String>{
  'stock_movements',
  'population_balances',
  'inventory_transactions',
  'inventory_transaction_lines',
  'inventory_balances',
  'production_outputs',
  'expenses',
  'sales',
  'feeding_records',
  'mortality_records',
};

bool allowsClientWins(String entityType) {
  return !serverAuthoritativeEntities.contains(entityType);
}

String describeConflictPolicy(String entityType) {
  if (!allowsClientWins(entityType)) {
    return 'Data ledger/stok/keuangan bersifat server-authoritative. Gunakan saldo server terbaru atau buat transaksi kompensasi.';
  }
  return 'Metadata dapat digabungkan (merge) atau dipilih secara manual oleh operator.';
}
