package coreauth

// MfaEnrollResponse represents the response from enrolling in MFA (TOTP).
type MfaEnrollResponse struct {
	MethodID   string   `json:"method_id"`
	MethodType string   `json:"method_type"`
	Secret     *string  `json:"secret,omitempty"`
	QrCodeURI  *string  `json:"qr_code_uri,omitempty"`
	BackupCodes []string `json:"backup_codes,omitempty"`
}

// SmsMfaEnrollResponse represents the response from enrolling in SMS-based MFA.
type SmsMfaEnrollResponse struct {
	MethodID    string  `json:"method_id"`
	MethodType  string  `json:"method_type"`
	PhoneNumber string  `json:"phone_number"`
	MaskedPhone string  `json:"masked_phone"`
	ExpiresAt   *string `json:"expires_at,omitempty"`
}

// VerifyMfaRequest represents a request to verify an MFA code.
type VerifyMfaRequest struct {
	Code string `json:"code"`
}

// EnrollSmsRequest represents a request to enroll in SMS-based MFA.
type EnrollSmsRequest struct {
	PhoneNumber string `json:"phone_number"`
}

// EnrollWithTokenRequest represents a request to enroll in MFA using an enrollment token.
type EnrollWithTokenRequest struct {
	EnrollmentToken string `json:"enrollment_token"`
}

// VerifyWithTokenRequest represents a request to verify MFA using an enrollment token and code.
type VerifyWithTokenRequest struct {
	EnrollmentToken string `json:"enrollment_token"`
	Code            string `json:"code"`
}

// MfaMethod represents an MFA method configured for a user.
type MfaMethod struct {
	ID         string  `json:"id"`
	UserID     string  `json:"user_id"`
	MethodType string  `json:"method_type"`
	Verified   bool    `json:"verified"`
	Name       *string `json:"name,omitempty"`
	CreatedAt  *string `json:"created_at,omitempty"`
	LastUsedAt *string `json:"last_used_at,omitempty"`
}
