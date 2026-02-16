export interface Connection {
  id: string;
  name: string;
  connection_type: string;
  scope: string;
  organization_id?: string;
  config?: Record<string, unknown>;
  is_enabled: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface CreateConnectionRequest {
  name: string;
  connection_type: string;
  config?: Record<string, unknown>;
}

export interface UpdateConnectionRequest {
  name?: string;
  config?: Record<string, unknown>;
  is_enabled?: boolean;
}

export interface AuthMethod {
  connection_id: string;
  name: string;
  method_type: string;
  scope: string;
}
