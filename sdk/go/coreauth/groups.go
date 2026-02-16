package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// GroupsService provides group management, membership, role assignment, and invitation operations.
type GroupsService struct {
	http *httpClient
}

// --- Groups ---

// Create creates a new group within a tenant.
func (s *GroupsService) Create(ctx context.Context, tenantID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/tenants/%s/groups", tenantID), data)
}

// List returns all groups within a tenant.
func (s *GroupsService) List(ctx context.Context, tenantID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/tenants/%s/groups", tenantID), nil)
}

// Get retrieves a specific group by ID.
func (s *GroupsService) Get(ctx context.Context, tenantID, groupID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s", tenantID, groupID), nil)
}

// Update modifies an existing group.
func (s *GroupsService) Update(ctx context.Context, tenantID, groupID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s", tenantID, groupID), data)
}

// Delete removes a group.
func (s *GroupsService) Delete(ctx context.Context, tenantID, groupID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s", tenantID, groupID), nil)
	return err
}

// --- Members ---

// AddMember adds a user to a group.
func (s *GroupsService) AddMember(ctx context.Context, tenantID, groupID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s/members", tenantID, groupID), data)
}

// ListMembers returns all members of a group.
func (s *GroupsService) ListMembers(ctx context.Context, tenantID, groupID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s/members", tenantID, groupID), nil)
}

// UpdateMember updates a member's attributes within a group.
func (s *GroupsService) UpdateMember(ctx context.Context, tenantID, groupID, userID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s/members/%s", tenantID, groupID, userID), data)
}

// RemoveMember removes a user from a group.
func (s *GroupsService) RemoveMember(ctx context.Context, tenantID, groupID, userID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s/members/%s", tenantID, groupID, userID), nil)
	return err
}

// --- Roles ---

// AssignRole assigns a role to a group.
func (s *GroupsService) AssignRole(ctx context.Context, tenantID, groupID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s/roles", tenantID, groupID), data)
}

// ListRoles returns all roles assigned to a group.
func (s *GroupsService) ListRoles(ctx context.Context, tenantID, groupID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s/roles", tenantID, groupID), nil)
}

// RemoveRole removes a role from a group.
func (s *GroupsService) RemoveRole(ctx context.Context, tenantID, groupID, roleID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/tenants/%s/groups/%s/roles/%s", tenantID, groupID, roleID), nil)
	return err
}

// --- User Groups ---

// GetUserGroups returns all groups a user belongs to within a tenant.
func (s *GroupsService) GetUserGroups(ctx context.Context, tenantID, userID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/tenants/%s/users/%s/groups", tenantID, userID), nil)
}

// --- Invitations ---

// CreateInvitation creates a new invitation to join an organization.
func (s *GroupsService) CreateInvitation(ctx context.Context, orgID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/invitations", orgID), data)
}

// ListInvitations returns all invitations for an organization.
func (s *GroupsService) ListInvitations(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/invitations", orgID), nil)
}

// RevokeInvitation revokes an outstanding invitation.
func (s *GroupsService) RevokeInvitation(ctx context.Context, orgID, invitationID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/organizations/%s/invitations/%s", orgID, invitationID), nil)
	return err
}

// ResendInvitation resends an invitation email.
func (s *GroupsService) ResendInvitation(ctx context.Context, orgID, invitationID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/invitations/%s/resend", orgID, invitationID), nil)
}

// VerifyInvitation validates an invitation token without accepting it.
func (s *GroupsService) VerifyInvitation(ctx context.Context, token string) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/invitations/verify", map[string]string{"token": token})
}

// AcceptInvitation accepts an invitation using its token.
func (s *GroupsService) AcceptInvitation(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/invitations/accept", data)
}
