export type SpeciesCategory =
  | "fish"
  | "crustacean"
  | "mollusk"
  | "poultry"
  | "mammal"
  | "reptile"
  | "insect"
  | "other";

export type OutputType =
  | "meat"
  | "egg"
  | "milk"
  | "fry"
  | "cull"
  | "offspring"
  | "other";

export type SyncOperationType = "create" | "update" | "delete";

export interface SyncCommand {
  client_operation_id: string;
  entity_type: string;
  entity_id: string;
  operation_type: SyncOperationType;
  base_version?: number | null;
  server_current_version?: number | null;
  device_occurred_at: string;
  payload: Record<string, unknown>;
}

export interface SyncPushRequest {
  device_id: string;
  commands: SyncCommand[];
}

export interface CalculationAssumptions {
  is_estimate: boolean;
  last_sample_at?: string | null;
  notes: string[];
}

export interface CalculationResult<T> {
  value: T;
  unit: string;
  formula_code: string;
  formula_version: number;
  parameter_version?: number | null;
  calculated_at: string;
  assumptions: CalculationAssumptions;
}

export interface ClientConfig {
  baseUrl: string;
  getToken?: () => Promise<string | null> | string | null;
  getOrganizationId?: () => Promise<string | null> | string | null;
}

export class TernakMatricApiClient {
  constructor(private readonly config: ClientConfig) {}

  async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const headers = new Headers(init.headers);
    headers.set("Content-Type", "application/json");

    const token = await this.config.getToken?.();
    if (token) {
      headers.set("Authorization", `Bearer ${token}`);
    }
    const orgId = await this.config.getOrganizationId?.();
    if (orgId) {
      headers.set("X-Organization-Id", orgId);
    }

    const res = await fetch(`${this.config.baseUrl}${path}`, {
      ...init,
      headers,
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`API ${res.status}: ${text}`);
    }
    return (await res.json()) as T;
  }
}
