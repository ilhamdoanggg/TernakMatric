import { computed, ref } from "vue";
import { LogService } from "@/services/logService";
import { useAuthStore } from "./authStore";
import type {
  LogConnectionStatus,
  LogEntry,
  LogLevel,
  SystemLogArchiveDto,
  SystemLogContentDto,
} from "@/types/logs";

export function useLogStore() {
  const authStore = useAuthStore();
  const logService = new LogService(
    () => authStore.token.value,
    () => authStore.organizationId.value,
  );

  const archives = ref<SystemLogArchiveDto[]>([]);
  const loadingArchives = ref(false);
  const syncing = ref(false);

  const selectedArchive = ref<SystemLogArchiveDto | null>(null);
  const logContent = ref<SystemLogContentDto | null>(null);
  const loadingContent = ref(false);

  const realtimeLines = ref<LogEntry[]>([]);
  const connectionStatus = ref<LogConnectionStatus>("disconnected");
  const autoScroll = ref(true);
  const selectedLevel = ref<LogLevel | "ALL">("ALL");
  const searchFilter = ref("");
  const notification = ref<{ message: string; type: "success" | "error" } | null>(null);

  let streamAbortController: AbortController | null = null;

  function showNotification(message: string, type: "success" | "error" = "success") {
    notification.value = { message, type };
    setTimeout(() => {
      notification.value = null;
    }, 4000);
  }

  const filteredLines = computed(() => {
    return realtimeLines.value.filter((entry) => {
      if (selectedLevel.value !== "ALL" && entry.level !== selectedLevel.value) {
        return false;
      }
      if (searchFilter.value.trim()) {
        const q = searchFilter.value.toLowerCase();
        return entry.raw.toLowerCase().includes(q);
      }
      return true;
    });
  });

  async function fetchArchives() {
    loadingArchives.value = true;
    try {
      const data = await logService.listArchives({ limit: 100 });
      archives.value = data;
    } catch (err) {
      showNotification(
        `Gagal memuat arsip log: ${(err as Error)?.message ?? "Terjadi kesalahan"}`,
        "error",
      );
    } finally {
      loadingArchives.value = false;
    }
  }

  async function syncLogs() {
    syncing.value = true;
    try {
      const res = await logService.sync();
      showNotification(
        `Berhasil menyinkronkan ${res.synced_archives} berkas log ke database dari ${res.log_dir}.`,
        "success",
      );
      await fetchArchives();
    } catch (err) {
      showNotification(
        `Gagal sinkronisasi log: ${(err as Error)?.message ?? "Terjadi kesalahan"}`,
        "error",
      );
    } finally {
      syncing.value = false;
    }
  }

  async function fetchContent(fileName: string, limit: number = 500, tail: boolean = true) {
    loadingContent.value = true;
    try {
      const res = await logService.getContent(fileName, limit, tail);
      logContent.value = res;
    } catch (err) {
      showNotification(
        `Gagal membaca berkas log '${fileName}': ${(err as Error)?.message ?? "Terjadi kesalahan"}`,
        "error",
      );
    } finally {
      loadingContent.value = false;
    }
  }

  function selectArchive(archive: SystemLogArchiveDto) {
    selectedArchive.value = archive;
    fetchContent(archive.file_name, 500, true);
  }

  function clearSelectedArchive() {
    selectedArchive.value = null;
    logContent.value = null;
  }

  function startRealtimeStream() {
    if (import.meta.env.SSR) return;
    stopRealtimeStream();

    streamAbortController = new AbortController();
    logService.streamRealtimeLogs(
      (entry) => {
        if (connectionStatus.value === "live") {
          realtimeLines.value.push(entry);
          if (realtimeLines.value.length > 500) {
            realtimeLines.value.splice(0, realtimeLines.value.length - 500);
          }
        }
      },
      (status) => {
        connectionStatus.value = status;
      },
      streamAbortController.signal,
    );
  }

  function stopRealtimeStream() {
    if (streamAbortController) {
      streamAbortController.abort();
      streamAbortController = null;
    }
    connectionStatus.value = "disconnected";
  }

  function togglePause() {
    if (connectionStatus.value === "live") {
      connectionStatus.value = "paused";
    } else if (connectionStatus.value === "paused") {
      connectionStatus.value = "live";
    } else {
      startRealtimeStream();
    }
  }

  function clearRealtimeLines() {
    realtimeLines.value = [];
  }

  return {
    archives,
    loadingArchives,
    syncing,
    selectedArchive,
    logContent,
    loadingContent,
    realtimeLines,
    filteredLines,
    connectionStatus,
    autoScroll,
    selectedLevel,
    searchFilter,
    notification,
    fetchArchives,
    syncLogs,
    fetchContent,
    selectArchive,
    clearSelectedArchive,
    startRealtimeStream,
    stopRealtimeStream,
    togglePause,
    clearRealtimeLines,
    showNotification,
  };
}
