"""Connection types."""

from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class Connection(BaseModel):
    id: str
    name: str
    connection_type: str
    scope: str
    organization_id: Optional[str] = None
    config: Any = None
    is_enabled: bool = True
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class CreateConnectionRequest(BaseModel):
    name: str
    connection_type: str
    config: Any = None


class UpdateConnectionRequest(BaseModel):
    name: Optional[str] = None
    config: Any = None
    is_enabled: Optional[bool] = None


class AuthMethod(BaseModel):
    connection_id: str
    name: str
    method_type: str
    scope: str
