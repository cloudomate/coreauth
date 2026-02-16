from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class TenantRegistryResponse(BaseModel):
    id: Optional[str] = None
    slug: Optional[str] = None
    name: Optional[str] = None
    status: Optional[str] = None
    isolation_mode: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class CreateRegistryTenantRequest(BaseModel):
    slug: str
    name: str
    isolation_mode: Optional[str] = None


class ConfigureDedicatedDbRequest(BaseModel):
    connection_string: str


class TenantRouterStats(BaseModel):
    total_tenants: int
    active_tenants: int
    shared_tenants: int
    dedicated_tenants: int


class Action(BaseModel):
    id: Optional[str] = None
    organization_id: Optional[str] = None
    name: Optional[str] = None
    description: Optional[str] = None
    trigger_type: Optional[str] = None
    code: Optional[str] = None
    runtime: Optional[str] = None
    timeout_seconds: Optional[int] = None
    is_enabled: Optional[bool] = None
    total_executions: Optional[int] = None
    total_failures: Optional[int] = None
    last_executed_at: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class CreateActionRequest(BaseModel):
    name: str
    description: Optional[str] = None
    trigger_type: str
    code: str
    runtime: Optional[str] = None
    timeout_seconds: Optional[int] = None
    secrets: Optional[dict[str, str]] = None
    execution_order: Optional[int] = None


class UpdateActionRequest(BaseModel):
    name: Optional[str] = None
    description: Optional[str] = None
    code: Optional[str] = None
    runtime: Optional[str] = None
    timeout_seconds: Optional[int] = None
    secrets: Optional[dict[str, str]] = None
    execution_order: Optional[int] = None
    is_enabled: Optional[bool] = None


class ActionExecution(BaseModel):
    id: Optional[str] = None
    action_id: Optional[str] = None
    organization_id: Optional[str] = None
    trigger_type: Optional[str] = None
    user_id: Optional[str] = None
    status: Optional[str] = None
    execution_time_ms: Optional[int] = None
    input_data: Optional[dict[str, Any]] = None
    output_data: Optional[dict[str, Any]] = None
    error_message: Optional[str] = None
    executed_at: Optional[str] = None


class ActionTestResponse(BaseModel):
    success: bool
    data: Optional[dict[str, Any]] = None
    error: Optional[str] = None


class RateLimitConfig(BaseModel):
    data: Optional[dict[str, Any]] = None


class UpdateRateLimitRequest(BaseModel):
    data: Optional[dict[str, Any]] = None


class TokenClaimsConfig(BaseModel):
    data: Optional[dict[str, Any]] = None


class UpdateTokenClaimsRequest(BaseModel):
    data: Optional[dict[str, Any]] = None


class ConnectionTestResult(BaseModel):
    success: bool
    message: str
    latency_ms: Optional[int] = None


class HealthResponse(BaseModel):
    status: str
    version: Optional[str] = None
