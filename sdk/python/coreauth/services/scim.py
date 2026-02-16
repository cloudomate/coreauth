from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class ScimService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    # --- SCIM Config (public) ---

    def get_config(self) -> dict:
        return self._http.get("/scim/v2/ServiceProviderConfig")

    def get_resource_types(self) -> dict:
        return self._http.get("/scim/v2/ResourceTypes")

    def get_schemas(self) -> dict:
        return self._http.get("/scim/v2/Schemas")

    # --- SCIM Users ---

    def list_users(
        self,
        filter: Optional[str] = None,
        count: Optional[int] = None,
        start_index: Optional[int] = None,
        **params: Any,
    ) -> dict:
        p: dict[str, Any] = {**params}
        if filter:
            p["filter"] = filter
        if count is not None:
            p["count"] = count
        if start_index is not None:
            p["startIndex"] = start_index
        return self._http.get("/scim/v2/Users", params=p or None)

    def create_user(self, **kwargs: Any) -> dict:
        return self._http.post("/scim/v2/Users", json=kwargs)

    def get_user(self, user_id: str) -> dict:
        return self._http.get(f"/scim/v2/Users/{user_id}")

    def replace_user(self, user_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/scim/v2/Users/{user_id}", json=kwargs)

    def patch_user(self, user_id: str, operations: list[dict]) -> dict:
        return self._http.patch(f"/scim/v2/Users/{user_id}", json={"Operations": operations})

    def delete_user(self, user_id: str) -> dict:
        return self._http.delete(f"/scim/v2/Users/{user_id}")

    # --- SCIM Groups ---

    def list_scim_groups(
        self,
        filter: Optional[str] = None,
        count: Optional[int] = None,
        start_index: Optional[int] = None,
        **params: Any,
    ) -> dict:
        p: dict[str, Any] = {**params}
        if filter:
            p["filter"] = filter
        if count is not None:
            p["count"] = count
        if start_index is not None:
            p["startIndex"] = start_index
        return self._http.get("/scim/v2/Groups", params=p or None)

    def create_scim_group(self, **kwargs: Any) -> dict:
        return self._http.post("/scim/v2/Groups", json=kwargs)

    def get_scim_group(self, group_id: str) -> dict:
        return self._http.get(f"/scim/v2/Groups/{group_id}")

    def patch_scim_group(self, group_id: str, operations: list[dict]) -> dict:
        return self._http.patch(f"/scim/v2/Groups/{group_id}", json={"Operations": operations})

    def delete_scim_group(self, group_id: str) -> dict:
        return self._http.delete(f"/scim/v2/Groups/{group_id}")

    # --- SCIM Tokens ---

    def list_scim_tokens(self, org_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/scim/tokens")

    def create_scim_token(self, org_id: str, name: str, expires_at: Optional[str] = None) -> dict:
        body: dict[str, Any] = {"name": name}
        if expires_at:
            body["expires_at"] = expires_at
        return self._http.post(f"/api/organizations/{org_id}/scim/tokens", json=body)

    def revoke_scim_token(self, org_id: str, token_id: str) -> dict:
        return self._http.delete(f"/api/organizations/{org_id}/scim/tokens/{token_id}")

    # --- Sessions ---

    def list_sessions(self, user_id: Optional[str] = None) -> dict:
        params: dict[str, Any] = {}
        if user_id:
            params["user_id"] = user_id
        return self._http.get("/api/sessions", params=params or None)

    def revoke_session(self, session_id: str) -> dict:
        return self._http.delete(f"/api/sessions/{session_id}")

    def revoke_all_sessions(self) -> dict:
        return self._http.post("/api/sessions/revoke-all")

    # --- OIDC Providers ---

    def list_oidc_providers(self, tenant_id: str) -> dict:
        return self._http.get("/api/oidc/providers", params={"tenant_id": tenant_id})

    def create_oidc_provider(self, **kwargs: Any) -> dict:
        return self._http.post("/api/oidc/providers", json=kwargs)

    def update_oidc_provider(self, provider_id: str, **kwargs: Any) -> dict:
        return self._http.patch(f"/api/oidc/providers/{provider_id}", json=kwargs)

    def delete_oidc_provider(self, provider_id: str) -> dict:
        return self._http.delete(f"/api/oidc/providers/{provider_id}")

    def list_public_providers(self, tenant_id: str) -> dict:
        return self._http.get("/api/oidc/providers/public", params={"tenant_id": tenant_id})

    def list_provider_templates(self) -> dict:
        return self._http.get("/api/oidc/templates")

    def get_provider_template(self, provider_type: str) -> dict:
        return self._http.get(f"/api/oidc/templates/{provider_type}")

    def sso_check(self, email: str) -> dict:
        return self._http.get("/api/oidc/sso-check", params={"email": email})
