package coreauth

// CreateWebhookRequest represents a request to create a webhook.
type CreateWebhookRequest struct {
	Name          string            `json:"name"`
	URL           string            `json:"url"`
	Events        []string          `json:"events"`
	IsEnabled     bool              `json:"is_enabled"`
	RetryPolicy   map[string]any    `json:"retry_policy,omitempty"`
	CustomHeaders map[string]string `json:"custom_headers,omitempty"`
}

// UpdateWebhookRequest represents a request to update a webhook.
type UpdateWebhookRequest struct {
	Name          *string            `json:"name,omitempty"`
	URL           *string            `json:"url,omitempty"`
	Events        []string           `json:"events,omitempty"`
	IsEnabled     *bool              `json:"is_enabled,omitempty"`
	RetryPolicy   map[string]any     `json:"retry_policy,omitempty"`
	CustomHeaders map[string]string  `json:"custom_headers,omitempty"`
}

// WebhookResponse represents a webhook configuration.
type WebhookResponse struct {
	ID                    string            `json:"id"`
	OrganizationID        string            `json:"organization_id"`
	Name                  string            `json:"name"`
	URL                   string            `json:"url"`
	Events                []string          `json:"events"`
	IsEnabled             bool              `json:"is_enabled"`
	RetryPolicy           map[string]any    `json:"retry_policy,omitempty"`
	CustomHeaders         map[string]string `json:"custom_headers,omitempty"`
	TotalDeliveries       *int64            `json:"total_deliveries,omitempty"`
	SuccessfulDeliveries  *int64            `json:"successful_deliveries,omitempty"`
	FailedDeliveries      *int64            `json:"failed_deliveries,omitempty"`
	LastTriggeredAt       *string           `json:"last_triggered_at,omitempty"`
	CreatedAt             *string           `json:"created_at,omitempty"`
	UpdatedAt             *string           `json:"updated_at,omitempty"`
}

// WebhookWithSecretResponse represents a webhook with its signing secret exposed.
type WebhookWithSecretResponse struct {
	WebhookResponse
	Secret string `json:"secret"`
}

// TestWebhookRequest represents a request to test a webhook.
type TestWebhookRequest struct {
	EventType *string `json:"event_type,omitempty"`
}

// TestWebhookResponse represents the result of a webhook test.
type TestWebhookResponse struct {
	Success        bool    `json:"success"`
	StatusCode     *int    `json:"status_code,omitempty"`
	ResponseTimeMs *int64  `json:"response_time_ms,omitempty"`
	ResponseBody   *string `json:"response_body,omitempty"`
	Error          *string `json:"error,omitempty"`
}

// WebhookDelivery represents a single webhook delivery attempt.
type WebhookDelivery struct {
	ID             string         `json:"id"`
	WebhookID      string         `json:"webhook_id"`
	EventID        string         `json:"event_id"`
	EventType      string         `json:"event_type"`
	Payload        map[string]any `json:"payload,omitempty"`
	Status         string         `json:"status"`
	ResponseStatus *int           `json:"response_status,omitempty"`
	ResponseTimeMs *int64         `json:"response_time_ms,omitempty"`
	AttemptCount   *int           `json:"attempt_count,omitempty"`
	MaxAttempts    *int           `json:"max_attempts,omitempty"`
	NextRetryAt    *string        `json:"next_retry_at,omitempty"`
	LastError      *string        `json:"last_error,omitempty"`
	DeliveredAt    *string        `json:"delivered_at,omitempty"`
	FailedAt       *string        `json:"failed_at,omitempty"`
	CreatedAt      *string        `json:"created_at,omitempty"`
}

// WebhookEventType represents a supported webhook event type.
type WebhookEventType struct {
	ID            string         `json:"id"`
	Category      string         `json:"category"`
	Description   string         `json:"description"`
	PayloadSchema map[string]any `json:"payload_schema,omitempty"`
	CreatedAt     *string        `json:"created_at,omitempty"`
}
