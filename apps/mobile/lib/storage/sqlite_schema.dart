import 'dart:convert';

import 'package:path/path.dart' as p;
import 'package:sqflite/sqflite.dart';

import '../core/api/sync_models.dart';
import '../sync/sync_engine.dart';

/// Migrasi SQLite lokal untuk penyimpanan offline dan antrean sync (ADR 002).
const List<String> sqliteMigrations = <String>[
  '''
  CREATE TABLE IF NOT EXISTS local_enclosures (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL,
    name TEXT NOT NULL,
    capacity_value TEXT,
    capacity_unit TEXT,
    version INTEGER NOT NULL DEFAULT 1
  );
  ''',
  '''
  CREATE TABLE IF NOT EXISTS local_batches (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL,
    batch_code TEXT NOT NULL,
    species_id TEXT NOT NULL,
    enclosure_id TEXT NOT NULL,
    current_population INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1
  );
  ''',
  '''
  CREATE TABLE IF NOT EXISTS pending_sync_operations (
    client_operation_id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    base_version INTEGER,
    payload_json TEXT NOT NULL,
    device_occurred_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
  );
  ''',
  '''
  CREATE TABLE IF NOT EXISTS local_sync_conflicts (
    id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    conflict_type TEXT NOT NULL,
    server_snapshot_json TEXT,
    client_snapshot_json TEXT,
    created_at TEXT NOT NULL
  );
  ''',
  '''
  CREATE TABLE IF NOT EXISTS sync_state (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
  );
  ''',
];

class SqfliteQueueStore implements SyncQueueStore {
  SqfliteQueueStore._(this._db);

  final Database _db;
  static const String _cursorKey = 'sync_pull_cursor';

  static Future<SqfliteQueueStore> open({String fileName = 'ternakmatric_field.db'}) async {
    final dbDir = await getDatabasesPath();
    final dbPath = p.join(dbDir, fileName);
    final db = await openDatabase(
      dbPath,
      version: 1,
      onCreate: (Database database, int version) async {
        for (final statement in sqliteMigrations) {
          await database.execute(statement);
        }
      },
    );
    return SqfliteQueueStore._(db);
  }

  @override
  Future<void> enqueue(SyncCommand command) async {
    await _db.insert(
      'pending_sync_operations',
      <String, Object?>{
        'client_operation_id': command.clientOperationId,
        'entity_type': command.entityType,
        'entity_id': command.entityId,
        'operation_type': command.operationType.value,
        'base_version': command.baseVersion,
        'payload_json': jsonEncode(command.payload),
        'device_occurred_at': command.deviceOccurredAt,
        'status': 'pending',
      },
      conflictAlgorithm: ConflictAlgorithm.ignore,
    );
  }

  @override
  Future<List<SyncCommand>> listPending() async {
    final rows = await _db.query(
      'pending_sync_operations',
      where: 'status = ?',
      whereArgs: const <Object>['pending'],
      orderBy: 'device_occurred_at ASC',
    );
    return rows.map((row) {
      final decodedPayload = jsonDecode(row['payload_json']! as String);
      return SyncCommand(
        clientOperationId: row['client_operation_id']! as String,
        entityType: row['entity_type']! as String,
        entityId: row['entity_id']! as String,
        operationType: SyncOperationType.fromString(row['operation_type']! as String),
        baseVersion: row['base_version'] as int?,
        deviceOccurredAt: row['device_occurred_at']! as String,
        payload: Map<String, dynamic>.from(decodedPayload as Map),
      );
    }).toList();
  }

  @override
  Future<int> countPending() async {
    final result = await _db.rawQuery(
      "SELECT COUNT(*) AS cnt FROM pending_sync_operations WHERE status = 'pending'",
    );
    return Sqflite.firstIntValue(result) ?? 0;
  }

  @override
  Future<void> markApplied(String clientOperationId) async {
    await _db.update(
      'pending_sync_operations',
      const <String, Object?>{'status': 'applied'},
      where: 'client_operation_id = ?',
      whereArgs: <Object>[clientOperationId],
    );
  }

  @override
  Future<void> markConflict(String clientOperationId, String reason) async {
    await _db.transaction((txn) async {
      final rows = await txn.query(
        'pending_sync_operations',
        where: 'client_operation_id = ?',
        whereArgs: <Object>[clientOperationId],
        limit: 1,
      );
      await txn.update(
        'pending_sync_operations',
        const <String, Object?>{'status': 'conflict'},
        where: 'client_operation_id = ?',
        whereArgs: <Object>[clientOperationId],
      );
      if (rows.isNotEmpty) {
        final op = rows.first;
        await txn.insert(
          'local_sync_conflicts',
          <String, Object?>{
            'id': clientOperationId,
            'entity_type': op['entity_type'],
            'entity_id': op['entity_id'],
            'conflict_type': 'server_rejected',
            'server_snapshot_json': jsonEncode(<String, dynamic>{'message': reason}),
            'client_snapshot_json': op['payload_json'],
            'created_at': DateTime.now().toUtc().toIso8601String(),
          },
          conflictAlgorithm: ConflictAlgorithm.replace,
        );
      }
    });
  }

  @override
  Future<int> getCursor() async {
    final rows = await _db.query(
      'sync_state',
      where: 'key = ?',
      whereArgs: const <Object>[_cursorKey],
      limit: 1,
    );
    if (rows.isEmpty) {
      return 0;
    }
    return int.tryParse(rows.first['value'] as String? ?? '0') ?? 0;
  }

  @override
  Future<void> setCursor(int cursor) async {
    await _db.insert(
      'sync_state',
      <String, Object?>{'key': _cursorKey, 'value': '$cursor'},
      conflictAlgorithm: ConflictAlgorithm.replace,
    );
  }

  Future<void> close() => _db.close();
}
