import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:ternakmatric_mobile/core/api/api_client.dart';
import 'package:ternakmatric_mobile/core/api/sync_models.dart';
import 'package:ternakmatric_mobile/sync/sync_engine.dart';

class _InMemoryQueueStore implements SyncQueueStore {
  final List<SyncCommand> _items = <SyncCommand>[];
  final Map<String, String> _statuses = <String, String>{};
  final Map<String, String> conflicts = <String, String>{};
  int _cursor = 10;

  @override
  Future<void> enqueue(SyncCommand command) async {
    _items.add(command);
    _statuses[command.clientOperationId] = 'pending';
  }

  @override
  Future<List<SyncCommand>> listPending() async {
    return _items
        .where((c) => _statuses[c.clientOperationId] == 'pending')
        .toList();
  }

  @override
  Future<int> countPending() async => (await listPending()).length;

  @override
  Future<void> markApplied(String clientOperationId) async {
    _statuses[clientOperationId] = 'applied';
  }

  @override
  Future<void> markConflict(String clientOperationId, String reason) async {
    _statuses[clientOperationId] = 'conflict';
    conflicts[clientOperationId] = reason;
  }

  @override
  Future<int> getCursor() async => _cursor;

  @override
  Future<void> setCursor(int cursor) async {
    _cursor = cursor;
  }
}

void main() {
  test('runOfflineSyncCycle pushes pending commands, records conflicts, and advances cursor', () async {
    final store = _InMemoryQueueStore();
    final opApplied = buildFieldLogCommand(
      batchCode: 'B-NILA-2611',
      entityId: '11111111-1111-4111-8111-111111111111',
      feedKgDecimal: '12.500',
      mortalityCount: 0,
      occurredAtUtc: DateTime.utc(2026, 11, 27, 0, 0),
    );
    final opConflict = buildFieldLogCommand(
      batchCode: 'B-NILA-2612',
      entityId: '22222222-2222-4222-8222-222222222222',
      feedKgDecimal: '5.000',
      mortalityCount: 1,
      occurredAtUtc: DateTime.utc(2026, 11, 27, 0, 5),
    );
    await store.enqueue(opApplied);
    await store.enqueue(opConflict);

    final mockHttp = MockClient((http.Request request) async {
      expect(request.headers['Authorization'], 'Bearer test-jwt');
      expect(request.headers['X-Organization-Id'], 'org-123');

      if (request.url.path == '/api/v1/sync/push') {
        return http.Response(
          jsonEncode(<String, dynamic>{
            'results': <Map<String, dynamic>>[
              <String, dynamic>{
                'client_operation_id': opApplied.clientOperationId,
                'entity_id': opApplied.entityId,
                'status': 'applied',
                'deduplicated': false,
              },
              <String, dynamic>{
                'client_operation_id': opConflict.clientOperationId,
                'entity_id': opConflict.entityId,
                'status': 'conflict',
                'deduplicated': false,
                'message': 'Stale base_version',
              },
            ],
            'latest_cursor': 14,
          }),
          200,
        );
      }

      if (request.url.path == '/api/v1/sync/pull') {
        expect(request.url.queryParameters['cursor'], '10');
        return http.Response(
          jsonEncode(<String, dynamic>{
            'events': <Map<String, dynamic>>[
              <String, dynamic>{
                'cursor': 14,
                'aggregate_type': 'batch',
                'aggregate_id': opApplied.entityId,
                'event_type': 'feeding.recorded',
                'payload': <String, dynamic>{'feed_kg': '12.500'},
                'created_at': '2026-11-27T00:05:00Z',
              },
            ],
            'next_cursor': 14,
          }),
          200,
        );
      }

      return http.Response('Not found', 404);
    });

    final client = TernakMatricApiClient(
      baseUrl: 'http://localhost:8080',
      getToken: () => 'test-jwt',
      getOrganizationId: () => 'org-123',
      httpClient: mockHttp,
    );

    final result = await runOfflineSyncCycle(
      client: client,
      deviceId: '33333333-3333-4333-8333-333333333333',
      store: store,
    );

    expect(result.pushed, 1);
    expect(result.conflicts, 1);
    expect(result.pulled, 1);
    expect(await store.countPending(), 0);
    expect(store.conflicts[opConflict.clientOperationId], 'Stale base_version');
    expect(await store.getCursor(), 14);
  });
}
