const SERVER_AUTHORITATIVE_ENTITIES = new Set([
  "stock_movements",
  "inventory_transactions",
  "inventory_transaction_lines",
  "expenses",
  "sales",
  "production_outputs",
  "feeding_records",
  "mortality_records",
]);

export function allowsClientWins(entityType: string): boolean {
  return !SERVER_AUTHORITATIVE_ENTITIES.has(entityType);
}

export function describeConflictPolicy(entityType: string): string {
  if (!allowsClientWins(entityType)) {
    return "Data ledger/stok/keuangan bersifat server-authoritative. Gunakan saldo server terbaru atau buat transaksi kompensasi.";
  }
  return "Metadata dapat digabungkan (merge) atau dipilih secara manual oleh operator.";
}
