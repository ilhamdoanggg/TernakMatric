import type { SpeciesCategory } from "@ternakmatric/api-contracts";

export interface CohortPreviewRow {
  batchCode: string;
  species: string;
  category: SpeciesCategory;
  enclosure: string;
  population: number;
  growthMetric: string;
  fcr: string;
  hppPerKg: string;
}

export interface MetricCardItem {
  label: string;
  value: string;
  hint?: string;
}
