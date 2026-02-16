export interface Application {
  id: string;
  organization_id?: string;
  name: string;
  slug?: string;
  description?: string;
  app_type?: string;
  client_id: string;
  callback_urls?: string[];
  logout_urls?: string[];
  web_origins?: string[];
  grant_types?: string[];
  allowed_scopes?: string[];
  is_enabled?: boolean;
  is_first_party?: boolean;
  access_token_lifetime_seconds?: number;
  refresh_token_lifetime_seconds?: number;
  created_at?: string;
  updated_at?: string;
}

export interface ApplicationWithSecret extends Application {
  client_secret_plain: string;
}

export interface CreateOAuthAppRequest {
  name: string;
  slug: string;
  app_type: string;
  callback_urls: string[];
  description?: string;
  logo_url?: string;
  logout_urls?: string[];
  web_origins?: string[];
  access_token_lifetime_seconds?: number;
  refresh_token_lifetime_seconds?: number;
  grant_types?: string[];
  allowed_scopes?: string[];
}

export interface UpdateApplicationRequest {
  name?: string;
  description?: string;
  callback_urls?: string[];
  logout_urls?: string[];
  is_enabled?: boolean;
  [key: string]: any;
}

export interface EmailTemplate {
  template_type: string;
  subject?: string;
  html_body?: string;
  text_body?: string;
  variables?: Record<string, any>;
  is_custom?: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface AuthenticateAppRequest {
  client_id: string;
  client_secret: string;
}
