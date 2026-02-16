export interface CreateWebhookRequest {
  name: string;
  url: string;
  events: string[];
  is_enabled?: boolean;
  retry_policy?: Record<string, any>;
  custom_headers?: Record<string, string>;
}

export interface UpdateWebhookRequest {
  name?: string;
  url?: string;
  events?: string[];
  is_enabled?: boolean;
  retry_policy?: Record<string, any>;
  custom_headers?: Record<string, string>;
}

export interface WebhookResponse {
  id: string;
  organization_id: string;
  name: string;
  url: string;
  events: string[];
  is_enabled: boolean;
  retry_policy?: Record<string, any>;
  custom_headers?: Record<string, string>;
  total_deliveries?: number;
  successful_deliveries?: number;
  failed_deliveries?: number;
  last_triggered_at?: string;
  created_at?: string;
  updated_at?: string;
}

export interface WebhookWithSecretResponse extends WebhookResponse {
  secret: string;
}

export interface TestWebhookRequest {
  event_type?: string;
}

export interface TestWebhookResponse {
  success: boolean;
  status_code?: number;
  response_time_ms?: number;
  response_body?: string;
  error?: string;
}

export interface WebhookDelivery {
  id: string;
  webhook_id: string;
  event_id: string;
  event_type: string;
  payload?: Record<string, any>;
  status: string;
  response_status?: number;
  response_time_ms?: number;
  attempt_count?: number;
  max_attempts?: number;
  next_retry_at?: string;
  last_error?: string;
  delivered_at?: string;
  failed_at?: string;
  created_at?: string;
}

export interface DeliveryQuery {
  event_type?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

export interface WebhookEventType {
  id: string;
  category: string;
  description: string;
  payload_schema?: Record<string, any>;
  created_at?: string;
}
