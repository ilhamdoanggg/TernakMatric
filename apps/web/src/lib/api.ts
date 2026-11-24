import { TernakMatricApiClient } from "@ternakmatric/api-contracts";

interface GlobalWithProcess {
  process?: {
    env?: Record<string, string | undefined>;
  };
}

export function createWebApiClient(
  token?: string,
  organizationId?: string,
  baseUrlOverride?: string,
) {
  const ssrUrl = import.meta.env.SSR
    ? (globalThis as GlobalWithProcess).process?.env?.API_INTERNAL_URL
    : undefined;
  const baseUrl =
    baseUrlOverride ??
    ssrUrl ??
    import.meta.env.VITE_API_URL ??
    "http://localhost:8080";

  return new TernakMatricApiClient({
    baseUrl,
    getToken: () => token ?? null,
    getOrganizationId: () => organizationId ?? null,
  });
}
