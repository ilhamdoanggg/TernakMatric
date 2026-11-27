<script setup lang="ts">
import { RouterLink } from "vue-router";
import MetricCard from "@/components/MetricCard.vue";
import SpeciesOverviewTable from "@/features/dashboard/SpeciesOverviewTable.vue";
</script>

<template>
  <main class="mx-auto max-w-6xl px-6 py-10">
    <header class="mb-8 flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between">
      <div>
        <p class="text-xs font-semibold uppercase tracking-widest text-emerald-400">
          TernakMatric · Multi-Species SaaS
        </p>
        <h1 class="text-3xl font-bold tracking-tight text-white">
          Dashboard Produksi &amp; Analitik
        </h1>
      </div>
      <div class="flex items-center gap-3">
        <RouterLink
          to="/logs"
          class="rounded-lg border border-slate-700 bg-slate-800 px-3.5 py-1.5 text-xs font-semibold text-emerald-400 hover:bg-slate-700 hover:text-emerald-300 transition-colors flex items-center gap-1.5"
        >
          <span class="inline-block h-2 w-2 rounded-full bg-emerald-400 animate-pulse"></span>
          System Logs &rarr;
        </RouterLink>
        <span class="rounded-full border border-slate-800 bg-slate-900 px-3 py-1 text-xs text-slate-300">
          Vue 3 SSR + Calculation Engine v1 (FCR &middot; ADG &middot; SGR &middot; HPP)
        </span>
      </div>
    </header>

    <section class="mb-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      <MetricCard
        label="Estimasi Biomassa Aktif"
        value="14.280 kg"
        hint="Gabungan seluruh cohort aktif (estimasi sampling)"
      />
      <MetricCard
        label="Rata-rata FCR Ekonomis"
        value="1.42"
        hint="Ternormalisasi kg pakan / kg pertambahan"
      />
      <MetricCard
        label="HPP / Kg Hidup (Avg)"
        value="Rp 21.450"
        hint="Bibit + Pakan + Kesehatan + Operasional"
      />
      <MetricCard
        label="Vaksin Jatuh Tempo"
        value="3 Jadwal"
        hint="Dipantau oleh background worker"
      />
    </section>

    <SpeciesOverviewTable />
  </main>
</template>
