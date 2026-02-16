"""CoreAuth SDK client."""

from __future__ import annotations

from typing import Optional

from ._http import HttpClient
from .services import (
    AdminService,
    ApplicationsService,
    AuditService,
    AuthService,
    ConnectionsService,
    FgaService,
    GroupsService,
    MfaService,
    OAuth2Service,
    ScimService,
    TenantsService,
    WebhooksService,
)


class CoreAuthClient:
    """Main client for the CoreAuth API.

    Usage:
        client = CoreAuthClient("http://localhost:3000")
        resp = client.auth.login("my-tenant", "user@example.com", "password")
        client.set_token(resp["access_token"])
        profile = client.auth.get_profile()
    """

    def __init__(self, base_url: str, token: Optional[str] = None) -> None:
        self._http = HttpClient(base_url, token)
        self.auth = AuthService(self._http)
        self.oauth2 = OAuth2Service(self._http)
        self.mfa = MfaService(self._http)
        self.tenants = TenantsService(self._http)
        self.applications = ApplicationsService(self._http)
        self.fga = FgaService(self._http)
        self.audit = AuditService(self._http)
        self.webhooks = WebhooksService(self._http)
        self.groups = GroupsService(self._http)
        self.scim = ScimService(self._http)
        self.admin = AdminService(self._http)
        self.connections = ConnectionsService(self._http)

    def set_token(self, token: str) -> None:
        """Set the bearer token for all subsequent requests."""
        self._http.set_token(token)

    def clear_token(self) -> None:
        """Remove the bearer token."""
        self._http.clear_token()

    def close(self) -> None:
        """Close the underlying HTTP client."""
        self._http.close()

    def __enter__(self) -> CoreAuthClient:
        return self

    def __exit__(self, *args: object) -> None:
        self.close()

    def __repr__(self) -> str:
        return f"CoreAuthClient(base_url={self._http._base_url!r})"
