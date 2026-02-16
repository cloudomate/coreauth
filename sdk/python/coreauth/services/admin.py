from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class AdminService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    # --- Tenant Registry ---

    def list_tenants(self) -> dict:
        return self._http.get("/api/admin/tenants")

    def create_tenant(self, slug: str, name: str, isolation_mode: Optional[str] = None) -> dict:
        body: dict[str, Any] = {"slug": slug, "name": name}
        if isolation_mode:
            body["isolation_mode"] = isolation_mode
        return self._http.post("/api/admin/tenants", json=body)

    def get_stats(self) -> dict:
        return self._http.get("/api/admin/tenants/stats")

    def get_tenant(self, tenant_id: str) -> dict:
        return self._http.get(f"/api/admin/tenants/{tenant_id}")

    def configure_database(self, tenant_id: str, connection_string: str) -> dict:
        return self._http.post(f"/api/admin/tenants/{tenant_id}/database", json={
            "connection_string": connection_string,
        })

    def activate(self, tenant_id: str) -> dict:
        return self._http.post(f"/api/admin/tenants/{tenant_id}/activate")

    def suspend(self, tenant_id: str) -> dict:
        return self._http.post(f"/api/admin/tenants/{tenant_id}/suspend")

    def test_connection(self, tenant_id: str) -> dict:
        return self._http.post(f"/api/admin/tenants/{tenant_id}/test-connection")

    # --- Actions ---

    def create_action(self, org_id: str, name: str, trigger_type: str, code: str, **kwargs: Any) -> dict:
        body: dict[str, Any] = {
            "name": name,
            "trigger_type": trigger_type,
            "code": code,
            **kwargs,
        }
        return self._http.post(f"/api/organizations/{org_id}/actions", json=body)

    def list_actions(self, org_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/actions")

    def get_action(self, org_id: str, action_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/actions/{action_id}")

    def update_action(self, org_id: str, action_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/organizations/{org_id}/actions/{action_id}", json=kwargs)

    def delete_action(self, org_id: str, action_id: str) -> dict:
        return self._http.delete(f"/api/organizations/{org_id}/actions/{action_id}")

    def test_action(self, org_id: str, action_id: str, **kwargs: Any) -> dict:
        return self._http.post(f"/api/organizations/{org_id}/actions/{action_id}/test", json=kwargs)

    def get_action_executions(self, org_id: str, action_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/actions/{action_id}/executions")

    def get_org_executions(self, org_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/actions/executions")

    # --- Rate limits & token claims ---

    def get_rate_limits(self, tenant_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/rate-limits")

    def update_rate_limits(self, tenant_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/tenants/{tenant_id}/rate-limits", json=kwargs)

    def get_token_claims(self, tenant_id: str, app_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/applications/{app_id}/token-claims")

    def update_token_claims(self, tenant_id: str, app_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/tenants/{tenant_id}/applications/{app_id}/token-claims", json=kwargs)

    # --- Health ---

    def health(self) -> dict:
        return self._http.get("/health")
