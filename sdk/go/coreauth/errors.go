package coreauth

import "fmt"

// CoreAuthError is the base error type for SDK errors.
type CoreAuthError struct {
	Message string
}

func (e *CoreAuthError) Error() string {
	return e.Message
}

// ApiError represents a non-2xx API response.
type ApiError struct {
	StatusCode int    `json:"status_code"`
	ErrorCode  string `json:"error"`
	Message    string `json:"message"`
}

func (e *ApiError) Error() string {
	return fmt.Sprintf("[%d] %s: %s", e.StatusCode, e.ErrorCode, e.Message)
}

// IsNotFound returns true if the error is a 404.
func IsNotFound(err error) bool {
	if e, ok := err.(*ApiError); ok {
		return e.StatusCode == 404
	}
	return false
}

// IsUnauthorized returns true if the error is a 401.
func IsUnauthorized(err error) bool {
	if e, ok := err.(*ApiError); ok {
		return e.StatusCode == 401
	}
	return false
}

// IsForbidden returns true if the error is a 403.
func IsForbidden(err error) bool {
	if e, ok := err.(*ApiError); ok {
		return e.StatusCode == 403
	}
	return false
}

// IsConflict returns true if the error is a 409.
func IsConflict(err error) bool {
	if e, ok := err.(*ApiError); ok {
		return e.StatusCode == 409
	}
	return false
}

// IsRateLimited returns true if the error is a 429.
func IsRateLimited(err error) bool {
	if e, ok := err.(*ApiError); ok {
		return e.StatusCode == 429
	}
	return false
}
