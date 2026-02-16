import { HttpClient } from '../http.js';

export class ConnectionsService {
  constructor(private http: HttpClient) {}

  async list(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/connections`);
  }

  async create(orgId: string, data: Record<string, unknown>): Promise<any> {
    return this.http.post(`/api/organizations/${orgId}/connections`, data);
  }

  async get(orgId: string, connectionId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/connections/${connectionId}`);
  }

  async update(orgId: string, connectionId: string, data: Record<string, unknown>): Promise<any> {
    return this.http.put(`/api/organizations/${orgId}/connections/${connectionId}`, data);
  }

  async delete(orgId: string, connectionId: string): Promise<void> {
    await this.http.delete(`/api/organizations/${orgId}/connections/${connectionId}`);
  }

  async getAuthMethods(orgId: string): Promise<any> {
    return this.http.get(`/api/organizations/${orgId}/connections/auth-methods`);
  }

  async listAll(): Promise<any> {
    return this.http.get('/api/admin/connections');
  }

  async createPlatform(data: Record<string, unknown>): Promise<any> {
    return this.http.post('/api/admin/connections', data);
  }
}
