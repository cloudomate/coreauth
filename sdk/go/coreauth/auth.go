package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// AuthService provides authentication and self-service identity flows.
type AuthService struct {
	http *httpClient
}

// Register creates a new user account.
func (s *AuthService) Register(ctx context.Context, req RegisterRequest) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/auth/register", req)
}

// Login authenticates a user with email and password.
func (s *AuthService) Login(ctx context.Context, req LoginRequest) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/auth/login", req)
}

// LoginHierarchical authenticates a user with optional organization context.
func (s *AuthService) LoginHierarchical(ctx context.Context, req HierarchicalLoginRequest) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/auth/login-hierarchical", req)
}

// RefreshToken exchanges a refresh token for a new access token.
func (s *AuthService) RefreshToken(ctx context.Context, refreshToken string) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/auth/refresh", map[string]string{"refresh_token": refreshToken})
}

// Logout invalidates the current session.
func (s *AuthService) Logout(ctx context.Context) error {
	_, err := s.http.post(ctx, "/api/auth/logout", nil)
	return err
}

// GetProfile retrieves the authenticated user's profile.
func (s *AuthService) GetProfile(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/auth/me", nil)
}

// UpdateProfile updates the authenticated user's profile.
func (s *AuthService) UpdateProfile(ctx context.Context, req UpdateProfileRequest) (json.RawMessage, error) {
	return s.http.patch(ctx, "/api/auth/me", req)
}

// ChangePassword changes the authenticated user's password.
func (s *AuthService) ChangePassword(ctx context.Context, req ChangePasswordRequest) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/auth/change-password", req)
}

// VerifyEmail verifies a user's email address using a verification token.
func (s *AuthService) VerifyEmail(ctx context.Context, token string) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/verify-email", map[string]string{"token": token})
}

// ResendVerification resends the email verification message.
func (s *AuthService) ResendVerification(ctx context.Context) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/auth/resend-verification", nil)
}

// ForgotPassword initiates a password reset flow by sending a reset email.
func (s *AuthService) ForgotPassword(ctx context.Context, tenantID, email string) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/auth/forgot-password", map[string]string{
		"tenant_id": tenantID,
		"email":     email,
	})
}

// VerifyResetToken validates a password reset token.
func (s *AuthService) VerifyResetToken(ctx context.Context, token string) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/auth/verify-reset-token", map[string]string{"token": token})
}

// ResetPassword sets a new password using a valid reset token.
func (s *AuthService) ResetPassword(ctx context.Context, token, newPassword string) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/auth/reset-password", map[string]string{
		"token":        token,
		"new_password": newPassword,
	})
}

// PasswordlessStart initiates a passwordless authentication flow.
func (s *AuthService) PasswordlessStart(ctx context.Context, tenantID string, req PasswordlessStartRequest) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/tenants/%s/passwordless/start", tenantID), req)
}

// PasswordlessVerify completes a passwordless authentication flow.
func (s *AuthService) PasswordlessVerify(ctx context.Context, tenantID string, req PasswordlessVerifyRequest) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/tenants/%s/passwordless/verify", tenantID), req)
}

// PasswordlessResend resends a passwordless authentication code.
func (s *AuthService) PasswordlessResend(ctx context.Context, tenantID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/tenants/%s/passwordless/resend", tenantID), data)
}

// CreateLoginFlowBrowser creates a browser-based login flow.
func (s *AuthService) CreateLoginFlowBrowser(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/self-service/login/browser", params)
}

// CreateLoginFlowAPI creates an API-based login flow.
func (s *AuthService) CreateLoginFlowAPI(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/self-service/login/api", params)
}

// GetLoginFlow retrieves a login flow by its ID.
func (s *AuthService) GetLoginFlow(ctx context.Context, flowID string) (json.RawMessage, error) {
	return s.http.get(ctx, "/self-service/login", map[string]string{"flow": flowID})
}

// SubmitLoginFlow submits credentials to a login flow.
func (s *AuthService) SubmitLoginFlow(ctx context.Context, flowID string, data map[string]any) (json.RawMessage, error) {
	if data == nil {
		data = map[string]any{}
	}
	data["flow"] = flowID
	return s.http.post(ctx, "/self-service/login", data)
}

// CreateRegistrationFlowBrowser creates a browser-based registration flow.
func (s *AuthService) CreateRegistrationFlowBrowser(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/self-service/registration/browser", params)
}

// CreateRegistrationFlowAPI creates an API-based registration flow.
func (s *AuthService) CreateRegistrationFlowAPI(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/self-service/registration/api", params)
}

// GetRegistrationFlow retrieves a registration flow by its ID.
func (s *AuthService) GetRegistrationFlow(ctx context.Context, flowID string) (json.RawMessage, error) {
	return s.http.get(ctx, "/self-service/registration", map[string]string{"flow": flowID})
}

// SubmitRegistrationFlow submits data to a registration flow.
func (s *AuthService) SubmitRegistrationFlow(ctx context.Context, flowID string, data map[string]any) (json.RawMessage, error) {
	if data == nil {
		data = map[string]any{}
	}
	data["flow"] = flowID
	return s.http.post(ctx, "/self-service/registration", data)
}

// Whoami returns the current session information.
func (s *AuthService) Whoami(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/sessions/whoami", nil)
}
