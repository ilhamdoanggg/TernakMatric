import 'dart:convert';

enum SyncOperationType {
  create('create'),
  update('update'),
  delete('delete');

  const SyncOperationType(this.value);
  final String value;

  static SyncOperationType fromString(String raw) {
    return SyncOperationType.values.firstWhere(
      (e) => e.value == raw,
      orElse: () => SyncOperationType.create,
    );
  }
}

enum SyncStatus {
  pending('pending'),
  applied('applied'),
  rejected('rejected'),
  conflict('conflict');

  const SyncStatus(this.value);
  final String value;

  static SyncStatus fromString(String raw) {
    return SyncStatus.values.firstWhere(
      (e) => e.value == raw,
      orElse: () => SyncStatus.pending,
    );
  }
}

class ErrorEnvelope {
  const ErrorEnvelope({
    required this.code,
    required this.message,
    this.details,
  });

  final String code;
  final String message;
  final Object? details;

  static ErrorEnvelope? tryParse(String rawBody) {
    if (rawBody.trim().isEmpty) {
      return null;
    }
    try {
      final decoded = jsonDecode(rawBody);
      if (decoded is Map<String, dynamic> &&
          decoded['code'] is String &&
          decoded['message'] is String) {
        return ErrorEnvelope(
          code: decoded['code'] as String,
          message: decoded['message'] as String,
          details: decoded['details'],
        );
      }
    } catch (_) {
      // Fall back to raw HTTP body inside ApiClientException.
    }
    return null;
  }
}

class ApiClientException implements Exception {
  ApiClientException({
    required this.status,
    required this.rawBody,
    this.envelope,
  });

  final int status;
  final String rawBody;
  final ErrorEnvelope? envelope;

  String get code => envelope?.code ?? 'HTTP_ERROR';

  @override
  String toString() {
    final detail = envelope?.message ?? (rawBody.isNotEmpty ? rawBody : 'HTTP $status');
    return 'API $status ($code): $detail';
  }
}

class SyncCommand {
  const SyncCommand({
    required this.clientOperationId,
    required this.entityType,
    required this.entityId,
    required this.operationType,
    required this.deviceOccurredAt,
    required this.payload,
    this.baseVersion,
    this.serverCurrentVersion,
  });

  final String clientOperationId;
  final String entityType;
  final String entityId;
  final SyncOperationType operationType;
  final int? baseVersion;
  final int? serverCurrentVersion;
  final String deviceOccurredAt;
  final Map<String, dynamic> payload;

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'client_operation_id': clientOperationId,
      'entity_type': entityType,
      'entity_id': entityId,
      'operation_type': operationType.value,
      'base_version': baseVersion,
      'server_current_version': serverCurrentVersion,
      'device_occurred_at': deviceOccurredAt,
      'payload': payload,
    };
  }

  factory SyncCommand.fromJson(Map<String, dynamic> json) {
    return SyncCommand(
      clientOperationId: json['client_operation_id'] as String,
      entityType: json['entity_type'] as String,
      entityId: json['entity_id'] as String,
      operationType: SyncOperationType.fromString(json['operation_type'] as String),
      baseVersion: json['base_version'] as int?,
      serverCurrentVersion: json['server_current_version'] as int?,
      deviceOccurredAt: json['device_occurred_at'] as String,
      payload: Map<String, dynamic>.from(json['payload'] as Map),
    );
  }
}

class SyncPushRequest {
  const SyncPushRequest({
    required this.deviceId,
    required this.commands,
  });

  final String deviceId;
  final List<SyncCommand> commands;

  Map<String, dynamic> toJson() {
    return <String, dynamic>{
      'device_id': deviceId,
      'commands': commands.map((c) => c.toJson()).toList(),
    };
  }
}

class SyncPushItemResult {
  const SyncPushItemResult({
    required this.clientOperationId,
    required this.entityId,
    required this.status,
    required this.deduplicated,
    this.message,
  });

  final String clientOperationId;
  final String entityId;
  final SyncStatus status;
  final bool deduplicated;
  final String? message;

  factory SyncPushItemResult.fromJson(Map<String, dynamic> json) {
    return SyncPushItemResult(
      clientOperationId: json['client_operation_id'] as String,
      entityId: json['entity_id'] as String,
      status: SyncStatus.fromString(json['status'] as String),
      deduplicated: (json['deduplicated'] as bool?) ?? false,
      message: json['message'] as String?,
    );
  }
}

class SyncPushResponse {
  const SyncPushResponse({
    required this.results,
    required this.latestCursor,
  });

  final List<SyncPushItemResult> results;
  final int latestCursor;

  factory SyncPushResponse.fromJson(Map<String, dynamic> json) {
    final rawResults = json['results'] as List<dynamic>? ?? const <dynamic>[];
    return SyncPushResponse(
      results: rawResults
          .map((item) => SyncPushItemResult.fromJson(Map<String, dynamic>.from(item as Map)))
          .toList(),
      latestCursor: (json['latest_cursor'] as num?)?.toInt() ?? 0,
    );
  }
}

class OutboxChangeEvent {
  const OutboxChangeEvent({
    required this.cursor,
    required this.aggregateType,
    required this.aggregateId,
    required this.eventType,
    required this.payload,
    required this.createdAt,
  });

  final int cursor;
  final String aggregateType;
  final String aggregateId;
  final String eventType;
  final Map<String, dynamic> payload;
  final String createdAt;

  factory OutboxChangeEvent.fromJson(Map<String, dynamic> json) {
    return OutboxChangeEvent(
      cursor: (json['cursor'] as num).toInt(),
      aggregateType: json['aggregate_type'] as String,
      aggregateId: json['aggregate_id'] as String,
      eventType: json['event_type'] as String,
      payload: Map<String, dynamic>.from(json['payload'] as Map),
      createdAt: json['created_at'] as String,
    );
  }
}

class SyncPullResponse {
  const SyncPullResponse({
    required this.events,
    required this.nextCursor,
  });

  final List<OutboxChangeEvent> events;
  final int nextCursor;

  factory SyncPullResponse.fromJson(Map<String, dynamic> json) {
    final rawEvents = json['events'] as List<dynamic>? ?? const <dynamic>[];
    return SyncPullResponse(
      events: rawEvents
          .map((item) => OutboxChangeEvent.fromJson(Map<String, dynamic>.from(item as Map)))
          .toList(),
      nextCursor: (json['next_cursor'] as num?)?.toInt() ?? 0,
    );
  }
}
