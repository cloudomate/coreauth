package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// AuditService provides audit log and security event operations.
type AuditService struct {
	http *httpClient
}

// Query retrieves audit logs with optional query parameters for filtering.
func (s *AuditService) Query(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/audit/logs", params)
}

// Get retrieves a specific audit log entry by ID.
func (s *AuditService) Get(ctx context.Context, logID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/audit/logs/%s", logID), nil)
}

// SecurityEvents returns recent security-related events.
func (s *AuditService) SecurityEvents(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/audit/security-events", nil)
}

// FailedLogins returns failed login attempts for a specific user.
func (s *AuditService) FailedLogins(ctx context.Context, userID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/audit/failed-logins/%s", userID), nil)
}

// Export exports audit logs (typically as CSV or JSON).
func (s *AuditService) Export(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/audit/export", nil)
}

// Stats returns aggregate audit statistics.
func (s *AuditService) Stats(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/audit/stats", nil)
}

// LoginHistory returns the authenticated user's login history.
func (s *AuditService) LoginHistory(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/login-history", nil)
}

// SecurityAuditLogs returns security-focused audit logs.
func (s *AuditService) SecurityAuditLogs(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/security/audit-logs", nil)
}
