package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// ApplicationsService provides application management, OAuth app management,
// and email template operations.
type ApplicationsService struct {
	http *httpClient
}

// --- Authz Applications ---

// Create creates a new authorization application.
func (s *ApplicationsService) Create(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/applications", data)
}

// List returns all authorization applications.
func (s *ApplicationsService) List(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/applications", nil)
}

// Get retrieves an authorization application by ID.
func (s *ApplicationsService) Get(ctx context.Context, appID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/applications/%s", appID), nil)
}

// Update modifies an authorization application.
func (s *ApplicationsService) Update(ctx context.Context, appID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/applications/%s", appID), data)
}

// RotateSecret rotates the client secret for an authorization application.
func (s *ApplicationsService) RotateSecret(ctx context.Context, appID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/applications/%s/rotate-secret", appID), nil)
}

// Delete removes an authorization application.
func (s *ApplicationsService) Delete(ctx context.Context, appID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/applications/%s", appID), nil)
	return err
}

// Authenticate authenticates using application credentials.
func (s *ApplicationsService) Authenticate(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/applications/authenticate", data)
}

// --- OAuth Applications ---

// CreateOAuthApp creates a new OAuth application.
func (s *ApplicationsService) CreateOAuthApp(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/oauth/applications", data)
}

// ListOAuthApps returns all OAuth applications.
func (s *ApplicationsService) ListOAuthApps(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/oauth/applications", nil)
}

// GetOAuthApp retrieves an OAuth application by ID.
func (s *ApplicationsService) GetOAuthApp(ctx context.Context, appID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/oauth/applications/%s", appID), nil)
}

// UpdateOAuthApp modifies an OAuth application.
func (s *ApplicationsService) UpdateOAuthApp(ctx context.Context, appID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/oauth/applications/%s", appID), data)
}

// RotateOAuthSecret rotates the client secret for an OAuth application.
func (s *ApplicationsService) RotateOAuthSecret(ctx context.Context, appID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/oauth/applications/%s/rotate-secret", appID), nil)
}

// DeleteOAuthApp removes an OAuth application.
func (s *ApplicationsService) DeleteOAuthApp(ctx context.Context, appID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/oauth/applications/%s", appID), nil)
	return err
}

// --- Email Templates ---

// ListEmailTemplates returns all email templates for an organization.
func (s *ApplicationsService) ListEmailTemplates(ctx context.Context, orgID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/email-templates", orgID), nil)
}

// GetEmailTemplate retrieves a specific email template.
func (s *ApplicationsService) GetEmailTemplate(ctx context.Context, orgID, templateID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/organizations/%s/email-templates/%s", orgID, templateID), nil)
}

// UpdateEmailTemplate updates an email template.
func (s *ApplicationsService) UpdateEmailTemplate(ctx context.Context, orgID, templateID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/organizations/%s/email-templates/%s", orgID, templateID), data)
}

// DeleteEmailTemplate removes an email template, reverting to the default.
func (s *ApplicationsService) DeleteEmailTemplate(ctx context.Context, orgID, templateID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/organizations/%s/email-templates/%s", orgID, templateID), nil)
	return err
}

// PreviewEmailTemplate renders a preview of an email template.
func (s *ApplicationsService) PreviewEmailTemplate(ctx context.Context, orgID, templateID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/organizations/%s/email-templates/%s/preview", orgID, templateID), data)
}
