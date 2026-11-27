import { createWebApiClient } from "@/lib/api";
import type {
  ListLogsQuery,
  LogConnectionStatus,
  LogEntry,
  LogLevel,
  SyncLogsResponse,
  SystemLogArchiveDto,
  SystemLogContentDto,
} from "@/types/logs";

export function parseRawLogLine(raw: string, id: number): LogEntry {
  let level: LogLevel = "OTHER";
  if (raw.includes("ERROR") || raw.includes("error")) {
    level = "ERROR";
  } else if (raw.includes("WARN") || raw.includes("warn")) {
    level = "WARN";
  } else if (raw.includes("INFO") || raw.includes("info")) {
    level = "INFO";
  } else if (raw.includes("DEBUG") || raw.includes("debug")) {
    level = "DEBUG";
  }

  const tsMatch = raw.match(
    /\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?/,
  );
  const timestamp = tsMatch ? tsMatch[0] : "";
  const message = timestamp ? raw.replace(timestamp, "").trim() : raw;

  return {
    id,
    raw,
    level,
    timestamp,
    message,
  };
}

export class LogService {
  constructor(
    private readonly getToken?: () => string | null,
    private readonly getOrgId?: () => string | null,
  ) {}

  private getClient() {
    return createWebApiClient(this.getToken?.() ?? undefined, this.getOrgId?.() ?? undefined);
  }

  async listArchives(query?: ListLogsQuery): Promise<SystemLogArchiveDto[]> {
    const client = this.getClient();
    return client.listSystemLogs(query);
  }

  async getContent(
    fileName: string,
    limit: number = 500,
    tail: boolean = true,
  ): Promise<SystemLogContentDto> {
    const client = this.getClient();
    return client.getSystemLogContent(fileName, { limit, tail });
  }

  async sync(): Promise<SyncLogsResponse> {
    const client = this.getClient();
    return client.syncSystemLogs();
  }

  getStreamUrl(): string {
    const client = this.getClient();
    return client.getLogStreamUrl();
  }

  async streamRealtimeLogs(
    onLine: (entry: LogEntry) => void,
    onStatusChange: (status: LogConnectionStatus) => void,
    signal: AbortSignal,
  ): Promise<void> {
    if (import.meta.env.SSR) {
      return;
    }

    onStatusChange("connecting");

    const streamUrl = this.getStreamUrl();
    const token = this.getToken?.();
    const orgId = this.getOrgId?.();

    const headers: Record<string, string> = {
      Accept: "text/event-stream",
    };
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }
    if (orgId) {
      headers["X-Organization-Id"] = orgId;
    }

    try {
      const response = await fetch(streamUrl, {
        method: "GET",
        headers,
        signal,
      });

      if (!response.ok || !response.body) {
        onStatusChange("disconnected");
        return;
      }

      onStatusChange("live");
      const reader = response.body.getReader();
      const decoder = new TextDecoder("utf-8");
      let buffer = "";
      let lineCounter = 0;

      while (!signal.aborted) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const rawEvents = buffer.split("\n\n");
        buffer = rawEvents.pop() ?? "";

        for (const eventBlock of rawEvents) {
          if (!eventBlock.trim()) continue;
          const eventLines = eventBlock.split("\n");
          for (const line of eventLines) {
            if (line.startsWith("data:")) {
              const data = line.slice(5).trim();
              if (data) {
                const entry = parseRawLogLine(data, ++lineCounter);
                onLine(entry);
              }
            }
          }
        }
      }

      onStatusChange("disconnected");
    } catch (err: unknown) {
      if ((err as Error)?.name !== "AbortError") {
        onStatusChange("disconnected");
      }
    }
  }
}

export const logService = new LogService();
