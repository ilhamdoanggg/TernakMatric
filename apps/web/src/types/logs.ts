import type {
  ListLogsQuery,
  LogContentQuery,
  SyncLogsResponse,
  SystemLogArchiveDto,
  SystemLogContentDto,
} from "@ternakmatric/api-contracts";

export type {
  ListLogsQuery,
  LogContentQuery,
  SyncLogsResponse,
  SystemLogArchiveDto,
  SystemLogContentDto,
};

export type LogLevel = "INFO" | "WARN" | "ERROR" | "DEBUG" | "OTHER";

export type LogConnectionStatus =
  | "connecting"
  | "live"
  | "paused"
  | "disconnected";

export interface LogEntry {
  id: number;
  raw: string;
  level: LogLevel;
  timestamp: string;
  message: string;
}

export interface LogFilterOptions {
  level: LogLevel | "ALL";
  search: string;
  autoScroll: boolean;
}
