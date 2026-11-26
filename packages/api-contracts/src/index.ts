import type {
  components,
  operations,
  paths,
  webhooks,
} from "../generated/typescript/schema";

export type { components, operations, paths, webhooks };

export type Schemas = components["schemas"];

// Enums & primitive aliases
export type DeviceId = Schemas["DeviceId"];
export type SpeciesCategory = Schemas["SpeciesCategory"];
export type OutputType = Schemas["OutputType"];
export type MovementType = Schemas["MovementType"];
export type FcrMode = Schemas["FcrMode"];
export type SyncOperationType = Schemas["SyncOperationType"];
export type SyncStatus = Schemas["SyncStatus"];

// Calculation wrapper & results
export type CalculationAssumptions = Schemas["CalculationAssumptions"];
export type CalculationResultBatchPnlSummary =
  Schemas["CalculationResult_BatchPnlSummary"];
export type CalculationResultCapacityEvaluation =
  Schemas["CalculationResult_CapacityEvaluation"];
export type CalculationResultFcrSummary =
  Schemas["CalculationResult_FcrSummary"];
export type CalculationResultString = Schemas["CalculationResult_String"];

export interface CalculationResult<T> {
  value: T;
  unit: string;
  formula_code: string;
  formula_version: number;
  parameter_version?: number | null;
  calculated_at: string;
  assumptions: CalculationAssumptions;
}

// System & Error
export type HealthResponse = Schemas["HealthResponse"];
export type ErrorEnvelope = Schemas["ErrorEnvelope"];

// Tenant & Auth
export type RegisterRequest = Schemas["RegisterRequest"];
export type LoginRequest = Schemas["LoginRequest"];
export type AuthTokenResponse = Schemas["AuthTokenResponse"];
export type CreateOrganizationRequest = Schemas["CreateOrganizationRequest"];
export type OrganizationDto = Schemas["OrganizationDto"];
export type InviteMemberRequest = Schemas["InviteMemberRequest"];
export type MemberDto = Schemas["MemberDto"];
export type CreateFarmRequest = Schemas["CreateFarmRequest"];
export type UpdateFarmRequest = Schemas["UpdateFarmRequest"];
export type FarmDto = Schemas["FarmDto"];
export type AuditLogDto = Schemas["AuditLogDto"];

// Enclosure
export type UnitTypeDto = Schemas["UnitTypeDto"];
export type ProductionSystemDto = Schemas["ProductionSystemDto"];
export type EnclosureGeometry = Schemas["EnclosureGeometry"];
export type CreateEnclosureRequest = Schemas["CreateEnclosureRequest"];
export type UpdateEnclosureRequest = Schemas["UpdateEnclosureRequest"];
export type EnclosureDto = Schemas["EnclosureDto"];
export type CalculateCapacityRequest = Schemas["CalculateCapacityRequest"];
export type CapacityEvaluation = Schemas["CapacityEvaluation"];

// Species
export type CreateSpeciesRequest = Schemas["CreateSpeciesRequest"];
export type SpeciesDto = Schemas["SpeciesDto"];
export type CreateProfileRequest = Schemas["CreateProfileRequest"];
export type SpeciesProfileDto = Schemas["SpeciesProfileDto"];

// Production
export type CreateBatchRequest = Schemas["CreateBatchRequest"];
export type UpdateBatchRequest = Schemas["UpdateBatchRequest"];
export type BatchDto = Schemas["BatchDto"];
export type BatchDetailDto = Schemas["BatchDetailDto"];
export type StockMovementRequest = Schemas["StockMovementRequest"];
export type StockMovementResponse = Schemas["StockMovementResponse"];
export type RegisterTagRequest = Schemas["RegisterTagRequest"];
export type IndividualTagDto = Schemas["IndividualTagDto"];
export type GrowthSampleDto = Schemas["GrowthSampleDto"];
export type CreateGrowthSampleRequest = Schemas["CreateGrowthSampleRequest"];
export type GrowthSampleResponse = Schemas["GrowthSampleResponse"];
export type ProductionOutputDto = Schemas["ProductionOutputDto"];
export type CreateOutputRequest = Schemas["CreateOutputRequest"];
export type OutputResponse = Schemas["OutputResponse"];

// Feeding & Inventory
export type CreateWarehouseRequest = Schemas["CreateWarehouseRequest"];
export type WarehouseDto = Schemas["WarehouseDto"];
export type CreateFeedProductRequest = Schemas["CreateFeedProductRequest"];
export type FeedProductDto = Schemas["FeedProductDto"];
export type InventoryBalanceDto = Schemas["InventoryBalanceDto"];
export type InventoryLineInput = Schemas["InventoryLineInput"];
export type CreateInventoryTxRequest = Schemas["CreateInventoryTxRequest"];
export type InventoryTxResponse = Schemas["InventoryTxResponse"];
export type RecordFeedingRequest = Schemas["RecordFeedingRequest"];
export type FeedingRecordResponse = Schemas["FeedingRecordResponse"];
export type FcrSummary = Schemas["FcrSummary"];

// Environment
export type EnvironmentMetricDto = Schemas["EnvironmentMetricDto"];
export type EnvironmentReadingDto = Schemas["EnvironmentReadingDto"];
export type CreateReadingRequest = Schemas["CreateReadingRequest"];
export type IotIngestRequest = Schemas["IotIngestRequest"];

// Health & Biosecurity
export type CreateVaccinationScheduleRequest =
  Schemas["CreateVaccinationScheduleRequest"];
export type UpdateVaccinationStatusRequest =
  Schemas["UpdateVaccinationStatusRequest"];
export type VaccinationScheduleDto = Schemas["VaccinationScheduleDto"];
export type CreateHealthRecordRequest = Schemas["CreateHealthRecordRequest"];
export type HealthRecordDto = Schemas["HealthRecordDto"];
export type MortalityRecordDto = Schemas["MortalityRecordDto"];
export type RecordMortalityRequest = Schemas["RecordMortalityRequest"];
export type MortalityResponse = Schemas["MortalityResponse"];
export type CreateBiosecurityLogRequest =
  Schemas["CreateBiosecurityLogRequest"];
export type BiosecurityLogDto = Schemas["BiosecurityLogDto"];

// Finance
export type ExpenseCategoryDto = Schemas["ExpenseCategoryDto"];
export type CreateExpenseRequest = Schemas["CreateExpenseRequest"];
export type ExpenseDto = Schemas["ExpenseDto"];
export type SaleItemInput = Schemas["SaleItemInput"];
export type SaleItemDto = Schemas["SaleItemDto"];
export type SaleDto = Schemas["SaleDto"];
export type CreateSaleRequest = Schemas["CreateSaleRequest"];
export type SaleResponse = Schemas["SaleResponse"];
export type CostBreakdown = Schemas["CostBreakdown"];
export type BatchPnlSummary = Schemas["BatchPnlSummary"];
export type BatchFinancialSummaryResponse =
  Schemas["BatchFinancialSummaryResponse"];

// Offline Sync
export type RegisterDeviceRequest = Schemas["RegisterDeviceRequest"];
export type RegisterDeviceResponse = Schemas["RegisterDeviceResponse"];
export type SyncCommand = Schemas["SyncCommand"];
export type SyncPushRequest = Schemas["SyncPushRequest"];
export type SyncPushItemResult = Schemas["SyncPushItemResult"];
export type SyncPushResponse = Schemas["SyncPushResponse"];
export type OutboxChangeEvent = Schemas["OutboxChangeEvent"];
export type SyncPullResponse = Schemas["SyncPullResponse"];
export type SyncConflictRecord = Schemas["SyncConflictRecord"];

// Query parameter contracts accepted by backend handlers
export interface PaginationQuery {
  limit?: number;
}

export interface AuditLogQuery {
  entity_type?: string;
  entity_id?: string;
  limit?: number;
}

export interface SpeciesFilterQuery {
  category?: SpeciesCategory;
}

export interface BatchSummaryQuery {
  status?: string;
  limit?: number;
}

export interface InventoryBalanceQuery {
  warehouse_id?: string;
  feed_product_id?: string;
  limit?: number;
}

export interface FcrQuery {
  mode?: FcrMode;
}

export interface ExpenseQuery {
  batch_id?: string;
  farm_id?: string;
  category_id?: string;
  limit?: number;
}

export interface SaleQuery {
  batch_id?: string;
  limit?: number;
}

export interface SyncPullQuery {
  cursor?: number;
  limit?: number;
}

export interface ClientConfig {
  baseUrl: string;
  getToken?: () => Promise<string | null> | string | null;
  getOrganizationId?: () => Promise<string | null> | string | null;
}

/** Parses a backend JSON error body matching `ErrorEnvelope` (`code`, `message`, `details`). */
export function parseErrorEnvelope(rawBody: string): ErrorEnvelope | null {
  if (!rawBody.trim()) {
    return null;
  }
  try {
    const parsed: unknown = JSON.parse(rawBody);
    if (
      typeof parsed === "object" &&
      parsed !== null &&
      "code" in parsed &&
      "message" in parsed &&
      typeof parsed.code === "string" &&
      typeof parsed.message === "string"
    ) {
      return {
        code: parsed.code,
        message: parsed.message,
        details: "details" in parsed ? parsed.details : undefined,
      };
    }
  } catch {
    // Non-JSON responses fall back to rawBody inside ApiClientError.
  }
  return null;
}

/** Structured error thrown by `TernakMatricApiClient` when the API returns a non-2xx status. */
export class ApiClientError extends Error {
  readonly status: number;
  readonly code: string;
  readonly details?: unknown;
  readonly rawBody: string;

  constructor(
    status: number,
    envelope: ErrorEnvelope | null,
    rawBody: string,
  ) {
    const detailText = envelope?.message ?? (rawBody || `HTTP ${status}`);
    super(`API ${status} (${envelope?.code ?? "HTTP_ERROR"}): ${detailText}`);
    this.name = "ApiClientError";
    this.status = status;
    this.code = envelope?.code ?? "HTTP_ERROR";
    this.details = envelope?.details;
    this.rawBody = rawBody;
  }
}

function buildQueryString(
  params?: Record<string, string | number | undefined | null>,
): string {
  if (!params) {
    return "";
  }
  const search = new URLSearchParams();
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null) {
      search.set(key, String(value));
    }
  }
  const qs = search.toString();
  return qs ? `?${qs}` : "";
}

/** Typed REST API client consuming generated OpenAPI contracts for `apps/web` and `apps/mobile`. */
export class TernakMatricApiClient {
  private readonly baseUrl: string;

  constructor(private readonly config: ClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/+$/, "");
  }

  async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    const headers = new Headers(init.headers);
    if (!headers.has("Accept")) {
      headers.set("Accept", "application/json");
    }
    if (init.body !== undefined && !headers.has("Content-Type")) {
      headers.set("Content-Type", "application/json");
    }

    const token = await this.config.getToken?.();
    if (token) {
      headers.set("Authorization", `Bearer ${token}`);
    }
    const orgId = await this.config.getOrganizationId?.();
    if (orgId) {
      headers.set("X-Organization-Id", orgId);
    }

    const normalizedPath = path.startsWith("/") ? path : `/${path}`;
    const res = await fetch(`${this.baseUrl}${normalizedPath}`, {
      ...init,
      headers,
    });
    const text = await res.text();
    if (!res.ok) {
      throw new ApiClientError(res.status, parseErrorEnvelope(text), text);
    }
    if (!text.trim()) {
      return undefined as T;
    }
    return JSON.parse(text) as T;
  }

  private get<Res>(
    path: string,
    query?: Record<string, string | number | undefined | null>,
  ): Promise<Res> {
    return this.request<Res>(`${path}${buildQueryString(query)}`, {
      method: "GET",
    });
  }

  private post<Req, Res>(path: string, body: Req): Promise<Res> {
    return this.request<Res>(path, {
      method: "POST",
      body: JSON.stringify(body),
    });
  }

  private patch<Req, Res>(path: string, body: Req): Promise<Res> {
    return this.request<Res>(path, {
      method: "PATCH",
      body: JSON.stringify(body),
    });
  }

  // System
  health(): Promise<HealthResponse> {
    return this.get<HealthResponse>("/health");
  }

  // Auth & Tenant
  register(body: RegisterRequest): Promise<AuthTokenResponse> {
    return this.post<RegisterRequest, AuthTokenResponse>(
      "/api/v1/auth/register",
      body,
    );
  }

  login(body: LoginRequest): Promise<AuthTokenResponse> {
    return this.post<LoginRequest, AuthTokenResponse>(
      "/api/v1/auth/login",
      body,
    );
  }

  listOrganizations(): Promise<OrganizationDto[]> {
    return this.get<OrganizationDto[]>("/api/v1/organizations");
  }

  createOrganization(
    body: CreateOrganizationRequest,
  ): Promise<OrganizationDto> {
    return this.post<CreateOrganizationRequest, OrganizationDto>(
      "/api/v1/organizations",
      body,
    );
  }

  listMembers(organizationId: string): Promise<MemberDto[]> {
    return this.get<MemberDto[]>(
      `/api/v1/organizations/${encodeURIComponent(organizationId)}/members`,
    );
  }

  addMember(
    organizationId: string,
    body: InviteMemberRequest,
  ): Promise<MemberDto> {
    return this.post<InviteMemberRequest, MemberDto>(
      `/api/v1/organizations/${encodeURIComponent(organizationId)}/members`,
      body,
    );
  }

  listFarms(): Promise<FarmDto[]> {
    return this.get<FarmDto[]>("/api/v1/farms");
  }

  createFarm(body: CreateFarmRequest): Promise<FarmDto> {
    return this.post<CreateFarmRequest, FarmDto>("/api/v1/farms", body);
  }

  updateFarm(farmId: string, body: UpdateFarmRequest): Promise<FarmDto> {
    return this.patch<UpdateFarmRequest, FarmDto>(
      `/api/v1/farms/${encodeURIComponent(farmId)}`,
      body,
    );
  }

  listAuditLogs(query?: AuditLogQuery): Promise<AuditLogDto[]> {
    return this.get<AuditLogDto[]>("/api/v1/audit-logs", {
      entity_type: query?.entity_type,
      entity_id: query?.entity_id,
      limit: query?.limit,
    });
  }

  // Enclosure
  listUnitTypes(): Promise<UnitTypeDto[]> {
    return this.get<UnitTypeDto[]>("/api/v1/unit-types");
  }

  listProductionSystems(): Promise<ProductionSystemDto[]> {
    return this.get<ProductionSystemDto[]>("/api/v1/production-systems");
  }

  listEnclosures(): Promise<EnclosureDto[]> {
    return this.get<EnclosureDto[]>("/api/v1/enclosures");
  }

  createEnclosure(body: CreateEnclosureRequest): Promise<EnclosureDto> {
    return this.post<CreateEnclosureRequest, EnclosureDto>(
      "/api/v1/enclosures",
      body,
    );
  }

  getEnclosure(enclosureId: string): Promise<EnclosureDto> {
    return this.get<EnclosureDto>(
      `/api/v1/enclosures/${encodeURIComponent(enclosureId)}`,
    );
  }

  updateEnclosure(
    enclosureId: string,
    body: UpdateEnclosureRequest,
  ): Promise<EnclosureDto> {
    return this.patch<UpdateEnclosureRequest, EnclosureDto>(
      `/api/v1/enclosures/${encodeURIComponent(enclosureId)}`,
      body,
    );
  }

  calculateEnclosureCapacity(
    enclosureId: string,
    body: CalculateCapacityRequest,
  ): Promise<CalculationResultCapacityEvaluation> {
    return this.post<
      CalculateCapacityRequest,
      CalculationResultCapacityEvaluation
    >(
      `/api/v1/enclosures/${encodeURIComponent(enclosureId)}/calculate-capacity`,
      body,
    );
  }

  // Species
  listSpecies(query?: SpeciesFilterQuery): Promise<SpeciesDto[]> {
    return this.get<SpeciesDto[]>("/api/v1/species", {
      category: query?.category,
    });
  }

  createCustomSpecies(body: CreateSpeciesRequest): Promise<SpeciesDto> {
    return this.post<CreateSpeciesRequest, SpeciesDto>("/api/v1/species", body);
  }

  listSpeciesProfiles(speciesId: string): Promise<SpeciesProfileDto[]> {
    return this.get<SpeciesProfileDto[]>(
      `/api/v1/species/${encodeURIComponent(speciesId)}/profiles`,
    );
  }

  createSpeciesProfile(
    speciesId: string,
    body: CreateProfileRequest,
  ): Promise<SpeciesProfileDto> {
    return this.post<CreateProfileRequest, SpeciesProfileDto>(
      `/api/v1/species/${encodeURIComponent(speciesId)}/profiles`,
      body,
    );
  }

  // Production
  listBatches(): Promise<BatchDto[]> {
    return this.get<BatchDto[]>("/api/v1/batches");
  }

  listBatchSummaries(query?: BatchSummaryQuery): Promise<BatchDetailDto[]> {
    return this.get<BatchDetailDto[]>("/api/v1/batches/summary", {
      status: query?.status,
      limit: query?.limit,
    });
  }

  getBatch(batchId: string): Promise<BatchDetailDto> {
    return this.get<BatchDetailDto>(
      `/api/v1/batches/${encodeURIComponent(batchId)}`,
    );
  }

  createBatch(body: CreateBatchRequest): Promise<BatchDto> {
    return this.post<CreateBatchRequest, BatchDto>("/api/v1/batches", body);
  }

  updateBatch(batchId: string, body: UpdateBatchRequest): Promise<BatchDto> {
    return this.patch<UpdateBatchRequest, BatchDto>(
      `/api/v1/batches/${encodeURIComponent(batchId)}`,
      body,
    );
  }

  recordStockMovement(
    batchId: string,
    body: StockMovementRequest,
  ): Promise<StockMovementResponse> {
    return this.post<StockMovementRequest, StockMovementResponse>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/stock-movements`,
      body,
    );
  }

  listIndividualTags(batchId: string): Promise<IndividualTagDto[]> {
    return this.get<IndividualTagDto[]>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/tags`,
    );
  }

  registerIndividualTag(
    batchId: string,
    body: RegisterTagRequest,
  ): Promise<IndividualTagDto> {
    return this.post<RegisterTagRequest, IndividualTagDto>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/tags`,
      body,
    );
  }

  listGrowthSamples(
    batchId: string,
    query?: PaginationQuery,
  ): Promise<GrowthSampleDto[]> {
    return this.get<GrowthSampleDto[]>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/growth-samples`,
      { limit: query?.limit },
    );
  }

  recordGrowthSample(
    batchId: string,
    body: CreateGrowthSampleRequest,
  ): Promise<GrowthSampleResponse> {
    return this.post<CreateGrowthSampleRequest, GrowthSampleResponse>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/growth-samples`,
      body,
    );
  }

  listOutputs(
    batchId: string,
    query?: PaginationQuery,
  ): Promise<ProductionOutputDto[]> {
    return this.get<ProductionOutputDto[]>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/outputs`,
      { limit: query?.limit },
    );
  }

  recordOutput(
    batchId: string,
    body: CreateOutputRequest,
  ): Promise<OutputResponse> {
    return this.post<CreateOutputRequest, OutputResponse>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/outputs`,
      body,
    );
  }

  // Feeding & Inventory
  listWarehouses(): Promise<WarehouseDto[]> {
    return this.get<WarehouseDto[]>("/api/v1/warehouses");
  }

  createWarehouse(body: CreateWarehouseRequest): Promise<WarehouseDto> {
    return this.post<CreateWarehouseRequest, WarehouseDto>(
      "/api/v1/warehouses",
      body,
    );
  }

  listFeedProducts(): Promise<FeedProductDto[]> {
    return this.get<FeedProductDto[]>("/api/v1/feed-products");
  }

  createFeedProduct(body: CreateFeedProductRequest): Promise<FeedProductDto> {
    return this.post<CreateFeedProductRequest, FeedProductDto>(
      "/api/v1/feed-products",
      body,
    );
  }

  listInventoryBalances(
    query?: InventoryBalanceQuery,
  ): Promise<InventoryBalanceDto[]> {
    return this.get<InventoryBalanceDto[]>("/api/v1/inventory/balances", {
      warehouse_id: query?.warehouse_id,
      feed_product_id: query?.feed_product_id,
      limit: query?.limit,
    });
  }

  createInventoryTransaction(
    body: CreateInventoryTxRequest,
  ): Promise<InventoryTxResponse> {
    return this.post<CreateInventoryTxRequest, InventoryTxResponse>(
      "/api/v1/inventory/transactions",
      body,
    );
  }

  recordFeeding(body: RecordFeedingRequest): Promise<FeedingRecordResponse> {
    return this.post<RecordFeedingRequest, FeedingRecordResponse>(
      "/api/v1/feeding",
      body,
    );
  }

  getBatchFcr(
    batchId: string,
    query?: FcrQuery,
  ): Promise<CalculationResultFcrSummary> {
    return this.get<CalculationResultFcrSummary>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/fcr`,
      { mode: query?.mode },
    );
  }

  // Environment
  listEnvironmentMetrics(): Promise<EnvironmentMetricDto[]> {
    return this.get<EnvironmentMetricDto[]>("/api/v1/environment/metrics");
  }

  listEnclosureReadings(
    enclosureId: string,
  ): Promise<EnvironmentReadingDto[]> {
    return this.get<EnvironmentReadingDto[]>(
      `/api/v1/enclosures/${encodeURIComponent(enclosureId)}/environment`,
    );
  }

  recordManualReading(
    body: CreateReadingRequest,
  ): Promise<EnvironmentReadingDto> {
    return this.post<CreateReadingRequest, EnvironmentReadingDto>(
      "/api/v1/environment/readings",
      body,
    );
  }

  ingestIotReading(body: IotIngestRequest): Promise<EnvironmentReadingDto> {
    return this.post<IotIngestRequest, EnvironmentReadingDto>(
      "/api/v1/environment/iot-ingest",
      body,
    );
  }

  // Health & Biosecurity
  listVaccinations(batchId: string): Promise<VaccinationScheduleDto[]> {
    return this.get<VaccinationScheduleDto[]>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/vaccinations`,
    );
  }

  scheduleVaccination(
    batchId: string,
    body: CreateVaccinationScheduleRequest,
  ): Promise<VaccinationScheduleDto> {
    return this.post<CreateVaccinationScheduleRequest, VaccinationScheduleDto>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/vaccinations`,
      body,
    );
  }

  updateVaccinationStatus(
    vaccinationId: string,
    body: UpdateVaccinationStatusRequest,
  ): Promise<VaccinationScheduleDto> {
    return this.patch<UpdateVaccinationStatusRequest, VaccinationScheduleDto>(
      `/api/v1/vaccinations/${encodeURIComponent(vaccinationId)}`,
      body,
    );
  }

  listHealthRecords(
    batchId: string,
    query?: PaginationQuery,
  ): Promise<HealthRecordDto[]> {
    return this.get<HealthRecordDto[]>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/health-records`,
      { limit: query?.limit },
    );
  }

  recordHealthEvent(
    batchId: string,
    body: CreateHealthRecordRequest,
  ): Promise<HealthRecordDto> {
    return this.post<CreateHealthRecordRequest, HealthRecordDto>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/health-records`,
      body,
    );
  }

  listMortalities(
    batchId: string,
    query?: PaginationQuery,
  ): Promise<MortalityRecordDto[]> {
    return this.get<MortalityRecordDto[]>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/mortalities`,
      { limit: query?.limit },
    );
  }

  recordMortality(
    batchId: string,
    body: RecordMortalityRequest,
  ): Promise<MortalityResponse> {
    return this.post<RecordMortalityRequest, MortalityResponse>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/mortalities`,
      body,
    );
  }

  recordBiosecurityLog(
    farmId: string,
    body: CreateBiosecurityLogRequest,
  ): Promise<BiosecurityLogDto> {
    return this.post<CreateBiosecurityLogRequest, BiosecurityLogDto>(
      `/api/v1/farms/${encodeURIComponent(farmId)}/biosecurity-logs`,
      body,
    );
  }

  // Finance
  listExpenseCategories(): Promise<ExpenseCategoryDto[]> {
    return this.get<ExpenseCategoryDto[]>(
      "/api/v1/finance/expense-categories",
    );
  }

  listExpenses(query?: ExpenseQuery): Promise<ExpenseDto[]> {
    return this.get<ExpenseDto[]>("/api/v1/finance/expenses", {
      batch_id: query?.batch_id,
      farm_id: query?.farm_id,
      category_id: query?.category_id,
      limit: query?.limit,
    });
  }

  recordExpense(body: CreateExpenseRequest): Promise<ExpenseDto> {
    return this.post<CreateExpenseRequest, ExpenseDto>(
      "/api/v1/finance/expenses",
      body,
    );
  }

  listSales(query?: SaleQuery): Promise<SaleDto[]> {
    return this.get<SaleDto[]>("/api/v1/finance/sales", {
      batch_id: query?.batch_id,
      limit: query?.limit,
    });
  }

  recordSale(body: CreateSaleRequest): Promise<SaleResponse> {
    return this.post<CreateSaleRequest, SaleResponse>(
      "/api/v1/finance/sales",
      body,
    );
  }

  getBatchFinancialSummary(
    batchId: string,
  ): Promise<BatchFinancialSummaryResponse> {
    return this.get<BatchFinancialSummaryResponse>(
      `/api/v1/batches/${encodeURIComponent(batchId)}/financial-summary`,
    );
  }

  // Offline Sync
  registerDevice(body: RegisterDeviceRequest): Promise<RegisterDeviceResponse> {
    return this.post<RegisterDeviceRequest, RegisterDeviceResponse>(
      "/api/v1/sync/devices",
      body,
    );
  }

  pushSyncCommands(body: SyncPushRequest): Promise<SyncPushResponse> {
    return this.post<SyncPushRequest, SyncPushResponse>(
      "/api/v1/sync/push",
      body,
    );
  }

  pullSyncChanges(query?: SyncPullQuery): Promise<SyncPullResponse> {
    return this.get<SyncPullResponse>("/api/v1/sync/pull", {
      cursor: query?.cursor,
      limit: query?.limit,
    });
  }

  listSyncConflicts(): Promise<SyncConflictRecord[]> {
    return this.get<SyncConflictRecord[]>("/api/v1/sync/conflicts");
  }
}
