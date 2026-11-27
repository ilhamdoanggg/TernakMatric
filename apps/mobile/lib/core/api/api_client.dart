import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;

import 'sync_models.dart';

typedef TokenProvider = FutureOr<String?> Function();

/// REST API client selaras dengan kontrak OpenAPI `ternakmatric.v1.json`.
class TernakMatricApiClient {
  TernakMatricApiClient({
    required String baseUrl,
    this.getToken,
    this.getOrganizationId,
    http.Client? httpClient,
  })  : _baseUrl = baseUrl.replaceFirst(RegExp(r'/+$'), ''),
        _httpClient = httpClient ?? http.Client();

  final String _baseUrl;
  final TokenProvider? getToken;
  final TokenProvider? getOrganizationId;
  final http.Client _httpClient;

  Future<Map<String, dynamic>> requestJson(
    String path, {
    String method = 'GET',
    Object? body,
    Map<String, String>? extraHeaders,
  }) async {
    final normalizedPath = path.startsWith('/') ? path : '/$path';
    final uri = Uri.parse('$_baseUrl$normalizedPath');
    final headers = <String, String>{
      'Accept': 'application/json',
      if (body != null) 'Content-Type': 'application/json',
      ...?extraHeaders,
    };

    final token = await getToken?.call();
    if (token != null && token.isNotEmpty) {
      headers['Authorization'] = 'Bearer $token';
    }
    final organizationId = await getOrganizationId?.call();
    if (organizationId != null && organizationId.isNotEmpty) {
      headers['X-Organization-Id'] = organizationId;
    }

    final encodedBody = body == null ? null : jsonEncode(body);
    final http.Response response;
    switch (method.toUpperCase()) {
      case 'POST':
        response = await _httpClient.post(uri, headers: headers, body: encodedBody);
      case 'PATCH':
        response = await _httpClient.patch(uri, headers: headers, body: encodedBody);
      default:
        response = await _httpClient.get(uri, headers: headers);
    }

    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw ApiClientException(
        status: response.statusCode,
        rawBody: response.body,
        envelope: ErrorEnvelope.tryParse(response.body),
      );
    }

    if (response.body.trim().isEmpty) {
      return <String, dynamic>{};
    }
    final decoded = jsonDecode(response.body);
    return Map<String, dynamic>.from(decoded as Map);
  }

  Future<Map<String, dynamic>> registerDevice({
    String? deviceLabel,
    required String platform,
    required String devicePublicId,
  }) {
    return requestJson(
      '/api/v1/sync/devices',
      method: 'POST',
      body: <String, dynamic>{
        'device_label': deviceLabel,
        'platform': platform,
        'device_public_id': devicePublicId,
      },
    );
  }

  Future<SyncPushResponse> pushSyncCommands(SyncPushRequest request) async {
    final json = await requestJson(
      '/api/v1/sync/push',
      method: 'POST',
      body: request.toJson(),
    );
    return SyncPushResponse.fromJson(json);
  }

  Future<SyncPullResponse> pullSyncChanges({
    int cursor = 0,
    int limit = 100,
  }) async {
    final safeLimit = limit.clamp(1, 200);
    final json = await requestJson(
      '/api/v1/sync/pull?cursor=$cursor&limit=$safeLimit',
    );
    return SyncPullResponse.fromJson(json);
  }

  void close() {
    _httpClient.close();
  }
}
