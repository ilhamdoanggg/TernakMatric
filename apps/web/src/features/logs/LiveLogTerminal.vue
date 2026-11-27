<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { useLogStore } from "@/stores/logStore";

const logStore = useLogStore();
const terminalContainer = ref<HTMLElement | null>(null);

function scrollToBottom() {
  nextTick(() => {
    if (terminalContainer.value) {
      terminalContainer.value.scrollTop = terminalContainer.value.scrollHeight;
    }
  });
}

watch(
  () => logStore.realtimeLines.value.length,
  () => {
    if (logStore.autoScroll.value) {
      scrollToBottom();
    }
  },
);

onMounted(() => {
  logStore.startRealtimeStream();
});

onUnmounted(() => {
  logStore.stopRealtimeStream();
});
</script>

<template>
  <div class="rounded-xl border border-slate-800 bg-slate-900 shadow-xl overflow-hidden flex flex-col">
    <!-- Top toolbar -->
    <div class="flex flex-wrap items-center justify-between border-b border-slate-800 bg-slate-950/70 px-4 py-3 gap-3">
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2">
          <span
            class="h-2.5 w-2.5 rounded-full"
            :class="{
              'bg-emerald-400 animate-pulse': logStore.connectionStatus.value === 'live',
              'bg-amber-400': logStore.connectionStatus.value === 'paused',
              'bg-sky-400 animate-ping': logStore.connectionStatus.value === 'connecting',
              'bg-slate-500': logStore.connectionStatus.value === 'disconnected'
            }"
          />
          <span class="text-xs font-semibold uppercase tracking-wider text-slate-300">
            Realtime Stream
          </span>
          <span
            class="rounded px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider"
            :class="{
              'bg-emerald-950 text-emerald-400 border border-emerald-800/40': logStore.connectionStatus.value === 'live',
              'bg-amber-950 text-amber-300 border border-amber-800/40': logStore.connectionStatus.value === 'paused',
              'bg-sky-950 text-sky-300 border border-sky-800/40': logStore.connectionStatus.value === 'connecting',
              'bg-slate-800 text-slate-400': logStore.connectionStatus.value === 'disconnected'
            }"
          >
            {{ logStore.connectionStatus.value }}
          </span>
        </div>

        <span class="text-xs text-slate-500 hidden sm:inline">|</span>
        <span class="text-xs text-slate-400 hidden sm:inline">
          {{ logStore.filteredLines.value.length }} baris termuat (max 500)
        </span>
      </div>

      <!-- Action buttons & filters -->
      <div class="flex flex-wrap items-center gap-2">
        <!-- Search filter -->
        <input
          v-model="logStore.searchFilter.value"
          type="text"
          placeholder="Cari teks log..."
          class="rounded-lg border border-slate-800 bg-slate-900 px-2.5 py-1 text-xs text-slate-200 placeholder-slate-500 focus:border-emerald-500 focus:outline-none w-36 sm:w-44"
        />

        <!-- Level selector -->
        <select
          v-model="logStore.selectedLevel.value"
          class="rounded-lg border border-slate-800 bg-slate-900 px-2.5 py-1 text-xs text-slate-200 focus:border-emerald-500 focus:outline-none"
        >
          <option value="ALL">Semua Level</option>
          <option value="INFO">INFO</option>
          <option value="WARN">WARN</option>
          <option value="ERROR">ERROR</option>
          <option value="DEBUG">DEBUG</option>
        </select>

        <!-- Auto-scroll toggle -->
        <button
          class="rounded-lg border px-2.5 py-1 text-xs font-medium transition-colors"
          :class="logStore.autoScroll.value ? 'border-emerald-700/50 bg-emerald-950/60 text-emerald-300' : 'border-slate-800 bg-slate-900 text-slate-400 hover:text-slate-200'"
          @click="logStore.autoScroll.value = !logStore.autoScroll.value"
        >
          Auto-scroll: {{ logStore.autoScroll.value ? "ON" : "OFF" }}
        </button>

        <!-- Pause / Resume -->
        <button
          class="rounded-lg border border-slate-700 bg-slate-800 px-2.5 py-1 text-xs font-medium text-slate-200 hover:bg-slate-700 transition-colors"
          @click="logStore.togglePause"
        >
          {{ logStore.connectionStatus.value === 'live' ? 'Jeda' : logStore.connectionStatus.value === 'paused' ? 'Lanjutkan' : 'Sambungkan' }}
        </button>

        <!-- Clear -->
        <button
          class="rounded-lg border border-slate-800 bg-slate-900 px-2.5 py-1 text-xs font-medium text-slate-400 hover:bg-slate-800 hover:text-slate-200 transition-colors"
          @click="logStore.clearRealtimeLines"
        >
          Bersihkan
        </button>
      </div>
    </div>

    <!-- Terminal screen -->
    <div
      ref="terminalContainer"
      class="h-80 overflow-y-auto bg-slate-950 p-4 font-mono text-xs leading-relaxed select-text"
    >
      <div v-if="logStore.filteredLines.value.length === 0" class="flex h-full items-center justify-center text-slate-600">
        <span v-if="logStore.connectionStatus.value === 'connecting'">Menyambungkan ke aliran log realtime...</span>
        <span v-else-if="logStore.connectionStatus.value === 'live'">Menunggu aktivitas sistem terbaru...</span>
        <span v-else>Tidak ada baris log yang cocok dengan filter saat ini.</span>
      </div>

      <div v-else class="space-y-1">
        <div
          v-for="line in logStore.filteredLines.value"
          :key="line.id"
          class="flex items-start gap-2 hover:bg-slate-900/60 px-1 py-0.5 rounded transition-colors"
        >
          <!-- Level badge -->
          <span
            class="shrink-0 rounded px-1.5 py-0.2 text-[10px] font-bold uppercase tracking-wider"
            :class="{
              'bg-emerald-950 text-emerald-400 border border-emerald-800/50': line.level === 'INFO',
              'bg-amber-950 text-amber-300 border border-amber-800/50': line.level === 'WARN',
              'bg-red-950 text-red-400 border border-red-800/50': line.level === 'ERROR',
              'bg-cyan-950 text-cyan-300 border border-cyan-800/50': line.level === 'DEBUG',
              'bg-slate-800 text-slate-400': line.level === 'OTHER'
            }"
          >
            {{ line.level }}
          </span>

          <!-- Timestamp if present -->
          <span v-if="line.timestamp" class="shrink-0 text-slate-500 text-[11px]">
            {{ line.timestamp }}
          </span>

          <!-- Message -->
          <span
            class="whitespace-pre-wrap break-all text-slate-300 flex-1"
            :class="{
              'text-red-300 font-semibold': line.level === 'ERROR',
              'text-amber-200': line.level === 'WARN',
              'text-emerald-100': line.level === 'INFO',
            }"
          >
            {{ line.message }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
