export interface RegisterRequest {
  tenant_id: string;
  email: string;
  password: string;
  phone?: string;
}

export interface LoginRequest {
  tenant_id: string;
  email: string;
  password: string;
}

export interface HierarchicalLoginRequest {
  email: string;
  password: string;
  organization_slug?: string;
}

export interface RefreshTokenRequest {
  refresh_token: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  token_type: string;
  expires_in: number;
  user?: Record<string, any>;
  mfa_required?: boolean;
  mfa_token?: string;
}

export interface UserProfile {
  id: string;
  email: string;
  email_verified?: boolean;
  phone?: string;
  phone_verified?: boolean;
  metadata?: Record<string, any>;
  is_active?: boolean;
  mfa_enabled?: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface UpdateProfileRequest {
  first_name?: string;
  last_name?: string;
  full_name?: string;
  phone?: string;
  avatar_url?: string;
  language?: string;
  timezone?: string;
}

export interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}

export interface PasswordlessStartRequest {
  method: string;
  email: string;
}

export interface PasswordlessStartResponse {
  message: string;
  expires_in?: number;
}

export interface PasswordlessVerifyRequest {
  token_or_code: string;
}

export interface FlowResponse {
  id: string;
  type?: string;
  status?: string;
  ui?: Record<string, any>;
  created_at?: string;
  expires_at?: string;
  request_url?: string;
}

export interface SessionResponse {
  id: string;
  active: boolean;
  identity?: Record<string, any>;
  authenticated_at?: string;
  expires_at?: string;
}
