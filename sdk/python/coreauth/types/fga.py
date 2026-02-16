from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class CreateTupleRequest(BaseModel):
    tenant_id: str
    namespace: str
    object_id: str
    relation: str
    subject_type: str
    subject_id: str
    subject_relation: Optional[str] = None


class QueryTuplesRequest(BaseModel):
    tenant_id: str
    namespace: Optional[str] = None
    object_id: Optional[str] = None
    relation: Optional[str] = None
    subject_type: Optional[str] = None
    subject_id: Optional[str] = None


class RelationTuple(BaseModel):
    id: Optional[str] = None
    tenant_id: Optional[str] = None
    namespace: Optional[str] = None
    object_id: Optional[str] = None
    relation: Optional[str] = None
    subject_type: Optional[str] = None
    subject_id: Optional[str] = None
    subject_relation: Optional[str] = None
    created_at: Optional[str] = None


class CheckRequest(BaseModel):
    tenant_id: str
    subject_type: str
    subject_id: str
    relation: str
    namespace: str
    object_id: str
    context: Optional[dict[str, Any]] = None


class CheckResponse(BaseModel):
    allowed: bool
    reason: Optional[str] = None


class ExpandResponse(BaseModel):
    tree: dict[str, Any]


class ForwardAuthRequest(BaseModel):
    tenant_id: str
    subject_type: str
    subject_id: str
    relation: str
    namespace: str
    object_id: str


class CreateStoreRequest(BaseModel):
    name: str
    description: Optional[str] = None


class FgaStore(BaseModel):
    id: Optional[str] = None
    name: Optional[str] = None
    description: Optional[str] = None
    current_model_version: Optional[int] = None
    is_active: Optional[bool] = None
    tuple_count: Optional[int] = None
    settings: Optional[dict[str, Any]] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class UpdateStoreRequest(BaseModel):
    name: Optional[str] = None
    description: Optional[str] = None
    is_active: Optional[bool] = None


class WriteModelRequest(BaseModel):
    schema: dict[str, Any]


class AuthorizationModel(BaseModel):
    id: Optional[str] = None
    store_id: Optional[str] = None
    version: Optional[int] = None
    schema_json: Optional[dict[str, Any]] = None
    schema_dsl: Optional[str] = None
    is_valid: Optional[bool] = None
    validation_errors: Optional[list[str]] = None
    created_by: Optional[str] = None
    created_at: Optional[str] = None


class CreateApiKeyRequest(BaseModel):
    name: str
    permissions: list[str] = []
    rate_limit_per_minute: Optional[int] = None
    expires_at: Optional[str] = None


class FgaStoreApiKey(BaseModel):
    id: Optional[str] = None
    store_id: Optional[str] = None
    name: Optional[str] = None
    key_prefix: Optional[str] = None
    permissions: Optional[list[str]] = None
    rate_limit_per_minute: Optional[int] = None
    is_active: Optional[bool] = None
    last_used_at: Optional[str] = None
    expires_at: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class FgaStoreApiKeyWithSecret(BaseModel):
    id: Optional[str] = None
    store_id: Optional[str] = None
    name: Optional[str] = None
    key_prefix: Optional[str] = None
    permissions: Optional[list[str]] = None
    rate_limit_per_minute: Optional[int] = None
    is_active: Optional[bool] = None
    last_used_at: Optional[str] = None
    expires_at: Optional[str] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None
    key: str


class WriteTuplesRequest(BaseModel):
    writes: list[dict[str, Any]] = []
    deletes: Optional[list[dict[str, Any]]] = None
