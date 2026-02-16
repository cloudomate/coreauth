package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// AdminService provides administrative operations for tenant registry,
// actions/hooks, rate limits, token claims, and health checks.
type AdminService struct {
	http *httpClient
}

// --- Tenant Registry ---

// ListTenants returns all tenants in the system registry.
func (s *AdminService) ListTenants(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/admin/tenants", nil)
}

// CreateTenant creates a new tenant via the admin API.
func (s *AdminService) CreateTenant(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/admin/tenants", data)
}

// GetStats returns system-wide statistics.
func (s *AdminService) GetStats(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/admin/stats", nil)
}

// GetTenant retrieves a specific tenant by ID from the admin registry.
func (s *AdminService) GetTenant(ctx context.Context, tenantID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/admin/tenants/%s", tenantID), nil)
}

// ConfigureDatabase configures the database connection for an isolated tenant.
func (s *AdminService) ConfigureDatabase(ctx context.Context, tenantID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/admin/tenants/%s/database", tenantID), data)
}

// Activate activates a suspended tenant.
func (s *AdminService) Activate(ctx context.Context, tenantID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/admin/tenants/%s/activate", tenantID), nil)
}

// Suspend suspends an active tenant.
func (s *AdminService) Suspend(ctx context.Context, tenantID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/admin/tenants/%s/suspend", tenantID), nil)
}

// TestConnection tests the database connection for a tenant.
func (s *AdminService) TestConnection(ctx context.Context, tenantID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/admin/tenants/%s/test-connection", tenantID), nil)
}

// --- Actions ---

// CreateAction creates a new action (hook/trigger) for an organization.
func (s *AdminService) CreateAction(ctx context.Context, orgID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/actions", orgID), data)
}

// ListActions returns all actions for an organization.
func (s *AdminService) ListActions(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/actions", orgID), nil)
}

// GetAction retrieves a specific action by ID.
func (s *AdminService) GetAction(ctx context.Context, orgID, actionID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/actions/%s", orgID, actionID), nil)
}

// UpdateAction modifies an existing action.
func (s *AdminService) UpdateAction(ctx context.Context, orgID, actionID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/actions/%s", orgID, actionID), data)
}

// DeleteAction removes an action.
func (s *AdminService) DeleteAction(ctx context.Context, orgID, actionID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/organizations/%s/actions/%s", orgID, actionID), nil)
	return err
}

// TestAction executes an action in test mode.
func (s *AdminService) TestAction(ctx context.Context, orgID, actionID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/actions/%s/test", orgID, actionID), data)
}

// GetActionExecutions returns execution history for a specific action.
func (s *AdminService) GetActionExecutions(ctx context.Context, orgID, actionID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/actions/%s/executions", orgID, actionID), nil)
}

// GetOrgExecutions returns all action executions across an organization.
func (s *AdminService) GetOrgExecutions(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/actions/executions", orgID), nil)
}

// --- Rate Limits ---

// GetRateLimits retrieves the rate limit configuration for an organization.
func (s *AdminService) GetRateLimits(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/rate-limits", orgID), nil)
}

// UpdateRateLimits updates the rate limit configuration for an organization.
func (s *AdminService) UpdateRateLimits(ctx context.Context, orgID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/rate-limits", orgID), data)
}

// --- Token Claims ---

// GetTokenClaims retrieves the custom token claims configuration for an organization.
func (s *AdminService) GetTokenClaims(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/token-claims", orgID), nil)
}

// UpdateTokenClaims updates the custom token claims configuration for an organization.
func (s *AdminService) UpdateTokenClaims(ctx context.Context, orgID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/token-claims", orgID), data)
}

// --- Health ---

// Health checks whether the CoreAuth backend is healthy.
func (s *AdminService) Health(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/health", nil)
}
