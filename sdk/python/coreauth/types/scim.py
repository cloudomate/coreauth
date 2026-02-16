from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class ScimUser(BaseModel):
    schemas: Optional[list[str]] = None
    id: Optional[str] = None
    external_id: Optional[str] = None
    user_name: Optional[str] = None
    name: Optional[dict[str, str]] = None
    display_name: Optional[str] = None
    emails: Optional[list[dict[str, Any]]] = None
    phone_numbers: Optional[list[dict[str, Any]]] = None
    active: Optional[bool] = None
    groups: Optional[list[dict[str, Any]]] = None
    meta: Optional[dict[str, Any]] = None


class CreateScimUserRequest(BaseModel):
    schemas: Optional[list[str]] = None
    external_id: Optional[str] = None
    user_name: str
    name: dict[str, str]
    display_name: Optional[str] = None
    emails: list[dict[str, Any]] = []
    phone_numbers: Optional[list[dict[str, Any]]] = None
    active: bool = True
    password: Optional[str] = None


class ScimGroup(BaseModel):
    schemas: Optional[list[str]] = None
    id: Optional[str] = None
    external_id: Optional[str] = None
    display_name: Optional[str] = None
    members: Optional[list[dict[str, Any]]] = None
    meta: Optional[dict[str, Any]] = None


class CreateScimGroupRequest(BaseModel):
    schemas: Optional[list[str]] = None
    external_id: Optional[str] = None
    display_name: str
    members: Optional[list[dict[str, Any]]] = None


class ScimPatchRequest(BaseModel):
    schemas: Optional[list[str]] = None
    operations: list[dict[str, Any]] = []


class ScimListResponse(BaseModel):
    schemas: Optional[list[str]] = None
    total_results: int
    items_per_page: int
    start_index: int
    resources: list[dict[str, Any]] = []


class ScimTokenResponse(BaseModel):
    id: Optional[str] = None
    name: Optional[str] = None
    token_prefix: Optional[str] = None
    expires_at: Optional[str] = None
    created_at: Optional[str] = None


class ScimTokenWithSecret(BaseModel):
    id: Optional[str] = None
    name: Optional[str] = None
    token_prefix: Optional[str] = None
    expires_at: Optional[str] = None
    created_at: Optional[str] = None
    secret: Optional[str] = None


class CreateScimTokenRequest(BaseModel):
    name: str
    expires_at: Optional[str] = None


class SessionInfo(BaseModel):
    id: Optional[str] = None
    user_id: Optional[str] = None
    ip_address: Optional[str] = None
    user_agent: Optional[str] = None
    authenticated_at: Optional[str] = None
    last_active_at: Optional[str] = None
    expires_at: Optional[str] = None
    created_at: Optional[str] = None


class OidcProvider(BaseModel):
    id: Optional[str] = None
    tenant_id: Optional[str] = None
    name: Optional[str] = None
    provider_type: Optional[str] = None
    issuer: Optional[str] = None
    client_id: Optional[str] = None
    authorization_endpoint: Optional[str] = None
    token_endpoint: Optional[str] = None
    userinfo_endpoint: Optional[str] = None
    jwks_uri: Optional[str] = None
    scopes: Optional[list[str]] = None
    groups_claim: Optional[str] = None
    group_role_mappings: Optional[dict[str, str]] = None
    allowed_group_id: Optional[str] = None
    is_enabled: Optional[bool] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class CreateOidcProviderRequest(BaseModel):
    tenant_id: str
    name: str
    provider_type: str
    issuer: str
    client_id: str
    client_secret: str
    authorization_endpoint: str
    token_endpoint: str
    userinfo_endpoint: Optional[str] = None
    jwks_uri: str
    scopes: Optional[list[str]] = None
    groups_claim: Optional[str] = None
    group_role_mappings: Optional[dict[str, str]] = None
    allowed_group_id: Optional[str] = None


class UpdateOidcProviderRequest(BaseModel):
    is_enabled: Optional[bool] = None
    name: Optional[str] = None
    client_id: Optional[str] = None
    client_secret: Optional[str] = None


class OidcLoginResponse(BaseModel):
    authorization_url: str
    state: str


class SsoCheckResponse(BaseModel):
    has_sso: bool
    providers: list[dict[str, Any]] = []
