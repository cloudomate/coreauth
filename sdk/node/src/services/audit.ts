import { HttpClient } from '../http.js';

export class AuditService {
  constructor(private http: HttpClient) {}

  query(params?: Record<string, any>): Promise<any> {
    return this.http.get('/api/audit/logs', params);
  }

  get(logId: string): Promise<any> {
    return this.http.get(`/api/audit/logs/${logId}`);
  }

  securityEvents(): Promise<any> {
    return this.http.get('/api/audit/security-events');
  }

  failedLogins(userId: string): Promise<any> {
    return this.http.get(`/api/audit/failed-logins/${userId}`);
  }

  export(): Promise<any> {
    return this.http.get('/api/audit/export');
  }

  stats(): Promise<any> {
    return this.http.get('/api/audit/stats');
  }

  loginHistory(): Promise<any> {
    return this.http.get('/api/login-history');
  }

  securityAuditLogs(): Promise<any> {
    return this.http.get('/api/security/audit-logs');
  }
}
