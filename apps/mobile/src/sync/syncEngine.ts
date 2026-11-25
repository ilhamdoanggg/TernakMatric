import type { SyncCommand, SyncPushRequest } from "@ternakmatric/api-contracts";
import { TernakMatricApiClient } from "@ternakmatric/api-contracts";

export interface SyncQueueStore {
  listPending(): Promise<SyncCommand[]>;
  markApplied(clientOperationId: string): Promise<void>;
  markConflict(clientOperationId: string, reason: string): Promise<void>;
  getCursor(): Promise<number>;
  setCursor(cursor: number): Promise<void>;
}

export async function runOfflineSyncCycle(
  client: TernakMatricApiClient,
  deviceId: string,
  store: SyncQueueStore,
): Promise<{ pushed: number; pulled: number }> {
  const pending = await store.listPending();
  let pushed = 0;

  if (pending.length > 0) {
    const req: SyncPushRequest = {
      device_id: deviceId,
      commands: pending,
    };
    const res = await client.request<{
      results: Array<{
        client_operation_id: string;
        status: string;
        message?: string;
      }>;
      latest_cursor: number;
    }>("/api/v1/sync/push", {
      method: "POST",
      body: JSON.stringify(req),
    });

    for (const item of res.results) {
      if (item.status === "applied") {
        await store.markApplied(item.client_operation_id);
        pushed += 1;
      } else if (item.status === "conflict" || item.status === "rejected") {
        await store.markConflict(
          item.client_operation_id,
          item.message ?? "Server rejected stale operation",
        );
      }
    }
  }

  const cursor = await store.getCursor();
  const pullRes = await client.request<{
    events: Array<{ cursor: number }>;
    next_cursor: number;
  }>(`/api/v1/sync/pull?cursor=${cursor}&limit=100`);

  if (pullRes.next_cursor > cursor) {
    await store.setCursor(pullRes.next_cursor);
  }

  return { pushed, pulled: pullRes.events.length };
}
