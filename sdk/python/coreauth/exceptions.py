"""CoreAuth SDK exceptions."""

from __future__ import annotations


class CoreAuthError(Exception):
    """Base exception for all CoreAuth SDK errors."""


class ApiError(CoreAuthError):
    """API returned a non-2xx response."""

    def __init__(self, status_code: int, error: str = "", message: str = "") -> None:
        self.status_code = status_code
        self.error = error
        self.message = message
        super().__init__(f"[{status_code}] {error}: {message}" if error else f"[{status_code}] {message}")


class AuthenticationError(ApiError):
    """401 Unauthorized."""

    def __init__(self, error: str = "unauthorized", message: str = "Authentication required") -> None:
        super().__init__(401, error, message)


class ForbiddenError(ApiError):
    """403 Forbidden."""

    def __init__(self, error: str = "forbidden", message: str = "Insufficient permissions") -> None:
        super().__init__(403, error, message)


class NotFoundError(ApiError):
    """404 Not Found."""

    def __init__(self, error: str = "not_found", message: str = "Resource not found") -> None:
        super().__init__(404, error, message)


class ConflictError(ApiError):
    """409 Conflict."""

    def __init__(self, error: str = "conflict", message: str = "Resource already exists") -> None:
        super().__init__(409, error, message)


class ValidationError(ApiError):
    """400 Bad Request."""

    def __init__(self, error: str = "validation_error", message: str = "Invalid request") -> None:
        super().__init__(400, error, message)


class RateLimitError(ApiError):
    """429 Too Many Requests."""

    def __init__(self, error: str = "rate_limited", message: str = "Rate limit exceeded") -> None:
        super().__init__(429, error, message)
