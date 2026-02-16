from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class ApplicationsService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    # --- Authz applications ---

    def create(
        self,
        tenant_id: str,
        name: str,
        application_type: str,
        redirect_uris: list[str],
        allowed_scopes: list[str],
        **kwargs: Any,
    ) -> dict:
        body: dict[str, Any] = {
            "tenant_id": tenant_id,
            "name": name,
            "application_type": application_type,
            "redirect_uris": redirect_uris,
            "allowed_scopes": allowed_scopes,
            **kwargs,
        }
        return self._http.post("/api/applications", json=body)

    def list(self, tenant_id: str) -> dict:
        return self._http.get(f"/api/tenants/{tenant_id}/applications")

    def get(self, app_id: str, tenant_id: str) -> dict:
        return self._http.get(f"/api/applications/{app_id}/tenants/{tenant_id}")

    def update(self, app_id: str, tenant_id: str, **kwargs: Any) -> dict:
        return self._http.post(f"/api/applications/{app_id}/tenants/{tenant_id}", json=kwargs)

    def rotate_secret(self, app_id: str, tenant_id: str) -> dict:
        return self._http.post(f"/api/applications/{app_id}/tenants/{tenant_id}/rotate-secret")

    def delete(self, app_id: str, tenant_id: str) -> dict:
        return self._http.delete(f"/api/applications/{app_id}/tenants/{tenant_id}")

    def authenticate(self, client_id: str, client_secret: str) -> dict:
        return self._http.post("/api/applications/authenticate", json={
            "client_id": client_id,
            "client_secret": client_secret,
        })

    # --- OAuth apps (org-scoped) ---

    def create_oauth_app(self, org_id: str, name: str, slug: str, app_type: str, callback_urls: list[str], **kwargs: Any) -> dict:
        body: dict[str, Any] = {
            "name": name,
            "slug": slug,
            "app_type": app_type,
            "callback_urls": callback_urls,
            **kwargs,
        }
        return self._http.post(f"/api/organizations/{org_id}/applications", json=body)

    def list_oauth_apps(self, org_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/applications")

    def get_oauth_app(self, org_id: str, app_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/applications/{app_id}")

    def update_oauth_app(self, org_id: str, app_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/organizations/{org_id}/applications/{app_id}", json=kwargs)

    def rotate_oauth_secret(self, org_id: str, app_id: str) -> dict:
        return self._http.post(f"/api/organizations/{org_id}/applications/{app_id}/rotate-secret")

    def delete_oauth_app(self, org_id: str, app_id: str) -> dict:
        return self._http.delete(f"/api/organizations/{org_id}/applications/{app_id}")

    # --- Email templates ---

    def list_email_templates(self, org_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/email-templates")

    def get_email_template(self, org_id: str, template_type: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/email-templates/{template_type}")

    def update_email_template(self, org_id: str, template_type: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/organizations/{org_id}/email-templates/{template_type}", json=kwargs)

    def delete_email_template(self, org_id: str, template_type: str) -> dict:
        return self._http.delete(f"/api/organizations/{org_id}/email-templates/{template_type}")

    def preview_email_template(self, org_id: str, template_type: str, **kwargs: Any) -> dict:
        return self._http.post(f"/api/organizations/{org_id}/email-templates/{template_type}/preview", json=kwargs)
