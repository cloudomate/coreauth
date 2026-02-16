export interface CreateTupleRequest {
  tenant_id: string;
  namespace: string;
  object_id: string;
  relation: string;
  subject_type: string;
  subject_id: string;
  subject_relation?: string;
}

export interface QueryTuplesRequest {
  tenant_id: string;
  namespace?: string;
  object_id?: string;
  relation?: string;
  subject_type?: string;
  subject_id?: string;
}

export interface RelationTuple {
  id: string;
  tenant_id: string;
  namespace: string;
  object_id: string;
  relation: string;
  subject_type: string;
  subject_id: string;
  subject_relation?: string;
  created_at?: string;
}

export interface CheckRequest {
  tenant_id: string;
  subject_type: string;
  subject_id: string;
  relation: string;
  namespace: string;
  object_id: string;
  context?: Record<string, any>;
}

export interface CheckResponse {
  allowed: boolean;
  reason?: string;
}

export interface ExpandResponse {
  tree: Record<string, any>;
}

export interface ForwardAuthRequest {
  tenant_id: string;
  subject_type: string;
  subject_id: string;
  relation: string;
  namespace: string;
  object_id: string;
}

export interface FgaStore {
  id: string;
  name: string;
  description?: string;
  current_model_version: number;
  is_active: boolean;
  tuple_count: number;
  settings?: Record<string, any>;
  created_at?: string;
  updated_at?: string;
}

export interface CreateStoreRequest {
  name: string;
  description?: string;
}

export interface UpdateStoreRequest {
  name?: string;
  description?: string;
  is_active?: boolean;
}

export interface WriteModelRequest {
  schema: Record<string, any>;
  created_by?: string;
}

export interface AuthorizationModel {
  id: string;
  store_id: string;
  version: number;
  schema_json: Record<string, any>;
  schema_dsl?: string;
  is_valid: boolean;
  validation_errors?: string[];
  created_by?: string;
  created_at?: string;
}

export interface CreateApiKeyRequest {
  name: string;
  permissions: string[];
  rate_limit_per_minute?: number;
  expires_at?: string;
}

export interface FgaStoreApiKey {
  id: string;
  store_id: string;
  name: string;
  key_prefix: string;
  permissions: string[];
  rate_limit_per_minute: number;
  is_active: boolean;
  last_used_at?: string;
  expires_at?: string;
  created_at?: string;
  updated_at?: string;
}

export interface FgaStoreApiKeyWithSecret extends FgaStoreApiKey {
  key: string;
}

export interface WriteTuplesRequest {
  writes?: Record<string, any>[];
  deletes?: Record<string, any>[];
}
