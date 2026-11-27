<script setup lang="ts">
import type { CohortPreviewRow } from "@/types/dashboard";

const sampleCohorts: CohortPreviewRow[] = [
  {
    batchCode: "B-NILA-2611",
    species: "Nila (Oreochromis niloticus)",
    category: "fish",
    enclosure: "Kolam Beton K-01",
    population: 4850,
    growthMetric: "SGR 3.12%/hari",
    fcr: "1.28",
    hppPerKg: "Rp 18.200",
  },
  {
    batchCode: "B-BROILER-2611",
    species: "Ayam Broiler",
    category: "poultry",
    enclosure: "Kandang Closed House C-02",
    population: 9820,
    growthMetric: "ADG 61.5 g/hari",
    fcr: "1.51",
    hppPerKg: "Rp 20.900",
  },
  {
    batchCode: "B-KAMBING-2610",
    species: "Kambing Boer Cross",
    category: "mammal",
    enclosure: "Pen Paddock P-03",
    population: 64,
    growthMetric: "ADG 145 g/hari",
    fcr: "4.10",
    hppPerKg: "Rp 42.300",
  },
];
</script>

<template>
  <section class="overflow-hidden rounded-xl border border-slate-800 bg-slate-900/70">
    <div class="border-b border-slate-800 px-5 py-4">
      <h2 class="text-base font-semibold text-white">
        Cohort / Batch Aktif Lintas Spesies
      </h2>
    </div>
    <div class="overflow-x-auto">
      <table class="w-full text-left text-sm">
        <thead class="border-b border-slate-800 text-xs uppercase text-slate-400">
          <tr>
            <th class="px-5 py-3">Batch</th>
            <th class="px-5 py-3">Spesies</th>
            <th class="px-5 py-3">Enclosure</th>
            <th class="px-5 py-3">Populasi</th>
            <th class="px-5 py-3">Pertumbuhan</th>
            <th class="px-5 py-3">FCR</th>
            <th class="px-5 py-3">HPP / kg</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/70">
          <tr
            v-for="row in sampleCohorts"
            :key="row.batchCode"
            class="hover:bg-slate-800/40"
          >
            <td class="px-5 py-3 font-mono text-emerald-300">
              {{ row.batchCode }}
            </td>
            <td class="px-5 py-3">{{ row.species }}</td>
            <td class="px-5 py-3 text-slate-300">{{ row.enclosure }}</td>
            <td class="px-5 py-3">{{ row.population.toLocaleString("id-ID") }}</td>
            <td class="px-5 py-3">{{ row.growthMetric }}</td>
            <td class="px-5 py-3">{{ row.fcr }}</td>
            <td class="px-5 py-3 font-medium text-white">{{ row.hppPerKg }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>
