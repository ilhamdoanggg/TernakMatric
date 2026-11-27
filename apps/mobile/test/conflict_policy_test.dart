import 'package:flutter_test/flutter_test.dart';
import 'package:ternakmatric_mobile/offline/conflict_policy.dart';

void main() {
  test('forbids client_wins on ledger, inventory, output, and finance entities', () {
    for (final entity in <String>[
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
    ]) {
      expect(allowsClientWins(entity), isFalse, reason: '$entity must be server-authoritative');
      expect(describeConflictPolicy(entity), contains('server-authoritative'));
    }
  });

  test('allows non-ledger metadata entities to be merged or manually resolved', () {
    expect(allowsClientWins('enclosure_notes'), isTrue);
    expect(describeConflictPolicy('enclosure_notes'), contains('merge'));
  });
}
