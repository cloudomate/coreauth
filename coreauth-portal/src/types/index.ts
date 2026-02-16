export interface User {
  id: string;
  tenant_id?: string;
  email: string;
  email_verified: boolean;
  phone?: string;
  phone_verified: boolean;
  metadata: UserMetadata;
  created_at: string;
}

export interface UserMetadata {
  first_name?: string;
  last_name?: string;
  avatar_url?: string;
  language?: string;
  timezone?: string;
  custom?: Record<string, any>;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user: User;
}

export interface MfaMethod {
  id: string;
  method_type: string;
  is_primary: boolean;
  created_at: string;
}

export interface TenantSecuritySettings {
  mfa_required: boolean;
  mfa_enforcement_date?: string;
  mfa_grace_period_days: number;
  allowed_mfa_methods: string[];
  password_min_length: number;
  password_require_uppercase: boolean;
  password_require_lowercase: boolean;
  password_require_number: boolean;
  password_require_special: boolean;
  max_login_attempts: number;
  lockout_duration_minutes: number;
  session_timeout_hours: number;
}

export interface OidcProvider {
  id: string;
  tenant_id: string;
  provider_name: string;
  client_id: string;
  issuer_url: string;
  authorization_endpoint: string;
  token_endpoint: string;
  userinfo_endpoint: string;
  scopes: string[];
  groups_claim?: string;
  group_role_mappings?: Record<string, string>;
  is_active: boolean;
  created_at: string;
}

export interface OidcProviderTemplate {
  provider_type: string;
  display_name: string;
  authorization_endpoint: string;
  token_endpoint: string;
  userinfo_endpoint?: string;
  jwks_uri: string;
  scopes: string[];
  groups_claim: string;
  issuer_pattern: string;
  instructions: string;
}

export interface Invitation {
  id: string;
  tenant_id: string;
  email: string;
  invited_by: string;
  role_id?: string;
  expires_at: string;
  created_at: string;
  accepted_at?: string;
}
