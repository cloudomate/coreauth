import { HttpClient } from '../http.js';

export class ScimService {
  constructor(private http: HttpClient) {}

  // --- SCIM Config ---

  getConfig(): Promise<any> {
    return this.http.get('/scim/v2/ServiceProviderConfig');
  }

  getResourceTypes(): Promise<any> {
    return this.http.get('/scim/v2/ResourceTypes');
  }

  getSchemas(): Promise<any> {
    return this.http.get('/scim/v2/Schemas');
  }

  // --- SCIM Users ---

  listUsers(params?: {
    filter?: string;
    count?: number;
    startIndex?: number;
    [key: string]: any;
  }): Promise<any> {
    return this.http.get('/scim/v2/Users', params);
  }

  createUser(data: Record<string, any>): Promise<any> {
    return this.http.post('/scim/v2/Users', data);
  }

  getUser(userId: string): Promise<any> {
    return this.http.get(`/scim/v2/Users/${userId}`);
  }

  replaceUser(userId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/scim/v2/Users/${userId}`, data);
  }

  patchUser(userId: string, operations: Record<string, any>[]): Promise<any> {
    return this.http.patch(`/scim/v2/Users/${userId}`, { Operations: operations });
  }

  deleteUser(userId: string): Promise<any> {
    return this.http.delete(`/scim/v2/Users/${userId}`);
  }

  // --- SCIM Groups ---

  listScimGroups(params?: {
    filter?: string;
    count?: number;
    startIndex?: number;
    [key: string]: any;
  }): Promise<any> {
    return this.http.get('/scim/v2/Groups', params);
  }

  createScimGroup(data: Record<string, any>): Promise<any> {
    return this.http.post('/scim/v2/Groups', data);
  }

  getScimGroup(groupId: string): Promise<any> {
    return this.http.get(`/scim/v2/Groups/${groupId}`);
  }

  patchScimGroup(groupId: string, operations: Record<string, any>[]): Promise<any> {
    return this.http.patch(`/scim/v2/Groups/${groupId}`, { Operations: operations });
  }

  deleteScimGroup(groupId: string): Promise<any> {
    return this.http.delete(`/scim/v2/Groups/${groupId}`);
  }

  // --- SCIM Tokens ---

  listScimTokens(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/scim/tokens`);
  }

  createScimToken(orgId: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/scim/tokens`, data);
  }

  revokeScimToken(orgId: string, tokenId: string): Promise<any> {
    return this.http.delete(`/api/organizations/${orgId}/scim/tokens/${tokenId}`);
  }

  // --- Sessions ---

  listSessions(userId?: string): Promise<any> {
    const params: Record<string, any> = {};
    if (userId) params.user_id = userId;
    return this.http.get('/api/sessions', Object.keys(params).length ? params : undefined);
  }

  revokeSession(sessionId: string): Promise<any> {
    return this.http.delete(`/api/sessions/${sessionId}`);
  }

  revokeAllSessions(): Promise<any> {
    return this.http.post('/api/sessions/revoke-all');
  }

  // --- OIDC Providers ---

  listOidcProviders(tenantId: string): Promise<any> {
    return this.http.get('/api/oidc/providers', { tenant_id: tenantId });
  }

  createOidcProvider(data: Record<string, any>): Promise<any> {
    return this.http.post('/api/oidc/providers', data);
  }

  updateOidcProvider(providerId: string, data: Record<string, any>): Promise<any> {
    return this.http.patch(`/api/oidc/providers/${providerId}`, data);
  }

  deleteOidcProvider(providerId: string): Promise<any> {
    return this.http.delete(`/api/oidc/providers/${providerId}`);
  }

  listPublicProviders(tenantId: string): Promise<any> {
    return this.http.get('/api/oidc/providers/public', { tenant_id: tenantId });
  }

  listProviderTemplates(): Promise<any> {
    return this.http.get('/api/oidc/templates');
  }

  getProviderTemplate(providerType: string): Promise<any> {
    return this.http.get(`/api/oidc/templates/${providerType}`);
  }

  ssoCheck(email: string): Promise<any> {
    return this.http.get('/api/oidc/sso-check', { email });
  }
}
