from __future__ import annotations

from typing import Any, Optional

from pydantic import BaseModel


class CreateGroupRequest(BaseModel):
    name: str
    description: Optional[str] = None
    external_id: Optional[str] = None


class Group(BaseModel):
    id: Optional[str] = None
    tenant_id: Optional[str] = None
    organization_id: Optional[str] = None
    name: Optional[str] = None
    description: Optional[str] = None
    external_id: Optional[str] = None
    member_count: Optional[int] = None
    created_at: Optional[str] = None
    updated_at: Optional[str] = None


class UpdateGroupRequest(BaseModel):
    name: Optional[str] = None
    description: Optional[str] = None


class AddGroupMemberRequest(BaseModel):
    user_id: str
    role: Optional[str] = None


class GroupMember(BaseModel):
    id: Optional[str] = None
    user_id: Optional[str] = None
    group_id: Optional[str] = None
    role: Optional[str] = None
    email: Optional[str] = None
    full_name: Optional[str] = None
    joined_at: Optional[str] = None


class UpdateGroupMemberRequest(BaseModel):
    role: str


class GroupRole(BaseModel):
    role_id: Optional[str] = None
    group_id: Optional[str] = None
    created_at: Optional[str] = None


class AssignGroupRoleRequest(BaseModel):
    role_id: str


class CreateInvitationRequest(BaseModel):
    email: str
    role_id: Optional[str] = None
    metadata: Optional[dict[str, Any]] = None
    expires_in_days: Optional[int] = None


class InvitationResponse(BaseModel):
    id: Optional[str] = None
    tenant_id: Optional[str] = None
    email: Optional[str] = None
    role_id: Optional[str] = None
    status: Optional[str] = None
    invited_by: Optional[str] = None
    expires_at: Optional[str] = None
    created_at: Optional[str] = None


class AcceptInvitationRequest(BaseModel):
    token: str
    password: str
    full_name: str
    metadata: Optional[dict[str, Any]] = None
