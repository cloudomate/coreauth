package coreauth

// RegisterRequest represents a user registration request.
type RegisterRequest struct {
	TenantID string  `json:"tenant_id"`
	Email    string  `json:"email"`
	Password string  `json:"password"`
	Phone    *string `json:"phone,omitempty"`
}

// LoginRequest represents a user login request.
type LoginRequest struct {
	TenantID string `json:"tenant_id"`
	Email    string `json:"email"`
	Password string `json:"password"`
}

// HierarchicalLoginRequest represents a login request with optional organization context.
type HierarchicalLoginRequest struct {
	Email            string  `json:"email"`
	Password         string  `json:"password"`
	OrganizationSlug *string `json:"organization_slug,omitempty"`
}

// RefreshTokenRequest represents a token refresh request.
type RefreshTokenRequest struct {
	RefreshToken string `json:"refresh_token"`
}

// AuthResponse represents the response from authentication endpoints.
type AuthResponse struct {
	AccessToken  string         `json:"access_token"`
	RefreshToken string         `json:"refresh_token"`
	TokenType    string         `json:"token_type"`
	ExpiresIn    int            `json:"expires_in"`
	User         map[string]any `json:"user,omitempty"`
	MfaRequired  *bool          `json:"mfa_required,omitempty"`
	MfaToken     *string        `json:"mfa_token,omitempty"`
}

// UserProfile represents a user's profile information.
type UserProfile struct {
	ID            string         `json:"id"`
	Email         string         `json:"email"`
	EmailVerified *bool          `json:"email_verified,omitempty"`
	Phone         *string        `json:"phone,omitempty"`
	PhoneVerified *bool          `json:"phone_verified,omitempty"`
	Metadata      map[string]any `json:"metadata,omitempty"`
	IsActive      *bool          `json:"is_active,omitempty"`
	MfaEnabled    *bool          `json:"mfa_enabled,omitempty"`
	CreatedAt     *string        `json:"created_at,omitempty"`
	UpdatedAt     *string        `json:"updated_at,omitempty"`
}

// UpdateProfileRequest represents a request to update a user's profile.
type UpdateProfileRequest struct {
	FirstName *string `json:"first_name,omitempty"`
	LastName  *string `json:"last_name,omitempty"`
	FullName  *string `json:"full_name,omitempty"`
	Phone     *string `json:"phone,omitempty"`
	AvatarURL *string `json:"avatar_url,omitempty"`
	Language  *string `json:"language,omitempty"`
	Timezone  *string `json:"timezone,omitempty"`
}

// ChangePasswordRequest represents a request to change a user's password.
type ChangePasswordRequest struct {
	CurrentPassword string `json:"current_password"`
	NewPassword     string `json:"new_password"`
}

// PasswordlessStartRequest represents a request to start passwordless authentication.
type PasswordlessStartRequest struct {
	Method string `json:"method"`
	Email  string `json:"email"`
}

// PasswordlessStartResponse represents the response from starting passwordless authentication.
type PasswordlessStartResponse struct {
	Message   string `json:"message"`
	ExpiresIn *int   `json:"expires_in,omitempty"`
}

// PasswordlessVerifyRequest represents a request to verify a passwordless authentication code or token.
type PasswordlessVerifyRequest struct {
	TokenOrCode string `json:"token_or_code"`
}
