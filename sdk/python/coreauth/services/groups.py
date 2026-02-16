from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class GroupsService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    # --- Groups ---

    def create(self, tenant_id: str, name: str, **kwargs: Any) -> dict:
        body: dict[str, Any] = {"name": name, **kwargs}
        return self._http.post(f"/api/tenants/{tenant_id}/groups", json=body)

    def list(self, tenant_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/groups")

    def get(self, tenant_id: str, group_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/groups/{group_id}")

    def update(self, tenant_id: str, group_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/tenants/{tenant_id}/groups/{group_id}", json=kwargs)

    def delete(self, tenant_id: str, group_id: str) -> dict:
        return self._http.delete(f"/api/tenants/{tenant_id}/groups/{group_id}")

    # --- Members ---

    def add_member(self, tenant_id: str, group_id: str, user_id: str, role: Optional[str] = None) -> dict:
        body: dict[str, Any] = {"user_id": user_id}
        if role:
            body["role"] = role
        return self._http.post(f"/api/tenants/{tenant_id}/groups/{group_id}/members", json=body)

    def list_members(self, tenant_id: str, group_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/groups/{group_id}/members")

    def update_member(self, tenant_id: str, group_id: str, user_id: str, **kwargs: Any) -> dict:
        return self._http.patch(f"/api/tenants/{tenant_id}/groups/{group_id}/members/{user_id}", json=kwargs)

    def remove_member(self, tenant_id: str, group_id: str, user_id: str) -> dict:
        return self._http.delete(f"/api/tenants/{tenant_id}/groups/{group_id}/members/{user_id}")

    # --- Roles ---

    def assign_role(self, tenant_id: str, group_id: str, role_id: str) -> dict:
        return self._http.post(f"/api/tenants/{tenant_id}/groups/{group_id}/roles", json={"role_id": role_id})

    def list_roles(self, tenant_id: str, group_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/groups/{group_id}/roles")

    def remove_role(self, tenant_id: str, group_id: str, role_id: str) -> dict:
        return self._http.delete(f"/api/tenants/{tenant_id}/groups/{group_id}/roles/{role_id}")

    # --- User groups ---

    def get_user_groups(self, tenant_id: str, user_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/users/{user_id}/groups")

    # --- Invitations ---

    def create_invitation(self, tenant_id: str, email: str, **kwargs: Any) -> dict:
        body: dict[str, Any] = {"email": email, **kwargs}
        return self._http.post(f"/api/tenants/{tenant_id}/invitations", json=body)

    def list_invitations(self, tenant_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/invitations")

    def revoke_invitation(self, tenant_id: str, invitation_id: str) -> dict:
        return self._http.delete(f"/api/tenants/{tenant_id}/invitations/{invitation_id}")

    def resend_invitation(self, tenant_id: str, invitation_id: str) -> dict:
        return self._http.post(f"/api/tenants/{tenant_id}/invitations/{invitation_id}/resend")

    def verify_invitation(self, token: str) -> dict:
        return self._http.get("/api/invitations/verify", params={"token": token})

    def accept_invitation(self, token: str, password: str, full_name: str, metadata: Optional[dict] = None) -> dict:
        body: dict[str, Any] = {
            "token": token,
            "password": password,
            "full_name": full_name,
        }
        if metadata:
            body["metadata"] = metadata
        return self._http.post("/api/invitations/accept", json=body)
