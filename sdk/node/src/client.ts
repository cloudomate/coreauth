import { HttpClient } from './http.js';
import { AuthService } from './services/auth.js';
import { OAuth2Service } from './services/oauth2.js';
import { MfaService } from './services/mfa.js';
import { TenantsService } from './services/tenants.js';
import { ApplicationsService } from './services/applications.js';
import { FgaService } from './services/fga.js';
import { AuditService } from './services/audit.js';
import { WebhooksService } from './services/webhooks.js';
import { GroupsService } from './services/groups.js';
import { ScimService } from './services/scim.js';
import { AdminService } from './services/admin.js';
import { ConnectionsService } from './services/connections.js';

export interface CoreAuthClientOptions {
  token?: string;
}

/**
 * Main client for the CoreAuth API.
 *
 * @example
 * ```ts
 * const client = new CoreAuthClient('http://localhost:3000');
 * const resp = await client.auth.login('my-tenant', 'user@example.com', 'password');
 * client.setToken(resp.access_token);
 * const profile = await client.auth.getProfile();
 * ```
 */
export class CoreAuthClient {
  private http: HttpClient;
  public readonly auth: AuthService;
  public readonly oauth2: OAuth2Service;
  public readonly mfa: MfaService;
  public readonly tenants: TenantsService;
  public readonly applications: ApplicationsService;
  public readonly fga: FgaService;
  public readonly audit: AuditService;
  public readonly webhooks: WebhooksService;
  public readonly groups: GroupsService;
  public readonly scim: ScimService;
  public readonly admin: AdminService;
  public readonly connections: ConnectionsService;

  constructor(baseUrl: string, options?: CoreAuthClientOptions) {
    this.http = new HttpClient(baseUrl, options?.token);
    this.auth = new AuthService(this.http);
    this.oauth2 = new OAuth2Service(this.http);
    this.mfa = new MfaService(this.http);
    this.tenants = new TenantsService(this.http);
    this.applications = new ApplicationsService(this.http);
    this.fga = new FgaService(this.http);
    this.audit = new AuditService(this.http);
    this.webhooks = new WebhooksService(this.http);
    this.groups = new GroupsService(this.http);
    this.scim = new ScimService(this.http);
    this.admin = new AdminService(this.http);
    this.connections = new ConnectionsService(this.http);
  }

  setToken(token: string): void {
    this.http.setToken(token);
  }

  clearToken(): void {
    this.http.clearToken();
  }
}
