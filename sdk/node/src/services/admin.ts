import { HttpClient } from '../http.js';

export class AdminService {
  constructor(private http: HttpClient) {}

  // --- Tenant Registry ---

  listTenants(): Promise<any> {
    return this.http.get('/api/admin/tenants');
  }

  createTenant(slug: string, name: string, isolationMode?: string): Promise<any> {
    const body: Record<string, any> = { slug, name };
    if (isolationMode) body.isolation_mode = isolationMode;
    return this.http.post('/api/admin/tenants', body);
  }

  getStats(): Promise<any> {
    return this.http.get('/api/admin/tenants/stats');
  }

  getTenant(tenantId: string): Promise<any> {
    return this.http.get(`/api/admin/tenants/${tenantId}`);
  }

  configureDatabase(tenantId: string, connectionString: string): Promise<any> {
    return this.http.post(`/api/admin/tenants/${tenantId}/database`, {
      connection_string: connectionString,
    });
  }

  activate(tenantId: string): Promise<any> {
    return this.http.post(`/api/admin/tenants/${tenantId}/activate`);
  }

  suspend(tenantId: string): Promise<any> {
    return this.http.post(`/api/admin/tenants/${tenantId}/suspend`);
  }

  testConnection(tenantId: string): Promise<any> {
    return this.http.post(`/api/admin/tenants/${tenantId}/test-connection`);
  }

  // --- Actions ---

  createAction(orgId: string, name: string, triggerType: string, code: string, data?: Record<string, any>): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/actions`, {
      name,
      trigger_type: triggerType,
      code,
      ...data,
    });
  }

  listActions(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/actions`);
  }

  getAction(orgId: string, actionId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/actions/${actionId}`);
  }

  updateAction(orgId: string, actionId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/organizations/${orgId}/actions/${actionId}`, data);
  }

  deleteAction(orgId: string, actionId: string): Promise<any> {
    return this.http.delete(`/api/organizations/${orgId}/actions/${actionId}`);
  }

  testAction(orgId: string, actionId: string, data?: Record<string, any>): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/actions/${actionId}/test`, data);
  }

  getActionExecutions(orgId: string, actionId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/actions/${actionId}/executions`);
  }

  getOrgExecutions(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/actions/executions`);
  }

  // --- Rate limits ---

  getRateLimits(tenantId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/rate-limits`);
  }

  updateRateLimits(tenantId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/tenants/${tenantId}/rate-limits`, data);
  }

  // --- Token claims ---

  getTokenClaims(tenantId: string, appId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/applications/${appId}/token-claims`);
  }

  updateTokenClaims(tenantId: string, appId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/tenants/${tenantId}/applications/${appId}/token-claims`, data);
  }

  // --- Health ---

  health(): Promise<any> {
    return this.http.get('/health');
  }
}
