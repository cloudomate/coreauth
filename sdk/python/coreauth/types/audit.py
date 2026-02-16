from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class AuditLogQuery(BaseModel):
    tenant_id: Optional[str] = None
    event_types: Optional[list[str]] = None
    event_categories: Optional[list[str]] = None
    actor_id: Optional[str] = None
    target_id: Optional[str] = None
    status: Optional[str] = None
    from_date: Optional[str] = None
    to_date: Optional[str] = None
    limit: Optional[int] = None
    offset: Optional[int] = None


class AuditLog(BaseModel):
    id: Optional[str] = None
    tenant_id: Optional[str] = None
    event_type: Optional[str] = None
    event_category: Optional[str] = None
    event_action: Optional[str] = None
    actor_type: Optional[str] = None
    actor_id: Optional[str] = None
    actor_name: Optional[str] = None
    actor_ip_address: Optional[str] = None
    actor_user_agent: Optional[str] = None
    target_type: Optional[str] = None
    target_id: Optional[str] = None
    target_name: Optional[str] = None
    description: Optional[str] = None
    metadata: Optional[dict[str, Any]] = None
    status: Optional[str] = None
    error_message: Optional[str] = None
    request_id: Optional[str] = None
    session_id: Optional[str] = None
    created_at: Optional[str] = None


class AuditLogsResponse(BaseModel):
    logs: list[AuditLog] = []
    total: int
    limit: int
    offset: int


class AuditStats(BaseModel):
    data: Optional[dict[str, Any]] = None
