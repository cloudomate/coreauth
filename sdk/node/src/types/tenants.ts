export interface CreateTenantRequest {
  name: string;
  slug: string;
  admin_email: string;
  admin_password: string;
  admin_full_name?: string;
  account_type?: string;
  isolation_mode?: string;
}

export interface CreateTenantResponse {
  tenant_id: string;
  tenant_name: string;
  admin_user_id: string;
  message: string;
  email_verification_required?: boolean;
  isolation_mode?: string;
  database_setup_required?: boolean;
}

export interface SecuritySettings {
  mfa_required?: boolean;
  password_min_length?: number;
  max_login_attempts?: number;
  lockout_duration_minutes?: number;
  session_timeout_hours?: number;
  require_email_verification?: boolean;
  password_require_uppercase?: boolean;
  password_require_lowercase?: boolean;
  password_require_number?: boolean;
  password_require_special?: boolean;
  enforce_sso?: boolean;
}

export interface BrandingSettings {
  logo_url?: string;
  primary_color?: string;
  favicon_url?: string;
  custom_css?: string;
  app_name?: string;
  background_color?: string;
  terms_url?: string;
  privacy_url?: string;
  support_url?: string;
}

export interface UpdateUserRoleRequest {
  role: string;
}
