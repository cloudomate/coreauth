"""Connections service."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from .._http import HttpClient


class ConnectionsService:
    """Manage authentication connections (SSO, OIDC, SAML, social, database)."""

    def __init__(self, http: HttpClient) -> None:
        self._http = http

    def list(self, org_id: str) -> list[dict[str, Any]]:
        """List all connections for an organization (includes platform connections)."""
        return self._http.get(f"/api/organizations/{org_id}/connections")

    def create(self, org_id: str, data: dict[str, Any]) -> dict[str, Any]:
        """Create an organization-scoped connection."""
        return self._http.post(f"/api/organizations/{org_id}/connections", data)

    def get(self, org_id: str, connection_id: str) -> dict[str, Any]:
        """Get a specific connection."""
        return self._http.get(f"/api/organizations/{org_id}/connections/{connection_id}")

    def update(self, org_id: str, connection_id: str, data: dict[str, Any]) -> dict[str, Any]:
        """Update a connection."""
        return self._http.put(f"/api/organizations/{org_id}/connections/{connection_id}", data)

    def delete(self, org_id: str, connection_id: str) -> None:
        """Delete a connection."""
        self._http.delete(f"/api/organizations/{org_id}/connections/{connection_id}")

    def get_auth_methods(self, org_id: str) -> list[dict[str, Any]]:
        """Get available authentication methods for an organization."""
        return self._http.get(f"/api/organizations/{org_id}/connections/auth-methods")

    def list_all(self) -> list[dict[str, Any]]:
        """Admin: list all connections across all organizations."""
        return self._http.get("/api/admin/connections")

    def create_platform(self, data: dict[str, Any]) -> dict[str, Any]:
        """Admin: create a platform-scoped connection."""
        return self._http.post("/api/admin/connections", data)
