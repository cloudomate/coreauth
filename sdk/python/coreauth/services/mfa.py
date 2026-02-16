from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class MfaService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    def enroll_totp(self) -> dict:
        return self._http.post("/api/mfa/enroll/totp")

    def verify_totp(self, method_id: str, code: str) -> dict:
        return self._http.post(f"/api/mfa/totp/{method_id}/verify", json={"code": code})

    def enroll_sms(self, phone_number: str) -> dict:
        return self._http.post("/api/mfa/enroll/sms", json={"phone_number": phone_number})

    def verify_sms(self, method_id: str, code: str) -> dict:
        return self._http.post(f"/api/mfa/sms/{method_id}/verify", json={"code": code})

    def resend_sms(self, method_id: str) -> dict:
        return self._http.post(f"/api/mfa/sms/{method_id}/resend")

    def list_methods(self) -> dict:
        return self._http.get("/api/mfa/methods")

    def delete_method(self, method_id: str) -> dict:
        return self._http.delete(f"/api/mfa/methods/{method_id}")

    def regenerate_backup_codes(self) -> dict:
        return self._http.post("/api/mfa/backup-codes/regenerate")

    def enroll_totp_with_token(self, enrollment_token: str) -> dict:
        return self._http.post("/api/mfa/enroll-with-token/totp", json={
            "enrollment_token": enrollment_token,
        })

    def verify_totp_with_token(self, method_id: str, enrollment_token: str, code: str) -> dict:
        return self._http.post(f"/api/mfa/verify-with-token/totp/{method_id}", json={
            "enrollment_token": enrollment_token,
            "code": code,
        })
