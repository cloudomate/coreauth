from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class TokenRequest(BaseModel):
    grant_type: str
    code: Optional[str] = None
    redirect_uri: Optional[str] = None
    client_id: Optional[str] = None
    client_secret: Optional[str] = None
    code_verifier: Optional[str] = None
    refresh_token: Optional[str] = None
    scope: Optional[str] = None
    audience: Optional[str] = None


class TokenResponse(BaseModel):
    access_token: Optional[str] = None
    token_type: Optional[str] = None
    expires_in: Optional[int] = None
    refresh_token: Optional[str] = None
    id_token: Optional[str] = None
    scope: Optional[str] = None


class UserInfoResponse(BaseModel):
    sub: Optional[str] = None
    name: Optional[str] = None
    given_name: Optional[str] = None
    family_name: Optional[str] = None
    email: Optional[str] = None
    email_verified: Optional[bool] = None
    picture: Optional[str] = None
    locale: Optional[str] = None
    updated_at: Optional[str] = None
    org_id: Optional[str] = None
    org_name: Optional[str] = None


class IntrospectionRequest(BaseModel):
    token: str
    token_type_hint: Optional[str] = None


class IntrospectionResponse(BaseModel):
    active: Optional[bool] = None
    scope: Optional[str] = None
    client_id: Optional[str] = None
    username: Optional[str] = None
    token_type: Optional[str] = None
    exp: Optional[int] = None
    iat: Optional[int] = None
    sub: Optional[str] = None
    aud: Optional[str] = None
    iss: Optional[str] = None
    jti: Optional[str] = None


class RevocationRequest(BaseModel):
    token: str
    token_type_hint: Optional[str] = None


class OidcDiscovery(BaseModel):
    issuer: str
    authorization_endpoint: str
    token_endpoint: str
    userinfo_endpoint: Optional[str] = None
    jwks_uri: str
    registration_endpoint: Optional[str] = None
    scopes_supported: Optional[list[str]] = None
    response_types_supported: Optional[list[str]] = None
    response_modes_supported: Optional[list[str]] = None
    grant_types_supported: Optional[list[str]] = None
    subject_types_supported: Optional[list[str]] = None
    id_token_signing_alg_values_supported: Optional[list[str]] = None
    token_endpoint_auth_methods_supported: Optional[list[str]] = None
    claims_supported: Optional[list[str]] = None
    code_challenge_methods_supported: Optional[list[str]] = None
    introspection_endpoint: Optional[str] = None
    revocation_endpoint: Optional[str] = None
    end_session_endpoint: Optional[str] = None


class Jwks(BaseModel):
    keys: list[dict[str, Any]]
