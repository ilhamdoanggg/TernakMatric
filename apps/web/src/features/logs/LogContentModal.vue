<script setup lang="ts">
import type { SystemLogContentDto } from "@/types/logs";
import { ref } from "vue";

const props = defineProps<{
  isOpen: boolean;
  content: SystemLogContentDto | null;
  loading: boolean;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "changeLimit", limit: number, tail: boolean): void;
}>();

const copied = ref(false);

function copyContent() {
  if (!props.content) return;
  const text = props.content.lines.join("\n");
  if (typeof navigator !== "undefined" && navigator.clipboard) {
    navigator.clipboard.writeText(text).then(() => {
      copied.value = true;
      setTimeout(() => {
        copied.value = false;
      }, 2000);
    });
  }
}
</script>

<template>
  <div
    v-if="isOpen"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/75 p-4 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <div
      class="flex max-h-[90vh] w-full max-w-5xl flex-col rounded-xl border border-slate-800 bg-slate-900 shadow-2xl"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-slate-800 px-6 py-4">
        <div>
          <div class="flex items-center gap-3">
            <h2 class="text-lg font-bold text-white">
              {{ content?.file_name ?? "Memuat Log..." }}
            </h2>
            <span
              v-if="content"
              class="rounded bg-slate-800 px-2.5 py-0.5 text-xs font-medium text-emerald-400"
            >
              {{ content.total_lines }} total baris
            </span>
          </div>
          <p v-if="content" class="mt-0.5 font-mono text-xs text-slate-400 truncate max-w-2xl">
            {{ content.file_path }}
          </p>
        </div>

        <div class="flex items-center gap-2">
          <!-- Limit selector -->
          <div class="flex items-center gap-1 bg-slate-950 rounded-lg p-1 border border-slate-800 text-xs">
            <button
              class="rounded px-2 py-1 text-slate-300 hover:text-white transition-colors"
              @click="emit('changeLimit', 200, true)"
            >
              Akhir 200
            </button>
            <button
              class="rounded px-2 py-1 text-slate-300 hover:text-white transition-colors"
              @click="emit('changeLimit', 500, true)"
            >
              Akhir 500
            </button>
            <button
              class="rounded px-2 py-1 text-slate-300 hover:text-white transition-colors"
              @click="emit('changeLimit', 1000, true)"
            >
              Akhir 1000
            </button>
          </div>

          <button
            class="rounded-lg border border-slate-700 bg-slate-800 px-3 py-1.5 text-xs font-semibold text-slate-200 hover:bg-slate-700 transition-colors"
            @click="copyContent"
          >
            {{ copied ? "Tersalin!" : "Salin Log" }}
          </button>
          <button
            class="rounded-lg p-1.5 text-slate-400 hover:bg-slate-800 hover:text-white transition-colors"
            @click="emit('close')"
          >
            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Truncated Banner -->
      <div
        v-if="content?.is_truncated"
        class="bg-amber-950/40 border-b border-amber-800/40 px-6 py-2 text-xs text-amber-300 flex items-center justify-between"
      >
        <span>
          Menampilkan {{ content.returned_lines }} baris terakhir dari total {{ content.total_lines }} baris.
        </span>
      </div>

      <!-- Log Content Window -->
      <div class="flex-1 overflow-auto bg-slate-950 p-4 font-mono text-xs">
        <div v-if="loading" class="py-12 text-center text-slate-400">
          Memuat isi berkas log...
        </div>
        <div v-else-if="!content?.lines || content.lines.length === 0" class="py-12 text-center text-slate-500">
          Berkas log kosong atau tidak ada data.
        </div>
        <div v-else class="space-y-0.5">
          <div
            v-for="(line, idx) in content.lines"
            :key="idx"
            class="flex items-start hover:bg-slate-900/60 py-0.5 px-1 rounded transition-colors"
          >
            <span class="w-12 shrink-0 select-none text-right pr-4 text-slate-600 font-mono text-[11px]">
              {{ idx + 1 }}
            </span>
            <span
              class="whitespace-pre-wrap break-all"
              :class="{
                'text-red-400': line.includes('ERROR'),
                'text-amber-300': line.includes('WARN'),
                'text-emerald-400': line.includes('INFO'),
                'text-cyan-300': line.includes('DEBUG'),
                'text-slate-300': !line.includes('ERROR') && !line.includes('WARN') && !line.includes('INFO') && !line.includes('DEBUG')
              }"
            >
              {{ line }}
            </span>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="border-t border-slate-800 px-6 py-3 flex items-center justify-between text-xs text-slate-500">
        <span>Tekan tombol di luar atau Esc untuk menutup.</span>
        <button
          class="rounded-lg bg-slate-800 px-4 py-1.5 font-medium text-slate-200 hover:bg-slate-700 transition-colors"
          @click="emit('close')"
        >
          Tutup
        </button>
      </div>
    </div>
  </div>
</template>
