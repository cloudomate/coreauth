package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// WebhooksService provides webhook management and delivery operations.
type WebhooksService struct {
	http *httpClient
}

// Create creates a new webhook for an organization.
func (s *WebhooksService) Create(ctx context.Context, orgID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/webhooks", orgID), data)
}

// List returns all webhooks for an organization.
func (s *WebhooksService) List(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/webhooks", orgID), nil)
}

// Get retrieves a specific webhook by ID.
func (s *WebhooksService) Get(ctx context.Context, orgID, webhookID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/webhooks/%s", orgID, webhookID), nil)
}

// Update modifies an existing webhook.
func (s *WebhooksService) Update(ctx context.Context, orgID, webhookID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/webhooks/%s", orgID, webhookID), data)
}

// Delete removes a webhook.
func (s *WebhooksService) Delete(ctx context.Context, orgID, webhookID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/organizations/%s/webhooks/%s", orgID, webhookID), nil)
	return err
}

// RotateSecret rotates the signing secret for a webhook.
func (s *WebhooksService) RotateSecret(ctx context.Context, orgID, webhookID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/webhooks/%s/rotate-secret", orgID, webhookID), nil)
}

// Test sends a test event to a webhook endpoint.
func (s *WebhooksService) Test(ctx context.Context, orgID, webhookID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/webhooks/%s/test", orgID, webhookID), nil)
}

// ListDeliveries returns delivery attempts for a webhook.
func (s *WebhooksService) ListDeliveries(ctx context.Context, orgID, webhookID string, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/webhooks/%s/deliveries", orgID, webhookID), params)
}

// GetDelivery retrieves a specific webhook delivery attempt.
func (s *WebhooksService) GetDelivery(ctx context.Context, orgID, webhookID, deliveryID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/webhooks/%s/deliveries/%s", orgID, webhookID, deliveryID), nil)
}

// RetryDelivery retries a failed webhook delivery.
func (s *WebhooksService) RetryDelivery(ctx context.Context, orgID, webhookID, deliveryID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/webhooks/%s/deliveries/%s/retry", orgID, webhookID, deliveryID), nil)
}

// ListEventTypes returns all available webhook event types.
func (s *WebhooksService) ListEventTypes(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/webhooks/event-types", nil)
}
