import { HttpClient } from '../http.js';

export class ApplicationsService {
  constructor(private http: HttpClient) {}

  // --- Authz Applications ---

  create(data: Record<string, any>): Promise<any> {
    return this.http.post('/api/applications', data);
  }

  list(tenantId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/applications`);
  }

  get(appId: string, tenantId: string): Promise<any> {
    return this.http.get(`/api/applications/${appId}/tenants/${tenantId}`);
  }

  update(appId: string, tenantId: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/applications/${appId}/tenants/${tenantId}`, data);
  }

  rotateSecret(appId: string, tenantId: string): Promise<any> {
    return this.http.post(`/api/applications/${appId}/tenants/${tenantId}/rotate-secret`);
  }

  delete(appId: string, tenantId: string): Promise<any> {
    return this.http.delete(`/api/applications/${appId}/tenants/${tenantId}`);
  }

  authenticate(clientId: string, clientSecret: string): Promise<any> {
    return this.http.post('/api/applications/authenticate', {
      client_id: clientId,
      client_secret: clientSecret,
    });
  }

  // --- OAuth Applications ---

  createOAuthApp(orgId: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/applications`, data);
  }

  listOAuthApps(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/applications`);
  }

  getOAuthApp(orgId: string, appId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/applications/${appId}`);
  }

  updateOAuthApp(orgId: string, appId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/organizations/${orgId}/applications/${appId}`, data);
  }

  rotateOAuthSecret(orgId: string, appId: string): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/applications/${appId}/rotate-secret`);
  }

  deleteOAuthApp(orgId: string, appId: string): Promise<any> {
    return this.http.delete(`/api/organizations/${orgId}/applications/${appId}`);
  }

  // --- Email Templates ---

  listEmailTemplates(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/email-templates`);
  }

  getEmailTemplate(orgId: string, type: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/email-templates/${type}`);
  }

  updateEmailTemplate(orgId: string, type: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/organizations/${orgId}/email-templates/${type}`, data);
  }

  deleteEmailTemplate(orgId: string, type: string): Promise<any> {
    return this.http.delete(`/api/organizations/${orgId}/email-templates/${type}`);
  }

  previewEmailTemplate(orgId: string, type: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/email-templates/${type}/preview`, data);
  }
}
