package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// MfaService provides multi-factor authentication operations.
type MfaService struct {
	http *httpClient
}

// EnrollTOTP initiates TOTP enrollment for the authenticated user.
func (s *MfaService) EnrollTOTP(ctx context.Context) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/mfa/enroll/totp", nil)
}

// VerifyTOTP verifies a TOTP code for the given MFA method.
func (s *MfaService) VerifyTOTP(ctx context.Context, methodID, code string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/mfa/totp/%s/verify", methodID), VerifyMfaRequest{Code: code})
}

// EnrollSMS initiates SMS-based MFA enrollment with the given phone number.
func (s *MfaService) EnrollSMS(ctx context.Context, phoneNumber string) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/mfa/enroll/sms", EnrollSmsRequest{PhoneNumber: phoneNumber})
}

// VerifySMS verifies an SMS code for the given MFA method.
func (s *MfaService) VerifySMS(ctx context.Context, methodID, code string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/mfa/sms/%s/verify", methodID), VerifyMfaRequest{Code: code})
}

// ResendSMS resends the SMS verification code for the given MFA method.
func (s *MfaService) ResendSMS(ctx context.Context, methodID string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/mfa/sms/%s/resend", methodID), nil)
}

// ListMethods returns all MFA methods configured for the authenticated user.
func (s *MfaService) ListMethods(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/mfa/methods", nil)
}

// DeleteMethod removes an MFA method by its ID.
func (s *MfaService) DeleteMethod(ctx context.Context, methodID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/mfa/methods/%s", methodID), nil)
	return err
}

// RegenerateBackupCodes generates a new set of backup codes, replacing any existing ones.
func (s *MfaService) RegenerateBackupCodes(ctx context.Context) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/mfa/backup-codes/regenerate", nil)
}

// EnrollTOTPWithToken initiates TOTP enrollment using an enrollment token (pre-auth flow).
func (s *MfaService) EnrollTOTPWithToken(ctx context.Context, enrollmentToken string) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/mfa/enroll-with-token/totp", EnrollWithTokenRequest{
		EnrollmentToken: enrollmentToken,
	})
}

// VerifyTOTPWithToken verifies a TOTP code using an enrollment token (pre-auth flow).
func (s *MfaService) VerifyTOTPWithToken(ctx context.Context, methodID, enrollmentToken, code string) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/mfa/verify-with-token/totp/%s", methodID), VerifyWithTokenRequest{
		EnrollmentToken: enrollmentToken,
		Code:            code,
	})
}
