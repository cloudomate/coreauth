import { HttpClient } from '../http.js';

export class TenantsService {
  constructor(private http: HttpClient) {}

  create(data: Record<string, any>): Promise<any> {
    return this.http.post('/api/tenants', data);
  }

  getBySlug(slug: string): Promise<any> {
    return this.http.get(`/api/organizations/by-slug/${slug}`);
  }

  listUsers(tenantId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/users`);
  }

  updateUserRole(tenantId: string, userId: string, role: string): Promise<any> {
    return this.http.put(`/api/tenants/${tenantId}/users/${userId}/role`, { role });
  }

  getSecurity(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/security`);
  }

  updateSecurity(orgId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/organizations/${orgId}/security`, data);
  }

  getBranding(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/branding`);
  }

  updateBranding(orgId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/organizations/${orgId}/branding`, data);
  }
}
