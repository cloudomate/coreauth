from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class CreateApplicationRequest(BaseModel):
    tenant_id: str
    name: str
    description: Optional[str] = None
    application_type: Optional[str] = None
    redirect_uris: list[str] = []
    allowed_scopes: list[str] = []
    metadata: Optional[dict[str, Any]] = None


class Application(BaseModel):
    id: Optional[str] = None
    tenant_id: Optional[str] = None
    organization_id: Optional[str] = None
    name: Optional[str] = None
    slug: Optional[str] = None
    description: Optional[str] = None
    logo_url: Optional[str] = None
    app_type: Optional[str] = None
    client_id: Optional[str] = None
    callback_urls: Optional[list[str]] = None
    logout_urls: Optional[list[str]] = None
    web_origins: Optional[list[str]] = None
    grant_types: Optional[list[str]] = None
    response_types: Optional[list[str]] = None
    allowed_scopes: Optional[list[str]] = None
    is_enabled: Optional[bool] = None
    is_first_party: Optional[bool] = None
    access_token_lifetime_seconds: Optional[int] = None
    refresh_token_lifetime_seconds: Optional[int] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class ApplicationWithSecret(BaseModel):
    id: Optional[str] = None
    tenant_id: Optional[str] = None
    organization_id: Optional[str] = None
    name: Optional[str] = None
    slug: Optional[str] = None
    description: Optional[str] = None
    logo_url: Optional[str] = None
    app_type: Optional[str] = None
    client_id: Optional[str] = None
    callback_urls: Optional[list[str]] = None
    logout_urls: Optional[list[str]] = None
    web_origins: Optional[list[str]] = None
    grant_types: Optional[list[str]] = None
    response_types: Optional[list[str]] = None
    allowed_scopes: Optional[list[str]] = None
    is_enabled: Optional[bool] = None
    is_first_party: Optional[bool] = None
    access_token_lifetime_seconds: Optional[int] = None
    refresh_token_lifetime_seconds: Optional[int] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None
    client_secret_plain: Optional[str] = None


class CreateOAuthAppRequest(BaseModel):
    name: str
    slug: Optional[str] = None
    description: Optional[str] = None
    logo_url: Optional[str] = None
    app_type: Optional[str] = None
    callback_urls: list[str] = []
    logout_urls: Optional[list[str]] = None
    web_origins: Optional[list[str]] = None
    access_token_lifetime_seconds: Optional[int] = None
    refresh_token_lifetime_seconds: Optional[int] = None
    grant_types: Optional[list[str]] = None
    allowed_scopes: Optional[list[str]] = None
    organization_id: Optional[str] = None


class UpdateApplicationRequest(BaseModel):
    name: Optional[str] = None
    description: Optional[str] = None
    redirect_uris: Optional[list[str]] = None
    allowed_scopes: Optional[list[str]] = None
    is_active: Optional[bool] = None
    metadata: Optional[dict[str, Any]] = None


class EmailTemplate(BaseModel):
    template_type: Optional[str] = None
    subject: Optional[str] = None
    html_body: Optional[str] = None
    text_body: Optional[str] = None
    variables: Optional[list[str]] = None
    is_custom: Optional[bool] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class UpdateEmailTemplateRequest(BaseModel):
    subject: Optional[str] = None
    html_body: Optional[str] = None
    text_body: Optional[str] = None


class AuthenticateAppRequest(BaseModel):
    client_id: str
    client_secret: str
