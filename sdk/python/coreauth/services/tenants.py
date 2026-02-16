from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class TenantsService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    def create(
        self,
        name: str,
        slug: str,
        admin_email: str,
        admin_password: str,
        admin_full_name: Optional[str] = None,
        **kwargs: Any,
    ) -> dict:
        body: dict[str, Any] = {
            "name": name,
            "slug": slug,
            "admin_email": admin_email,
            "admin_password": admin_password,
            **kwargs,
        }
        if admin_full_name:
            body["admin_full_name"] = admin_full_name
        return self._http.post("/api/tenants", json=body)

    def get_by_slug(self, slug: str) -> dict:
        return self._http.get(f"/api/organizations/by-slug/{slug}")

    def list_users(self, tenant_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/users")

    def update_user_role(self, tenant_id: str, user_id: str, role: str) -> dict:
        return self._http.put(f"/api/tenants/{tenant_id}/users/{user_id}/role", json={"role": role})

    def get_security(self, org_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/security")

    def update_security(self, org_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/organizations/{org_id}/security", json=kwargs)

    def get_branding(self, org_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/branding")

    def update_branding(self, org_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/organizations/{org_id}/branding", json=kwargs)
