from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class CreateWebhookRequest(BaseModel):
    name: str
    url: str
    events: list[str] = []
    is_enabled: bool = False
    retry_policy: Optional[dict[str, Any]] = None
    custom_headers: Optional[dict[str, str]] = None


class UpdateWebhookRequest(BaseModel):
    name: Optional[str] = None
    url: Optional[str] = None
    events: Optional[list[str]] = None
    is_enabled: Optional[bool] = None
    retry_policy: Optional[dict[str, Any]] = None
    custom_headers: Optional[dict[str, str]] = None


class WebhookResponse(BaseModel):
    id: Optional[str] = None
    organization_id: Optional[str] = None
    name: Optional[str] = None
    url: Optional[str] = None
    events: Optional[list[str]] = None
    is_enabled: Optional[bool] = None
    retry_policy: Optional[dict[str, Any]] = None
    custom_headers: Optional[dict[str, str]] = None
    total_deliveries: Optional[int] = None
    successful_deliveries: Optional[int] = None
    failed_deliveries: Optional[int] = None
    last_triggered_at: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class WebhookWithSecretResponse(BaseModel):
    id: Optional[str] = None
    organization_id: Optional[str] = None
    name: Optional[str] = None
    url: Optional[str] = None
    events: Optional[list[str]] = None
    is_enabled: Optional[bool] = None
    retry_policy: Optional[dict[str, Any]] = None
    custom_headers: Optional[dict[str, str]] = None
    total_deliveries: Optional[int] = None
    successful_deliveries: Optional[int] = None
    failed_deliveries: Optional[int] = None
    last_triggered_at: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None
    secret: Optional[str] = None


class TestWebhookRequest(BaseModel):
    event_type: Optional[str] = None


class TestWebhookResponse(BaseModel):
    success: bool
    status_code: Optional[int] = None
    response_time_ms: Optional[int] = None
    response_body: Optional[str] = None
    error: Optional[str] = None


class WebhookDelivery(BaseModel):
    id: Optional[str] = None
    webhook_id: Optional[str] = None
    event_id: Optional[str] = None
    event_type: Optional[str] = None
    payload: Optional[dict[str, Any]] = None
    status: Optional[str] = None
    response_status: Optional[int] = None
    response_time_ms: Optional[int] = None
    attempt_count: Optional[int] = None
    max_attempts: Optional[int] = None
    next_retry_at: Optional[str] = None
    last_error: Optional[str] = None
    delivered_at: Optional[str] = None
    failed_at: Optional[str] = None
    created_at: Optional[str] = None


class DeliveryQuery(BaseModel):
    event_type: Optional[str] = None
    status: Optional[str] = None
    limit: Optional[int] = None
    offset: Optional[int] = None


class WebhookEventType(BaseModel):
    id: Optional[str] = None
    category: Optional[str] = None
    description: Optional[str] = None
    payload_schema: Optional[dict[str, Any]] = None
    created_at: Optional[str] = None
