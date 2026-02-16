from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class AuditService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    def query(
        self,
        tenant_id: Optional[str] = None,
        event_types: Optional[str] = None,
        user_id: Optional[str] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
        limit: Optional[int] = None,
        offset: Optional[int] = None,
    ) -> dict:
        params: dict[str, Any] = {}
        if tenant_id:
            params["tenant_id"] = tenant_id
        if event_types:
            params["event_types"] = event_types
        if user_id:
            params["user_id"] = user_id
        if start_date:
            params["start_date"] = start_date
        if end_date:
            params["end_date"] = end_date
        if limit is not None:
            params["limit"] = limit
        if offset is not None:
            params["offset"] = offset
        return self._http.get("/api/audit/logs", params=params or None)

    def get(self, log_id: str) -> dict:
        return self._http.get(f"/api/audit/logs/{log_id}")

    def security_events(self) -> dict:
        return self._http.get("/api/audit/security-events")

    def failed_logins(self, user_id: str) -> dict:
        return self._http.get(f"/api/audit/failed-logins/{user_id}")

    def export(self) -> dict:
        return self._http.get("/api/audit/export")

    def stats(self) -> dict:
        return self._http.get("/api/audit/stats")

    def login_history(self) -> dict:
        return self._http.get("/api/login-history")

    def security_audit_logs(self) -> dict:
        return self._http.get("/api/security/audit-logs")
