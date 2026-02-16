export interface AuditLogQuery {
  tenant_id?: string;
  event_types?: string[];
  event_categories?: string[];
  actor_id?: string;
  target_id?: string;
  status?: string;
  from_date?: string;
  to_date?: string;
  limit?: number;
  offset?: number;
}

export interface AuditLog {
  id: string;
  tenant_id: string;
  event_type: string;
  event_category?: string;
  event_action?: string;
  actor_type?: string;
  actor_id?: string;
  actor_name?: string;
  actor_ip_address?: string;
  target_type?: string;
  target_id?: string;
  target_name?: string;
  description?: string;
  metadata?: Record<string, any>;
  status?: string;
  error_message?: string;
  created_at?: string;
}

export interface AuditLogsResponse {
  logs: AuditLog[];
  total: number;
  limit: number;
  offset: number;
}

export interface AuditStats {
  [key: string]: any;
}
