<script setup lang="ts">
import type { SystemLogArchiveDto } from "@/types/logs";

defineProps<{
  archives: SystemLogArchiveDto[];
  loading: boolean;
  syncing: boolean;
}>();

const emit = defineEmits<{
  (e: "view", archive: SystemLogArchiveDto): void;
  (e: "sync"): void;
  (e: "refresh"): void;
}>();

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
}

function formatNumber(val: number): string {
  return new Intl.NumberFormat("id-ID").format(val);
}

function formatDate(isoStr: string): string {
  try {
    const d = new Date(isoStr);
    return new Intl.DateTimeFormat("id-ID", {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(d);
  } catch {
    return isoStr;
  }
}
</script>

<template>
  <div class="rounded-xl border border-slate-800 bg-slate-900 shadow-xl overflow-hidden">
    <!-- Header Toolbar -->
    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between border-b border-slate-800 bg-slate-950/40 px-6 py-4 gap-3">
      <div>
        <h2 class="text-base font-bold text-white flex items-center gap-2">
          <span>Arsip Log Harian di Database</span>
          <span class="rounded-full bg-slate-800 px-2.5 py-0.5 text-xs font-semibold text-emerald-400">
            {{ archives.length }} berkas
          </span>
        </h2>
        <p class="text-xs text-slate-400 mt-0.5">
          Tercatat di tabel <code class="text-emerald-400">system_log_archives</code> dengan rotasi harian <code class="text-emerald-400">DD-MM-YYYY.log</code>.
        </p>
      </div>

      <div class="flex items-center gap-2">
        <button
          class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-xs font-medium text-slate-200 hover:bg-slate-700 disabled:opacity-50 transition-colors"
          :disabled="loading"
          @click="emit('refresh')"
        >
          {{ loading ? "Memuat..." : "Segarkan List" }}
        </button>

        <button
          class="rounded-lg bg-emerald-600 px-3.5 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-emerald-500 disabled:opacity-50 transition-colors flex items-center gap-1.5"
          :disabled="syncing"
          @click="emit('sync')"
        >
          <svg
            v-if="syncing"
            class="animate-spin -ml-0.5 mr-1.5 h-3.5 w-3.5 text-white"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
          </svg>
          <span>{{ syncing ? "Menyinkronkan..." : "Sinkronkan Sekarang" }}</span>
        </button>
      </div>
    </div>

    <!-- Table content -->
    <div class="overflow-x-auto">
      <table class="w-full text-left text-xs">
        <thead class="border-b border-slate-800 bg-slate-950/60 font-medium uppercase tracking-wider text-slate-400">
          <tr>
            <th scope="col" class="px-6 py-3">Tanggal Log</th>
            <th scope="col" class="px-6 py-3">Service</th>
            <th scope="col" class="px-6 py-3">Nama Berkas</th>
            <th scope="col" class="px-6 py-3">Lokasi Path</th>
            <th scope="col" class="px-6 py-3 text-right">Ukuran</th>
            <th scope="col" class="px-6 py-3 text-right">Baris</th>
            <th scope="col" class="px-6 py-3">Status</th>
            <th scope="col" class="px-6 py-3">Waktu Arsip</th>
            <th scope="col" class="px-6 py-3 text-center">Aksi</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-normal text-slate-300">
          <tr v-if="loading && archives.length === 0">
            <td colspan="9" class="px-6 py-12 text-center text-slate-500">
              Memuat arsip log...
            </td>
          </tr>
          <tr v-else-if="archives.length === 0">
            <td colspan="9" class="px-6 py-12 text-center text-slate-500">
              Belum ada berkas log yang diarsipkan di database. Klik tombol "Sinkronkan Sekarang" di atas untuk memindai berkas log lokal.
            </td>
          </tr>
          <tr
            v-for="log in archives"
            :key="log.id"
            class="hover:bg-slate-800/40 transition-colors"
          >
            <td class="whitespace-nowrap px-6 py-3 font-semibold text-white">
              {{ log.log_date }}
            </td>
            <td class="whitespace-nowrap px-6 py-3">
              <span class="rounded bg-slate-800 px-2 py-0.5 text-[11px] font-medium text-slate-300">
                {{ log.service_name }}
              </span>
            </td>
            <td class="whitespace-nowrap px-6 py-3 font-mono font-medium text-emerald-400">
              {{ log.file_name }}
            </td>
            <td class="px-6 py-3 font-mono text-[11px] text-slate-400 max-w-xs truncate" :title="log.file_path">
              {{ log.file_path }}
            </td>
            <td class="whitespace-nowrap px-6 py-3 text-right font-mono text-slate-300">
              {{ formatBytes(log.file_size_bytes) }}
            </td>
            <td class="whitespace-nowrap px-6 py-3 text-right font-mono text-slate-300">
              {{ formatNumber(log.line_count) }}
            </td>
            <td class="whitespace-nowrap px-6 py-3">
              <span
                class="rounded px-2 py-0.5 text-[11px] font-bold uppercase tracking-wider"
                :class="log.status === 'active' ? 'bg-emerald-950 text-emerald-400 border border-emerald-800/50' : 'bg-slate-800 text-slate-400'"
              >
                {{ log.status }}
              </span>
            </td>
            <td class="whitespace-nowrap px-6 py-3 text-slate-400 text-[11px]">
              {{ formatDate(log.archived_at) }}
            </td>
            <td class="whitespace-nowrap px-6 py-3 text-center">
              <button
                class="rounded-lg bg-emerald-950/60 border border-emerald-800/60 px-3 py-1 font-medium text-emerald-400 hover:bg-emerald-900/80 hover:text-emerald-300 transition-colors"
                @click="emit('view', log)"
              >
                Lihat Isi
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
