import type {
  AuthTokenResponse,
  LoginRequest,
  MemberDto,
  OrganizationDto,
  RegisterRequest,
} from "@ternakmatric/api-contracts";

export type {
  AuthTokenResponse,
  LoginRequest,
  MemberDto,
  OrganizationDto,
  RegisterRequest,
};

export interface UserSession {
  token: string | null;
  organizationId: string | null;
  userEmail: string | null;
  roles: string[];
  permissions: string[];
}
