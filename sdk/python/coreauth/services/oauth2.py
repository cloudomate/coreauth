from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional
from urllib.parse import urlencode

if TYPE_CHECKING:
    from .._http import HttpClient


class OAuth2Service:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    def discovery(self) -> dict:
        return self._http.get("/.well-known/openid-configuration")

    def jwks(self) -> dict:
        return self._http.get("/.well-known/jwks.json")

    def authorize(
        self,
        client_id: str,
        redirect_uri: str,
        response_type: str = "code",
        scope: str = "openid",
        **params: Any,
    ) -> str:
        query: dict[str, Any] = {
            "client_id": client_id,
            "redirect_uri": redirect_uri,
            "response_type": response_type,
            "scope": scope,
            **params,
        }
        base_url = self._http._base_url.rstrip("/")
        return f"{base_url}/authorize?{urlencode(query)}"

    def token(self, grant_type: str, **kwargs: Any) -> dict:
        data = {"grant_type": grant_type, **kwargs}
        return self._http.post_form("/oauth/token", data=data)

    def userinfo(self) -> dict:
        return self._http.get("/userinfo")

    def revoke(self, token: str, token_type_hint: Optional[str] = None) -> dict:
        data: dict[str, str] = {"token": token}
        if token_type_hint:
            data["token_type_hint"] = token_type_hint
        return self._http.post_form("/oauth/revoke", data=data)

    def introspect(self, token: str, token_type_hint: Optional[str] = None) -> dict:
        data: dict[str, str] = {"token": token}
        if token_type_hint:
            data["token_type_hint"] = token_type_hint
        return self._http.post_form("/oauth/introspect", data=data)

    def oidc_logout(
        self,
        id_token_hint: Optional[str] = None,
        post_logout_redirect_uri: Optional[str] = None,
        state: Optional[str] = None,
    ) -> dict:
        params: dict[str, str] = {}
        if id_token_hint:
            params["id_token_hint"] = id_token_hint
        if post_logout_redirect_uri:
            params["post_logout_redirect_uri"] = post_logout_redirect_uri
        if state:
            params["state"] = state
        return self._http.get("/logout", params=params or None)
