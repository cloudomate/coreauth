package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// ConnectionsService provides connection management operations.
type ConnectionsService struct {
	http *httpClient
}

// List returns all connections for an organization (includes platform connections).
func (s *ConnectionsService) List(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/connections", orgID), nil)
}

// Create creates an organization-scoped connection.
func (s *ConnectionsService) Create(ctx context.Context, orgID string, req CreateConnectionRequest) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/connections", orgID), req)
}

// Get retrieves a specific connection.
func (s *ConnectionsService) Get(ctx context.Context, orgID, connectionID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/connections/%s", orgID, connectionID), nil)
}

// Update updates a connection.
func (s *ConnectionsService) Update(ctx context.Context, orgID, connectionID string, req UpdateConnectionRequest) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/connections/%s", orgID, connectionID), req)
}

// Delete deletes a connection.
func (s *ConnectionsService) Delete(ctx context.Context, orgID, connectionID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/organizations/%s/connections/%s", orgID, connectionID), nil)
	return err
}

// GetAuthMethods returns available authentication methods for an organization.
func (s *ConnectionsService) GetAuthMethods(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/connections/auth-methods", orgID), nil)
}

// ListAll returns all connections (admin).
func (s *ConnectionsService) ListAll(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/admin/connections", nil)
}

// CreatePlatform creates a platform-scoped connection (admin).
func (s *ConnectionsService) CreatePlatform(ctx context.Context, req CreateConnectionRequest) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/admin/connections", req)
}
