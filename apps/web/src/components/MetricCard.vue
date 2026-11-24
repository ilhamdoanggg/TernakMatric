<script setup lang="ts">
defineProps<{
  label: string;
  value: string;
  hint: string;
}>();
</script>

<template>
  <div class="rounded-xl border border-slate-800 bg-slate-900/70 p-5">
    <p class="text-xs font-medium uppercase tracking-wide text-slate-400">
      {{ label }}
    </p>
    <p class="mt-2 text-2xl font-semibold text-white">{{ value }}</p>
    <p class="mt-1 text-xs text-slate-400">{{ hint }}</p>
  </div>
</template>
