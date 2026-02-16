from __future__ import annotations

from typing import Optional

from pydantic import BaseModel


class MfaEnrollResponse(BaseModel):
    method_id: Optional[str] = None
    method_type: Optional[str] = None
    secret: Optional[str] = None
    qr_code_uri: Optional[str] = None
    backup_codes: Optional[list[str]] = None


class SmsMfaEnrollResponse(BaseModel):
    method_id: Optional[str] = None
    method_type: Optional[str] = None
    phone_number: Optional[str] = None
    masked_phone: Optional[str] = None
    expires_at: Optional[str] = None


class VerifyMfaRequest(BaseModel):
    code: str


class EnrollSmsRequest(BaseModel):
    phone_number: str


class EnrollWithTokenRequest(BaseModel):
    enrollment_token: str


class VerifyWithTokenRequest(BaseModel):
    enrollment_token: str
    code: str


class MfaMethod(BaseModel):
    id: Optional[str] = None
    user_id: Optional[str] = None
    method_type: Optional[str] = None
    verified: Optional[bool] = None
    name: Optional[str] = None
    created_at: Optional[str] = None
    last_used_at: Optional[str] = None
