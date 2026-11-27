import { createWebApiClient } from "@/lib/api";
import type {
  AuthTokenResponse,
  LoginRequest,
  RegisterRequest,
} from "@/types/auth";

export class AuthService {
  private getClient() {
    return createWebApiClient();
  }

  async login(payload: LoginRequest): Promise<AuthTokenResponse> {
    const client = this.getClient();
    return client.login(payload);
  }

  async register(payload: RegisterRequest): Promise<AuthTokenResponse> {
    const client = this.getClient();
    return client.register(payload);
  }
}

export const authService = new AuthService();
