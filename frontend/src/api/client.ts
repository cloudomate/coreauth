import axios from 'axios';
import type {
  AuthResponse,
  User,
  MfaMethod,
  TenantSecuritySettings,
  OidcProvider,
  Invitation
} from '../types';

// Use relative URL so nginx can proxy to backend
const API_BASE_URL = import.meta.env.VITE_API_URL || '';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add auth token to requests
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Handle token refresh on 401
api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config;

    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true;

      const refreshToken = localStorage.getItem('refresh_token');
      if (refreshToken) {
        try {
          const { data } = await axios.post(`${API_BASE_URL}/api/auth/refresh`, {
            refresh_token: refreshToken,
          });

          localStorage.setItem('access_token', data.access_token);
          localStorage.setItem('refresh_token', data.refresh_token);

          originalRequest.headers.Authorization = `Bearer ${data.access_token}`;
          return api(originalRequest);
        } catch (refreshError) {
          localStorage.removeItem('access_token');
          localStorage.removeItem('refresh_token');
          window.location.href = '/login';
          return Promise.reject(refreshError);
        }
      }
    }

    return Promise.reject(error);
  }
);

export const authApi = {
  register: (data: { tenant_id: string; email: string; password: string; phone?: string }) =>
    api.post<AuthResponse>('/api/auth/register', data),

  login: (data: { tenant_id: string; email: string; password: string }) =>
    api.post<AuthResponse>('/api/auth/login', data),

  logout: () => api.post('/api/auth/logout'),

  me: () => api.get<User>('/api/auth/me'),

  forgotPassword: (data: { tenant_id: string; email: string }) =>
    api.post('/api/auth/forgot-password', data),

  resetPassword: (data: { token: string; new_password: string }) =>
    api.post('/api/auth/reset-password', data),

  resendVerification: () => api.post('/api/auth/resend-verification'),
};

export const mfaApi = {
  enrollTotp: () => api.post<{ secret: string; qr_code_uri: string; backup_codes: string[] }>('/api/mfa/enroll/totp'),

  verifyTotp: (methodId: string, code: string) =>
    api.post(`/api/mfa/totp/${methodId}/verify`, { code }),

  listMethods: () => api.get<MfaMethod[]>('/api/mfa/methods'),

  deleteMethod: (methodId: string) => api.delete(`/api/mfa/methods/${methodId}`),

  regenerateBackupCodes: () =>
    api.post<{ backup_codes: string[] }>('/api/mfa/backup-codes/regenerate'),
};

export const tenantApi = {
  create: (data: {
    name: string;
    slug: string;
    admin_email: string;
    admin_password: string;
    admin_full_name: string;
  }) => api.post<{
    tenant_id: string;
    tenant_name: string;
    admin_user_id: string;
    message: string;
  }>('/api/tenants', data),

  getSecurityPolicy: (tenantId: string) =>
    api.get<{ tenant_id: string; security: TenantSecuritySettings }>(`/api/tenants/${tenantId}/security`),

  updateSecurityPolicy: (tenantId: string, settings: Partial<TenantSecuritySettings>) =>
    api.post(`/api/tenants/${tenantId}/security`, settings),

  getMfaStatus: (tenantId: string) =>
    api.get<{
      total_users: number;
      users_with_mfa: number;
      mfa_coverage_percent: number;
    }>(`/api/tenants/${tenantId}/mfa-status`),

  enforceMfa: (tenantId: string) => api.post(`/api/tenants/${tenantId}/enforce-mfa`),

  // OIDC Providers
  listOidcProviders: () => api.get<OidcProvider[]>('/api/oidc/providers'),

  listOidcTemplates: () => api.get<import('../types').OidcProviderTemplate[]>('/api/oidc/templates'),

  getOidcTemplate: (providerType: string) =>
    api.get<import('../types').OidcProviderTemplate>(`/api/oidc/templates/${providerType}`),

  createOidcProvider: (data: {
    tenant_id: string;
    name: string;
    provider_type: string;
    client_id: string;
    client_secret: string;
    issuer: string;
    authorization_endpoint: string;
    token_endpoint: string;
    userinfo_endpoint?: string;
    jwks_uri: string;
    scopes?: string[];
    groups_claim?: string;
    group_role_mappings?: Record<string, string>;
  }) => api.post<OidcProvider>('/api/oidc/providers', data),

  // Invitations
  listInvitations: (tenantId: string) =>
    api.get<Invitation[]>(`/api/tenants/${tenantId}/invitations`),

  createInvitation: (tenantId: string, data: {
    email: string;
    role_id?: string;
    metadata?: Record<string, any>;
    expires_in_days?: number;
  }) => api.post(`/api/tenants/${tenantId}/invitations`, data),

  revokeInvitation: (tenantId: string, invitationId: string) =>
    api.delete(`/api/tenants/${tenantId}/invitations/${invitationId}`),

  resendInvitation: (tenantId: string, invitationId: string) =>
    api.post(`/api/tenants/${tenantId}/invitations/${invitationId}/resend`),
};

export const invitationApi = {
  verify: (token: string) => api.get<Invitation>(`/api/invitations/verify?token=${token}`),

  accept: (data: {
    token: string;
    password: string;
    full_name: string;
    metadata?: Record<string, any>;
  }) => api.post('/api/invitations/accept', data),
};

// Applications API
export const applicationApi = {
  list: (orgId: string, params?: { limit?: number; offset?: number }) =>
    api.get(`/api/organizations/${orgId}/applications`, { params }),

  get: (orgId: string, appId: string) =>
    api.get(`/api/organizations/${orgId}/applications/${appId}`),

  create: (orgId: string, data: {
    name: string;
    slug: string;
    description?: string;
    app_type: string;
    callback_urls?: string[];
    logout_urls?: string[];
    web_origins?: string[];
  }) => api.post(`/api/organizations/${orgId}/applications`, data),

  update: (orgId: string, appId: string, data: Partial<{
    name: string;
    description: string;
    callback_urls: string[];
    logout_urls: string[];
    web_origins: string[];
    is_enabled: boolean;
  }>) => api.put(`/api/organizations/${orgId}/applications/${appId}`, data),

  delete: (orgId: string, appId: string) =>
    api.delete(`/api/organizations/${orgId}/applications/${appId}`),

  rotateSecret: (orgId: string, appId: string) =>
    api.post(`/api/organizations/${orgId}/applications/${appId}/rotate-secret`),
};

// Actions API
export const actionApi = {
  list: (orgId: string, params?: { limit?: number; offset?: number }) =>
    api.get(`/api/organizations/${orgId}/actions`, { params }),

  get: (orgId: string, actionId: string) =>
    api.get(`/api/organizations/${orgId}/actions/${actionId}`),

  create: (orgId: string, data: {
    name: string;
    description?: string;
    trigger_type: string;
    code: string;
    runtime?: string;
    timeout_seconds?: number;
    secrets?: Record<string, any>;
    execution_order?: number;
    is_enabled?: boolean;
  }) => api.post(`/api/organizations/${orgId}/actions`, data),

  update: (orgId: string, actionId: string, data: Partial<{
    name: string;
    description: string;
    code: string;
    timeout_seconds: number;
    secrets: Record<string, any>;
    execution_order: number;
    is_enabled: boolean;
  }>) => api.put(`/api/organizations/${orgId}/actions/${actionId}`, data),

  delete: (orgId: string, actionId: string) =>
    api.delete(`/api/organizations/${orgId}/actions/${actionId}`),

  test: (orgId: string, actionId: string, context: any) =>
    api.post(`/api/organizations/${orgId}/actions/${actionId}/test`, context),

  getExecutions: (orgId: string, actionId: string, params?: { limit?: number; offset?: number }) =>
    api.get(`/api/organizations/${orgId}/actions/${actionId}/executions`, { params }),

  getOrgExecutions: (orgId: string, params?: { limit?: number; offset?: number }) =>
    api.get(`/api/organizations/${orgId}/actions/executions`, { params }),
};

// Connections API (SSO providers)
export const connectionApi = {
  list: (orgId: string, params?: { limit?: number; offset?: number }) =>
    api.get(`/api/organizations/${orgId}/connections`, { params }),

  get: (orgId: string, connId: string) =>
    api.get(`/api/organizations/${orgId}/connections/${connId}`),

  create: (orgId: string, data: {
    name: string;
    connection_type: string;
    configuration: Record<string, any>;
    is_enabled?: boolean;
  }) => api.post(`/api/organizations/${orgId}/connections`, data),

  update: (orgId: string, connId: string, data: Partial<{
    name: string;
    configuration: Record<string, any>;
    is_enabled: boolean;
  }>) => api.put(`/api/organizations/${orgId}/connections/${connId}`, data),

  delete: (orgId: string, connId: string) =>
    api.delete(`/api/organizations/${orgId}/connections/${connId}`),

  test: (orgId: string, connId: string) =>
    api.post(`/api/organizations/${orgId}/connections/${connId}/test`),
};

// Organizations API
export const organizationApi = {
  list: (params?: { limit?: number; offset?: number }) =>
    api.get('/api/organizations', { params }),

  get: (orgId: string) =>
    api.get(`/api/organizations/${orgId}`),

  create: (data: {
    name: string;
    slug: string;
    parent_organization_id?: string;
  }) => api.post('/api/organizations', data),

  update: (orgId: string, data: Partial<{
    name: string;
    slug: string;
    settings: Record<string, any>;
  }>) => api.put(`/api/organizations/${orgId}`, data),

  delete: (orgId: string) =>
    api.delete(`/api/organizations/${orgId}`),

  listChildren: (parentId: string) =>
    api.get(`/api/organizations/${parentId}/organizations`),

  getHierarchy: (orgId: string) =>
    api.get(`/api/organizations/${orgId}/hierarchy`),
};

export default api;
