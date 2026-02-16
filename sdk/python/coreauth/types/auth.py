from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class RegisterRequest(BaseModel):
    tenant_id: str
    email: str
    password: str
    phone: Optional[str] = None


class LoginRequest(BaseModel):
    tenant_id: str
    email: str
    password: str


class HierarchicalLoginRequest(BaseModel):
    email: str
    password: str
    organization_slug: Optional[str] = None


class RefreshTokenRequest(BaseModel):
    refresh_token: str


class AuthResponse(BaseModel):
    access_token: str
    refresh_token: str
    token_type: str
    expires_in: int
    user: Optional[dict[str, Any]] = None
    mfa_required: Optional[bool] = None
    mfa_token: Optional[str] = None


class UserProfile(BaseModel):
    id: str
    email: str
    email_verified: Optional[str] = None
    phone: Optional[str] = None
    phone_verified: Optional[str] = None
    metadata: Optional[str] = None
    is_active: Optional[str] = None
    mfa_enabled: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class UpdateProfileRequest(BaseModel):
    first_name: Optional[str] = None
    last_name: Optional[str] = None
    full_name: Optional[str] = None
    phone: Optional[str] = None
    avatar_url: Optional[str] = None
    language: Optional[str] = None
    timezone: Optional[str] = None


class ChangePasswordRequest(BaseModel):
    current_password: str
    new_password: str


class PasswordlessStartRequest(BaseModel):
    method: str
    email: str


class PasswordlessStartResponse(BaseModel):
    message: str
    expires_in: Optional[int] = None


class PasswordlessVerifyRequest(BaseModel):
    token_or_code: str


class PasswordlessVerifyResponse(BaseModel):
    access_token: Optional[str] = None
    refresh_token: Optional[str] = None
    token_type: Optional[str] = None
    expires_in: Optional[int] = None
    user: Optional[dict[str, Any]] = None
    mfa_required: Optional[bool] = None
    mfa_token: Optional[str] = None


class FlowResponse(BaseModel):
    id: str
    type: Optional[str] = None
    status: Optional[str] = None
    ui: Optional[dict[str, Any]] = None
    created_at: Optional[str] = None
    expires_at: Optional[str] = None
    request_url: Optional[str] = None


class SessionResponse(BaseModel):
    id: str
    active: bool
    identity: Optional[dict[str, Any]] = None
    authenticated_at: Optional[str] = None
    expires_at: Optional[str] = None
