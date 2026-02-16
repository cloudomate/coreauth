export interface MfaEnrollResponse {
  method_id: string;
  method_type: string;
  secret?: string;
  qr_code_uri?: string;
  backup_codes?: string[];
}

export interface SmsMfaEnrollResponse {
  method_id: string;
  method_type: string;
  phone_number: string;
  masked_phone: string;
  expires_at?: string;
}

export interface VerifyMfaRequest {
  code: string;
}

export interface EnrollSmsRequest {
  phone_number: string;
}

export interface EnrollWithTokenRequest {
  enrollment_token: string;
}

export interface VerifyWithTokenRequest {
  enrollment_token: string;
  code: string;
}

export interface MfaMethod {
  id: string;
  user_id: string;
  method_type: string;
  verified: boolean;
  name?: string;
  created_at?: string;
  last_used_at?: string;
}
