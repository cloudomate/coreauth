export interface TenantRegistryResponse {
  id: string;
  slug: string;
  name: string;
  status?: string;
  isolation_mode?: string;
  created_at?: string;
  updated_at?: string;
}

export interface CreateRegistryTenantRequest {
  slug: string;
  name: string;
  isolation_mode?: string;
}

export interface ConfigureDedicatedDbRequest {
  connection_string: string;
}

export interface TenantRouterStats {
  total_tenants: number;
  active_tenants: number;
  shared_tenants: number;
  dedicated_tenants: number;
}

export interface Action {
  id: string;
  organization_id: string;
  name: string;
  description?: string;
  trigger_type: string;
  code: string;
  runtime?: string;
  timeout_seconds?: number;
  is_enabled: boolean;
  total_executions?: number;
  total_failures?: number;
  last_executed_at?: string;
  created_at?: string;
  updated_at?: string;
}

export interface CreateActionRequest {
  name: string;
  trigger_type: string;
  code: string;
  description?: string;
  runtime?: string;
  timeout_seconds?: number;
  secrets?: Record<string, any>;
  execution_order?: number;
}

export interface UpdateActionRequest {
  name?: string;
  description?: string;
  code?: string;
  runtime?: string;
  timeout_seconds?: number;
  secrets?: Record<string, any>;
  execution_order?: number;
  is_enabled?: boolean;
}

export interface ActionExecution {
  id: string;
  action_id: string;
  organization_id: string;
  trigger_type: string;
  user_id?: string;
  status: string;
  execution_time_ms?: number;
  input_data?: Record<string, any>;
  output_data?: Record<string, any>;
  error_message?: string;
  executed_at?: string;
}

export interface ActionTestResponse {
  success: boolean;
  data?: Record<string, any>;
  error?: string;
}

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  latency_ms?: number;
}

export interface HealthResponse {
  status: string;
  version?: string;
}
