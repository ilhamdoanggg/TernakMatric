import { computed, ref } from "vue";

const token = ref<string | null>(null);
const organizationId = ref<string | null>(null);
const userEmail = ref<string | null>(null);
const roles = ref<string[]>(["ORG_ADMIN"]); // Default admin role for dev/operator dashboard
const permissions = ref<string[]>(["org.manage"]);

export function useAuthStore() {
  const isAuthenticated = computed(() => Boolean(token.value));

  const canManageOrg = computed(
    () =>
      roles.value.includes("ORG_OWNER") ||
      roles.value.includes("ORG_ADMIN") ||
      permissions.value.includes("org.manage"),
  );

  function setSession(newToken: string | null, newOrgId: string | null, email?: string) {
    token.value = newToken;
    organizationId.value = newOrgId;
    if (email) {
      userEmail.value = email;
    }
  }

  function clearSession() {
    token.value = null;
    organizationId.value = null;
    userEmail.value = null;
  }

  return {
    token,
    organizationId,
    userEmail,
    roles,
    permissions,
    isAuthenticated,
    canManageOrg,
    setSession,
    clearSession,
  };
}
