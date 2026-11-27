<script setup lang="ts">
import { onMounted, ref } from "vue";
import { RouterLink } from "vue-router";
import { useAuthStore } from "@/stores/authStore";
import { useLogStore } from "@/stores/logStore";
import type { SystemLogArchiveDto } from "@/types/logs";
import LiveLogTerminal from "@/features/logs/LiveLogTerminal.vue";
import LogArchiveList from "@/features/logs/LogArchiveList.vue";
import LogContentModal from "@/features/logs/LogContentModal.vue";

const authStore = useAuthStore();
const logStore = useLogStore();

const isModalOpen = ref(false);

function handleView(archive: SystemLogArchiveDto) {
  isModalOpen.value = true;
  logStore.selectArchive(archive);
}

function handleChangeLimit(limit: number, tail: boolean) {
  if (logStore.selectedArchive.value) {
    logStore.fetchContent(logStore.selectedArchive.value.file_name, limit, tail);
  }
}

function closeModal() {
  isModalOpen.value = false;
  logStore.clearSelectedArchive();
}

onMounted(() => {
  logStore.fetchArchives();
});
</script>

<template>
  <main class="mx-auto max-w-7xl px-6 py-10">
    <!-- Breadcrumb & Top Bar -->
    <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
      <nav class="flex items-center gap-2 text-xs text-slate-400">
        <RouterLink to="/" class="hover:text-emerald-400 transition-colors">
          Dashboard
        </RouterLink>
        <span>/</span>
        <span class="text-slate-200 font-semibold">System Logs</span>
      </nav>

      <RouterLink
        to="/"
        class="rounded-lg border border-slate-800 bg-slate-900 px-3 py-1.5 text-xs font-semibold text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
      >
        &larr; Kembali ke Dashboard
      </RouterLink>
    </div>

    <!-- Page Header -->
    <header class="mb-8 flex flex-col sm:flex-row sm:items-end sm:justify-between gap-4">
      <div>
        <p class="text-xs font-semibold uppercase tracking-widest text-emerald-400">
          Operasional Sistem &amp; Audit Log
        </p>
        <h1 class="text-3xl font-bold tracking-tight text-white mt-1">
          Pusat Log Sistem
        </h1>
        <p class="mt-2 text-sm text-slate-400 max-w-3xl">
          Pemantauan real-time stream log server dan arsip database rolling harian berformat
          <code class="text-emerald-400 font-mono">tanggal-bulan-tahun.log</code> dengan lokasi berkas kustom.
        </p>
      </div>

      <div class="flex items-center gap-2">
        <span class="rounded-full border border-slate-800 bg-slate-900 px-3.5 py-1.5 text-xs text-slate-300">
          Akses:
          <strong
            class="font-medium"
            :class="authStore.canManageOrg.value ? 'text-emerald-400' : 'text-slate-400'"
          >
            {{ authStore.canManageOrg.value ? "ORG_OWNER / ORG_ADMIN" : "Viewer" }}
          </strong>
        </span>
      </div>
    </header>

    <!-- Notification Toast -->
    <div
      v-if="logStore.notification.value"
      class="mb-6 rounded-lg px-4 py-3 text-xs font-medium transition-all"
      :class="logStore.notification.value.type === 'success' ? 'bg-emerald-950/80 border border-emerald-800 text-emerald-300' : 'bg-red-950/80 border border-red-800 text-red-300'"
    >
      {{ logStore.notification.value.message }}
    </div>

    <!-- Section 1: Realtime Log Terminal -->
    <section class="mb-10">
      <div class="mb-3 flex items-center justify-between">
        <h2 class="text-sm font-bold uppercase tracking-wider text-slate-300">
          1. Aliran Log Realtime (Live Stream)
        </h2>
        <span class="text-xs text-slate-500">
          Server-Sent Events (SSE) via /api/v1/system/logs/stream
        </span>
      </div>

      <LiveLogTerminal />
    </section>

    <!-- Section 2: Daily Log Archives (Database + File System) -->
    <section>
      <div class="mb-3 flex items-center justify-between">
        <h2 class="text-sm font-bold uppercase tracking-wider text-slate-300">
          2. Daftar Arsip Berkas Log Harian
        </h2>
        <span class="text-xs text-slate-500">
          Tersimpan lokal &amp; tercatat otomatis di PostgreSQL
        </span>
      </div>

      <LogArchiveList
        :archives="logStore.archives.value"
        :loading="logStore.loadingArchives.value"
        :syncing="logStore.syncing.value"
        @view="handleView"
        @sync="logStore.syncLogs"
        @refresh="logStore.fetchArchives"
      />
    </section>

    <!-- Modal Log Content Viewer -->
    <LogContentModal
      :is-open="isModalOpen"
      :content="logStore.logContent.value"
      :loading="logStore.loadingContent.value"
      @close="closeModal"
      @change-limit="handleChangeLimit"
    />
  </main>
</template>
