package coreauth

// CreateTenantRequest represents a request to create a new tenant.
type CreateTenantRequest struct {
	Name          string  `json:"name"`
	Slug          string  `json:"slug"`
	AdminEmail    string  `json:"admin_email"`
	AdminPassword string  `json:"admin_password"`
	AdminFullName *string `json:"admin_full_name,omitempty"`
	AccountType   *string `json:"account_type,omitempty"`
	IsolationMode *string `json:"isolation_mode,omitempty"`
}

// CreateTenantResponse represents the response from creating a tenant.
type CreateTenantResponse struct {
	TenantID                  string  `json:"tenant_id"`
	TenantName                string  `json:"tenant_name"`
	AdminUserID               string  `json:"admin_user_id"`
	Message                   string  `json:"message"`
	EmailVerificationRequired *bool   `json:"email_verification_required,omitempty"`
	IsolationMode             *string `json:"isolation_mode,omitempty"`
	DatabaseSetupRequired     *bool   `json:"database_setup_required,omitempty"`
}

// SecuritySettings represents tenant security configuration.
type SecuritySettings struct {
	MfaRequired              *bool `json:"mfa_required,omitempty"`
	PasswordMinLength        *int  `json:"password_min_length,omitempty"`
	MaxLoginAttempts         *int  `json:"max_login_attempts,omitempty"`
	LockoutDurationMinutes   *int  `json:"lockout_duration_minutes,omitempty"`
	SessionTimeoutHours      *int  `json:"session_timeout_hours,omitempty"`
	RequireEmailVerification *bool `json:"require_email_verification,omitempty"`
	PasswordRequireUppercase *bool `json:"password_require_uppercase,omitempty"`
	PasswordRequireLowercase *bool `json:"password_require_lowercase,omitempty"`
	PasswordRequireNumber    *bool `json:"password_require_number,omitempty"`
	PasswordRequireSpecial   *bool `json:"password_require_special,omitempty"`
	EnforceSSO               *bool `json:"enforce_sso,omitempty"`
}

// BrandingSettings represents tenant branding configuration.
type BrandingSettings struct {
	LogoURL         *string `json:"logo_url,omitempty"`
	PrimaryColor    *string `json:"primary_color,omitempty"`
	FaviconURL      *string `json:"favicon_url,omitempty"`
	CustomCSS       *string `json:"custom_css,omitempty"`
	AppName         *string `json:"app_name,omitempty"`
	BackgroundColor *string `json:"background_color,omitempty"`
	TermsURL        *string `json:"terms_url,omitempty"`
	PrivacyURL      *string `json:"privacy_url,omitempty"`
	SupportURL      *string `json:"support_url,omitempty"`
}

// UpdateUserRoleRequest represents a request to update a user's role.
type UpdateUserRoleRequest struct {
	Role string `json:"role"`
}
