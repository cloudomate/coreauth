package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// TenantsService provides tenant and organization management operations.
type TenantsService struct {
	http *httpClient
}

// Create creates a new tenant (organization).
func (s *TenantsService) Create(ctx context.Context, req CreateTenantRequest) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/tenants", req)
}

// GetBySlug retrieves an organization by its URL slug.
func (s *TenantsService) GetBySlug(ctx context.Context, slug string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/by-slug/%s", slug), nil)
}

// ListUsers returns all users belonging to a tenant.
func (s *TenantsService) ListUsers(ctx context.Context, tenantID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/tenants/%s/users", tenantID), nil)
}

// UpdateUserRole updates a user's role within a tenant.
func (s *TenantsService) UpdateUserRole(ctx context.Context, tenantID, userID, role string) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/tenants/%s/users/%s/role", tenantID, userID), UpdateUserRoleRequest{Role: role})
}

// GetSecurity retrieves the security settings for an organization.
func (s *TenantsService) GetSecurity(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/security", orgID), nil)
}

// UpdateSecurity updates the security settings for an organization.
func (s *TenantsService) UpdateSecurity(ctx context.Context, orgID string, req SecuritySettings) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/security", orgID), req)
}

// GetBranding retrieves the branding settings for an organization.
func (s *TenantsService) GetBranding(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/branding", orgID), nil)
}

// UpdateBranding updates the branding settings for an organization.
func (s *TenantsService) UpdateBranding(ctx context.Context, orgID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/branding", orgID), data)
}
