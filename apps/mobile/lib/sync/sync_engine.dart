import 'package:uuid/uuid.dart';

import '../core/api/api_client.dart';
import '../core/api/sync_models.dart';

abstract interface class SyncQueueStore {
  Future<void> enqueue(SyncCommand command);
  Future<List<SyncCommand>> listPending();
  Future<int> countPending();
  Future<void> markApplied(String clientOperationId);
  Future<void> markConflict(String clientOperationId, String reason);
  Future<int> getCursor();
  Future<void> setCursor(int cursor);
}

class SyncCycleResult {
  const SyncCycleResult({
    required this.pushed,
    required this.pulled,
    required this.conflicts,
  });

  final int pushed;
  final int pulled;
  final int conflicts;
}

SyncCommand buildFieldLogCommand({
  required String batchCode,
  required String entityId,
  required String feedKgDecimal,
  required int mortalityCount,
  required DateTime occurredAtUtc,
  Uuid uuid = const Uuid(),
}) {
  final operationId = uuid.v4();
  return SyncCommand(
    clientOperationId: operationId,
    entityType: 'feeding_records',
    entityId: entityId,
    operationType: SyncOperationType.create,
    deviceOccurredAt: occurredAtUtc.toUtc().toIso8601String(),
    payload: <String, dynamic>{
      'idempotency_key': operationId,
      'batch_code': batchCode,
      'feed_kg': feedKgDecimal,
      'mortality_count': mortalityCount,
    },
  );
}

Future<SyncCycleResult> runOfflineSyncCycle({
  required TernakMatricApiClient client,
  required String deviceId,
  required SyncQueueStore store,
}) async {
  final pending = await store.listPending();
  var pushed = 0;
  var conflicts = 0;

  if (pending.isNotEmpty) {
    final req = SyncPushRequest(deviceId: deviceId, commands: pending);
    final res = await client.pushSyncCommands(req);

    for (final item in res.results) {
      if (item.status == SyncStatus.applied) {
        await store.markApplied(item.clientOperationId);
        pushed += 1;
      } else if (item.status == SyncStatus.conflict ||
          item.status == SyncStatus.rejected) {
        await store.markConflict(
          item.clientOperationId,
          item.message ?? 'Server rejected stale operation',
        );
        conflicts += 1;
      }
    }
  }

  final cursor = await store.getCursor();
  final pullRes = await client.pullSyncChanges(cursor: cursor, limit: 100);
  if (pullRes.nextCursor > cursor) {
    await store.setCursor(pullRes.nextCursor);
  }

  return SyncCycleResult(
    pushed: pushed,
    pulled: pullRes.events.length,
    conflicts: conflicts,
  );
}
