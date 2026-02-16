import { HttpClient } from '../http.js';

export class GroupsService {
  constructor(private http: HttpClient) {}

  // --- Groups ---

  create(tenantId: string, name: string, data?: Record<string, any>): Promise<any> {
    return this.http.post(`/api/tenants/${tenantId}/groups`, { name, ...data });
  }

  list(tenantId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/groups`);
  }

  get(tenantId: string, groupId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/groups/${groupId}`);
  }

  update(tenantId: string, groupId: string, data: Record<string, any>): Promise<any> {
    return this.http.put(`/api/tenants/${tenantId}/groups/${groupId}`, data);
  }

  delete(tenantId: string, groupId: string): Promise<any> {
    return this.http.delete(`/api/tenants/${tenantId}/groups/${groupId}`);
  }

  // --- Members ---

  addMember(tenantId: string, groupId: string, userId: string, role?: string): Promise<any> {
    const body: Record<string, any> = { user_id: userId };
    if (role) body.role = role;
    return this.http.post(`/api/tenants/${tenantId}/groups/${groupId}/members`, body);
  }

  listMembers(tenantId: string, groupId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/groups/${groupId}/members`);
  }

  updateMember(tenantId: string, groupId: string, userId: string, data: Record<string, any>): Promise<any> {
    return this.http.patch(`/api/tenants/${tenantId}/groups/${groupId}/members/${userId}`, data);
  }

  removeMember(tenantId: string, groupId: string, userId: string): Promise<any> {
    return this.http.delete(`/api/tenants/${tenantId}/groups/${groupId}/members/${userId}`);
  }

  // --- Roles ---

  assignRole(tenantId: string, groupId: string, roleId: string): Promise<any> {
    return this.http.post(`/api/tenants/${tenantId}/groups/${groupId}/roles`, { role_id: roleId });
  }

  listRoles(tenantId: string, groupId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/groups/${groupId}/roles`);
  }

  removeRole(tenantId: string, groupId: string, roleId: string): Promise<any> {
    return this.http.delete(`/api/tenants/${tenantId}/groups/${groupId}/roles/${roleId}`);
  }

  // --- User groups ---

  getUserGroups(tenantId: string, userId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/users/${userId}/groups`);
  }

  // --- Invitations ---

  createInvitation(tenantId: string, email: string, data?: Record<string, any>): Promise<any> {
    return this.http.post(`/api/tenants/${tenantId}/invitations`, { email, ...data });
  }

  listInvitations(tenantId: string): Promise<any> {
    return this.http.get(`/api/tenants/${tenantId}/invitations`);
  }

  revokeInvitation(tenantId: string, invitationId: string): Promise<any> {
    return this.http.delete(`/api/tenants/${tenantId}/invitations/${invitationId}`);
  }

  resendInvitation(tenantId: string, invitationId: string): Promise<any> {
    return this.http.post(`/api/tenants/${tenantId}/invitations/${invitationId}/resend`);
  }

  verifyInvitation(token: string): Promise<any> {
    return this.http.get('/api/invitations/verify', { token });
  }

  acceptInvitation(data: Record<string, any>): Promise<any> {
    return this.http.post('/api/invitations/accept', data);
  }
}
