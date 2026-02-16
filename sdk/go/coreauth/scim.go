package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// ScimService provides SCIM 2.0 provisioning, session management, and OIDC provider operations.
type ScimService struct {
	http *httpClient
}

// --- SCIM Configuration ---

// GetConfig retrieves the SCIM service provider configuration.
func (s *ScimService) GetConfig(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/scim/v2/ServiceProviderConfig", nil)
}

// GetResourceTypes retrieves the SCIM resource type definitions.
func (s *ScimService) GetResourceTypes(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/scim/v2/ResourceTypes", nil)
}

// GetSchemas retrieves the SCIM schema definitions.
func (s *ScimService) GetSchemas(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/scim/v2/Schemas", nil)
}

// --- SCIM Users ---

// ListUsers returns SCIM users with optional filtering.
func (s *ScimService) ListUsers(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/scim/v2/Users", params)
}

// CreateUser provisions a new user via SCIM.
func (s *ScimService) CreateUser(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/scim/v2/Users", data)
}

// GetUser retrieves a SCIM user by ID.
func (s *ScimService) GetUser(ctx context.Context, userID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/scim/v2/Users/%s", userID), nil)
}

// ReplaceUser fully replaces a SCIM user (PUT).
func (s *ScimService) ReplaceUser(ctx context.Context, userID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/scim/v2/Users/%s", userID), data)
}

// PatchUser partially updates a SCIM user (PATCH).
func (s *ScimService) PatchUser(ctx context.Context, userID string, data map[string]any) (json.RawMessage, error) {
	return s.http.patch(ctx, fmt.Sprintf("/scim/v2/Users/%s", userID), data)
}

// DeleteUser deprovisions a SCIM user.
func (s *ScimService) DeleteUser(ctx context.Context, userID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/scim/v2/Users/%s", userID), nil)
	return err
}

// --- SCIM Groups ---

// ListScimGroups returns SCIM groups with optional filtering.
func (s *ScimService) ListScimGroups(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/scim/v2/Groups", params)
}

// CreateScimGroup creates a new SCIM group.
func (s *ScimService) CreateScimGroup(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/scim/v2/Groups", data)
}

// GetScimGroup retrieves a SCIM group by ID.
func (s *ScimService) GetScimGroup(ctx context.Context, groupID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/scim/v2/Groups/%s", groupID), nil)
}

// PatchScimGroup partially updates a SCIM group.
func (s *ScimService) PatchScimGroup(ctx context.Context, groupID string, data map[string]any) (json.RawMessage, error) {
	return s.http.patch(ctx, fmt.Sprintf("/scim/v2/Groups/%s", groupID), data)
}

// DeleteScimGroup removes a SCIM group.
func (s *ScimService) DeleteScimGroup(ctx context.Context, groupID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/scim/v2/Groups/%s", groupID), nil)
	return err
}

// --- SCIM Tokens ---

// ListScimTokens returns all SCIM bearer tokens for an organization.
func (s *ScimService) ListScimTokens(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/scim/tokens", orgID), nil)
}

// CreateScimToken creates a new SCIM bearer token for an organization.
func (s *ScimService) CreateScimToken(ctx context.Context, orgID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/scim/tokens", orgID), data)
}

// RevokeScimToken revokes a SCIM bearer token.
func (s *ScimService) RevokeScimToken(ctx context.Context, orgID, tokenID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/organizations/%s/scim/tokens/%s", orgID, tokenID), nil)
	return err
}

// --- Sessions ---

// ListSessions returns all active sessions for the authenticated user.
func (s *ScimService) ListSessions(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/sessions", nil)
}

// RevokeSession revokes a specific session by ID.
func (s *ScimService) RevokeSession(ctx context.Context, sessionID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/sessions/%s", sessionID), nil)
	return err
}

// RevokeAllSessions revokes all sessions for the authenticated user.
func (s *ScimService) RevokeAllSessions(ctx context.Context) error {
	_, err := s.http.del(ctx, "/api/sessions", nil)
	return err
}

// --- OIDC Providers ---

// ListOidcProviders returns all configured OIDC providers for an organization.
func (s *ScimService) ListOidcProviders(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/oidc-providers", orgID), nil)
}

// CreateOidcProvider configures a new OIDC provider for an organization.
func (s *ScimService) CreateOidcProvider(ctx context.Context, orgID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/oidc-providers", orgID), data)
}

// UpdateOidcProvider updates an OIDC provider configuration.
func (s *ScimService) UpdateOidcProvider(ctx context.Context, orgID, providerID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/oidc-providers/%s", orgID, providerID), data)
}

// DeleteOidcProvider removes an OIDC provider configuration.
func (s *ScimService) DeleteOidcProvider(ctx context.Context, orgID, providerID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/organizations/%s/oidc-providers/%s", orgID, providerID), nil)
	return err
}

// ListPublicProviders returns publicly-visible OIDC providers (e.g., for login page display).
func (s *ScimService) ListPublicProviders(ctx context.Context, orgSlug string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/public/oidc-providers/%s", orgSlug), nil)
}

// ListProviderTemplates returns the available OIDC provider templates.
func (s *ScimService) ListProviderTemplates(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/oidc-providers/templates", nil)
}

// GetProviderTemplate retrieves a specific OIDC provider template.
func (s *ScimService) GetProviderTemplate(ctx context.Context, templateName string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/oidc-providers/templates/%s", templateName), nil)
}

// SSOCheck checks if an email domain has SSO configured and returns the provider details.
func (s *ScimService) SSOCheck(ctx context.Context, email string) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/sso/check", map[string]string{"email": email})
}
