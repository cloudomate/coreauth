"""Internal HTTP transport for CoreAuth SDK."""

from __future__ import annotations

from typing import Any, Optional

import httpx

from .exceptions import (
    ApiError,
    AuthenticationError,
    ConflictError,
    CoreAuthError,
    ForbiddenError,
    NotFoundError,
    RateLimitError,
    ValidationError,
)


class HttpClient:
    """Low-level HTTP client wrapping httpx."""

    def __init__(self, base_url: str, token: Optional[str] = None) -> None:
        self._base_url = base_url.rstrip("/")
        self._token: Optional[str] = token
        self._client = httpx.Client(timeout=30.0)

    def set_token(self, token: str) -> None:
        self._token = token

    def clear_token(self) -> None:
        self._token = None

    def _headers(self) -> dict[str, str]:
        headers: dict[str, str] = {"Content-Type": "application/json"}
        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"
        return headers

    def _form_headers(self) -> dict[str, str]:
        headers: dict[str, str] = {"Content-Type": "application/x-www-form-urlencoded"}
        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"
        return headers

    def _handle_response(self, resp: httpx.Response) -> Any:
        if resp.status_code == 204:
            return None
        if 200 <= resp.status_code < 300:
            if not resp.content:
                return None
            return resp.json()
        # Error handling
        try:
            body = resp.json()
            error = body.get("error", "")
            message = body.get("message", "")
        except Exception:
            error = ""
            message = resp.text

        error_map = {
            400: ValidationError,
            401: AuthenticationError,
            403: ForbiddenError,
            404: NotFoundError,
            409: ConflictError,
            429: RateLimitError,
        }
        cls = error_map.get(resp.status_code, ApiError)
        if cls is ApiError:
            raise ApiError(resp.status_code, error, message)
        raise cls(error, message)

    def get(self, path: str, params: Optional[dict[str, Any]] = None) -> Any:
        url = f"{self._base_url}{path}"
        try:
            resp = self._client.get(url, headers=self._headers(), params=params)
        except httpx.HTTPError as e:
            raise CoreAuthError(f"Request failed: {e}") from e
        return self._handle_response(resp)

    def post(self, path: str, json: Optional[dict[str, Any]] = None) -> Any:
        url = f"{self._base_url}{path}"
        try:
            resp = self._client.post(url, headers=self._headers(), json=json)
        except httpx.HTTPError as e:
            raise CoreAuthError(f"Request failed: {e}") from e
        return self._handle_response(resp)

    def post_form(self, path: str, data: dict[str, Any]) -> Any:
        url = f"{self._base_url}{path}"
        try:
            resp = self._client.post(url, headers=self._form_headers(), data=data)
        except httpx.HTTPError as e:
            raise CoreAuthError(f"Request failed: {e}") from e
        return self._handle_response(resp)

    def put(self, path: str, json: Optional[dict[str, Any]] = None) -> Any:
        url = f"{self._base_url}{path}"
        try:
            resp = self._client.put(url, headers=self._headers(), json=json)
        except httpx.HTTPError as e:
            raise CoreAuthError(f"Request failed: {e}") from e
        return self._handle_response(resp)

    def patch(self, path: str, json: Optional[dict[str, Any]] = None) -> Any:
        url = f"{self._base_url}{path}"
        try:
            resp = self._client.patch(url, headers=self._headers(), json=json)
        except httpx.HTTPError as e:
            raise CoreAuthError(f"Request failed: {e}") from e
        return self._handle_response(resp)

    def delete(self, path: str, json: Optional[dict[str, Any]] = None) -> Any:
        url = f"{self._base_url}{path}"
        try:
            resp = self._client.delete(url, headers=self._headers(), json=json)
        except httpx.HTTPError as e:
            raise CoreAuthError(f"Request failed: {e}") from e
        return self._handle_response(resp)

    def close(self) -> None:
        self._client.close()
