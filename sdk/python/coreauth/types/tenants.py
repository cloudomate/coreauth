from __future__ import annotations

from typing import Optional

from pydantic import BaseModel


class CreateTenantRequest(BaseModel):
    name: str
    slug: str
    admin_email: str
    admin_password: str
    admin_full_name: Optional[str] = None
    account_type: Optional[str] = None
    isolation_mode: Optional[str] = None


class CreateTenantResponse(BaseModel):
    tenant_id: Optional[str] = None
    tenant_name: Optional[str] = None
    admin_user_id: Optional[str] = None
    message: Optional[str] = None
    email_verification_required: Optional[bool] = None
    isolation_mode: Optional[str] = None
    database_setup_required: Optional[bool] = None


class SecuritySettings(BaseModel):
    mfa_required: Optional[bool] = None
    password_min_length: Optional[int] = None
    max_login_attempts: Optional[int] = None
    lockout_duration_minutes: Optional[int] = None
    session_timeout_hours: Optional[int] = None
    require_email_verification: Optional[bool] = None
    password_require_uppercase: Optional[bool] = None
    password_require_lowercase: Optional[bool] = None
    password_require_number: Optional[bool] = None
    password_require_special: Optional[bool] = None
    enforce_sso: Optional[bool] = None


class UpdateSecuritySettingsRequest(BaseModel):
    mfa_required: Optional[bool] = None
    password_min_length: Optional[int] = None
    max_login_attempts: Optional[int] = None
    lockout_duration_minutes: Optional[int] = None
    session_timeout_hours: Optional[int] = None
    require_email_verification: Optional[bool] = None
    password_require_uppercase: Optional[bool] = None
    password_require_lowercase: Optional[bool] = None
    password_require_number: Optional[bool] = None
    password_require_special: Optional[bool] = None
    enforce_sso: Optional[bool] = None


class BrandingSettings(BaseModel):
    logo_url: Optional[str] = None
    primary_color: Optional[str] = None
    favicon_url: Optional[str] = None
    custom_css: Optional[str] = None
    app_name: Optional[str] = None
    background_color: Optional[str] = None
    terms_url: Optional[str] = None
    privacy_url: Optional[str] = None
    support_url: Optional[str] = None


class UpdateBrandingRequest(BaseModel):
    logo_url: Optional[str] = None
    primary_color: Optional[str] = None
    favicon_url: Optional[str] = None
    custom_css: Optional[str] = None
    app_name: Optional[str] = None
    background_color: Optional[str] = None
    terms_url: Optional[str] = None
    privacy_url: Optional[str] = None
    support_url: Optional[str] = None


class UpdateUserRoleRequest(BaseModel):
    role: str
