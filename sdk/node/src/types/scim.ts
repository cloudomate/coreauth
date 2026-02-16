export interface ScimUser {
  schemas?: string[];
  id: string;
  externalId?: string;
  userName: string;
  name?: Record<string, any>;
  displayName?: string;
  emails?: Record<string, any>[];
  phoneNumbers?: Record<string, any>[];
  active?: boolean;
  groups?: Record<string, any>[];
  meta?: Record<string, any>;
}

export interface CreateScimUserRequest {
  schemas?: string[];
  externalId?: string;
  userName: string;
  name: Record<string, any>;
  displayName?: string;
  emails: Record<string, any>[];
  phoneNumbers?: Record<string, any>[];
  active?: boolean;
  password?: string;
}

export interface ScimGroup {
  schemas?: string[];
  id: string;
  externalId?: string;
  displayName: string;
  members?: Record<string, any>[];
  meta?: Record<string, any>;
}

export interface CreateScimGroupRequest {
  schemas?: string[];
  externalId?: string;
  displayName: string;
  members?: Record<string, any>[];
}

export interface ScimPatchRequest {
  schemas?: string[];
  Operations: { op: string; path?: string; value?: any }[];
}

export interface ScimListResponse {
  schemas?: string[];
  totalResults: number;
  itemsPerPage: number;
  startIndex: number;
  Resources: any[];
}

export interface ScimTokenResponse {
  id: string;
  name: string;
  token_prefix: string;
  expires_at?: string;
  created_at?: string;
}

export interface ScimTokenWithSecret extends ScimTokenResponse {
  secret: string;
}

export interface CreateScimTokenRequest {
  name: string;
  expires_at?: string;
}

export interface SessionInfo {
  id: string;
  user_id?: string;
  ip_address?: string;
  user_agent?: string;
  authenticated_at?: string;
  last_active_at?: string;
  expires_at?: string;
  created_at?: string;
}

export interface OidcProvider {
  id: string;
  tenant_id: string;
  name: string;
  provider_type: string;
  issuer: string;
  client_id: string;
  authorization_endpoint?: string;
  token_endpoint?: string;
  userinfo_endpoint?: string;
  jwks_uri?: string;
  scopes?: string[];
  groups_claim?: string;
  group_role_mappings?: Record<string, any>;
  allowed_group_id?: string;
  is_enabled: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface CreateOidcProviderRequest {
  tenant_id: string;
  name: string;
  provider_type: string;
  issuer: string;
  client_id: string;
  client_secret: string;
  authorization_endpoint: string;
  token_endpoint: string;
  userinfo_endpoint?: string;
  jwks_uri: string;
  scopes?: string[];
  groups_claim?: string;
  group_role_mappings?: Record<string, any>;
  allowed_group_id?: string;
}

export interface UpdateOidcProviderRequest {
  is_enabled?: boolean;
  name?: string;
  client_id?: string;
  client_secret?: string;
}

export interface SsoCheckResponse {
  has_sso: boolean;
  providers: Record<string, any>[];
}
