import { HttpClient } from '../http.js';

export class AuthService {
  constructor(private http: HttpClient) {}

  register(tenantId: string, email: string, password: string, phone?: string): Promise<any> {
    const body: any = { tenant_id: tenantId, email, password };
    if (phone) body.phone = phone;
    return this.http.post('/api/auth/register', body);
  }

  login(tenantId: string, email: string, password: string): Promise<any> {
    return this.http.post('/api/auth/login', { tenant_id: tenantId, email, password });
  }

  loginHierarchical(email: string, password: string, organizationSlug?: string): Promise<any> {
    const body: any = { email, password };
    if (organizationSlug) body.organization_slug = organizationSlug;
    return this.http.post('/api/auth/login-hierarchical', body);
  }

  refreshToken(refreshToken: string): Promise<any> {
    return this.http.post('/api/auth/refresh', { refresh_token: refreshToken });
  }

  logout(): Promise<any> {
    return this.http.post('/api/auth/logout');
  }

  getProfile(): Promise<any> {
    return this.http.get('/api/auth/me');
  }

  updateProfile(data: Record<string, any>): Promise<any> {
    return this.http.patch('/api/auth/me', data);
  }

  changePassword(currentPassword: string, newPassword: string): Promise<any> {
    return this.http.post('/api/auth/change-password', {
      current_password: currentPassword,
      new_password: newPassword,
    });
  }

  verifyEmail(token: string): Promise<any> {
    return this.http.get('/api/verify-email', { token });
  }

  resendVerification(): Promise<any> {
    return this.http.post('/api/auth/resend-verification');
  }

  forgotPassword(tenantId: string, email: string): Promise<any> {
    return this.http.post('/api/auth/forgot-password', { tenant_id: tenantId, email });
  }

  verifyResetToken(token: string): Promise<any> {
    return this.http.get('/api/auth/verify-reset-token', { token });
  }

  resetPassword(token: string, newPassword: string): Promise<any> {
    return this.http.post('/api/auth/reset-password', { token, new_password: newPassword });
  }

  passwordlessStart(tenantId: string, method: string, email: string): Promise<any> {
    return this.http.post(`/api/tenants/${tenantId}/passwordless/start`, { method, email });
  }

  passwordlessVerify(tenantId: string, tokenOrCode: string): Promise<any> {
    return this.http.post(`/api/tenants/${tenantId}/passwordless/verify`, { token_or_code: tokenOrCode });
  }

  passwordlessResend(tenantId: string, data: Record<string, any>): Promise<any> {
    return this.http.post(`/api/tenants/${tenantId}/passwordless/resend`, data);
  }

  createLoginFlowBrowser(params?: { organization_id?: string; request_id?: string }): Promise<any> {
    return this.http.get('/self-service/login/browser', params);
  }

  createLoginFlowApi(params?: { organization_id?: string; request_id?: string }): Promise<any> {
    return this.http.get('/self-service/login/api', params);
  }

  getLoginFlow(flowId: string): Promise<any> {
    return this.http.get('/self-service/login', { flow: flowId });
  }

  submitLoginFlow(flowId: string, data: Record<string, any>): Promise<any> {
    return this.http.post('/self-service/login', { flow: flowId, ...data });
  }

  createRegistrationFlowBrowser(params?: { organization_id?: string }): Promise<any> {
    return this.http.get('/self-service/registration/browser', params);
  }

  createRegistrationFlowApi(params?: { organization_id?: string }): Promise<any> {
    return this.http.get('/self-service/registration/api', params);
  }

  getRegistrationFlow(flowId: string): Promise<any> {
    return this.http.get('/self-service/registration', { flow: flowId });
  }

  submitRegistrationFlow(flowId: string, data: Record<string, any>): Promise<any> {
    return this.http.post('/self-service/registration', { flow: flowId, ...data });
  }

  whoami(): Promise<any> {
    return this.http.get('/sessions/whoami');
  }
}
