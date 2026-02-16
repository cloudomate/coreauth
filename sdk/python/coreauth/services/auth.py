from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class AuthService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    def register(self, tenant_id: str, email: str, password: str, phone: Optional[str] = None) -> dict:
        body: dict[str, Any] = {"tenant_id": tenant_id, "email": email, "password": password}
        if phone:
            body["phone"] = phone
        return self._http.post("/api/auth/register", json=body)

    def login(self, tenant_id: str, email: str, password: str) -> dict:
        return self._http.post("/api/auth/login", json={
            "tenant_id": tenant_id,
            "email": email,
            "password": password,
        })

    def login_hierarchical(self, email: str, password: str, organization_slug: Optional[str] = None) -> dict:
        body: dict[str, Any] = {"email": email, "password": password}
        if organization_slug:
            body["organization_slug"] = organization_slug
        return self._http.post("/api/auth/login-hierarchical", json=body)

    def refresh_token(self, refresh_token: str) -> dict:
        return self._http.post("/api/auth/refresh", json={"refresh_token": refresh_token})

    def logout(self) -> None:
        self._http.post("/api/auth/logout")

    def get_profile(self) -> dict:
        return self._http.get("/api/auth/me")

    def update_profile(self, **kwargs: Any) -> dict:
        return self._http.patch("/api/auth/me", json=kwargs)

    def change_password(self, current_password: str, new_password: str) -> dict:
        return self._http.post("/api/auth/change-password", json={
            "current_password": current_password,
            "new_password": new_password,
        })

    def verify_email(self, token: str) -> dict:
        return self._http.get("/api/verify-email", params={"token": token})

    def resend_verification(self) -> dict:
        return self._http.post("/api/auth/resend-verification")

    def forgot_password(self, tenant_id: str, email: str) -> dict:
        return self._http.post("/api/auth/forgot-password", json={
            "tenant_id": tenant_id,
            "email": email,
        })

    def verify_reset_token(self, token: str) -> dict:
        return self._http.get("/api/auth/verify-reset-token", params={"token": token})

    def reset_password(self, token: str, new_password: str) -> dict:
        return self._http.post("/api/auth/reset-password", json={
            "token": token,
            "new_password": new_password,
        })

    def passwordless_start(self, tenant_id: str, method: str, email: str) -> dict:
        return self._http.post(f"/api/tenants/{tenant_id}/passwordless/start", json={
            "method": method,
            "email": email,
        })

    def passwordless_verify(self, tenant_id: str, token_or_code: str) -> dict:
        return self._http.post(f"/api/tenants/{tenant_id}/passwordless/verify", json={
            "token_or_code": token_or_code,
        })

    def passwordless_resend(self, tenant_id: str, **kwargs: Any) -> dict:
        return self._http.post(f"/api/tenants/{tenant_id}/passwordless/resend", json=kwargs)

    def create_login_flow_browser(self, organization_id: Optional[str] = None, request_id: Optional[str] = None) -> dict:
        params: dict[str, str] = {}
        if organization_id:
            params["organization_id"] = organization_id
        if request_id:
            params["request_id"] = request_id
        return self._http.get("/self-service/login/browser", params=params or None)

    def create_login_flow_api(self, organization_id: Optional[str] = None, request_id: Optional[str] = None) -> dict:
        params: dict[str, str] = {}
        if organization_id:
            params["organization_id"] = organization_id
        if request_id:
            params["request_id"] = request_id
        return self._http.get("/self-service/login/api", params=params or None)

    def get_login_flow(self, flow_id: str) -> dict:
        return self._http.get("/self-service/login", params={"flow": flow_id})

    def submit_login_flow(self, flow_id: str, **kwargs: Any) -> dict:
        return self._http.post("/self-service/login", json={"flow": flow_id, **kwargs})

    def create_registration_flow_browser(self, organization_id: Optional[str] = None) -> dict:
        params: dict[str, str] = {}
        if organization_id:
            params["organization_id"] = organization_id
        return self._http.get("/self-service/registration/browser", params=params or None)

    def create_registration_flow_api(self, organization_id: Optional[str] = None) -> dict:
        params: dict[str, str] = {}
        if organization_id:
            params["organization_id"] = organization_id
        return self._http.get("/self-service/registration/api", params=params or None)

    def get_registration_flow(self, flow_id: str) -> dict:
        return self._http.get("/self-service/registration", params={"flow": flow_id})

    def submit_registration_flow(self, flow_id: str, **kwargs: Any) -> dict:
        return self._http.post("/self-service/registration", json={"flow": flow_id, **kwargs})

    def whoami(self) -> dict:
        return self._http.get("/sessions/whoami")
