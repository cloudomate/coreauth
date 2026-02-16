from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class WebhooksService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    def create(self, org_id: str, name: str, url: str, events: list[str], is_enabled: bool = False, **kwargs: Any) -> dict:
        body: dict[str, Any] = {
            "name": name,
            "url": url,
            "events": events,
            "is_enabled": is_enabled,
            **kwargs,
        }
        return self._http.post(f"/api/organizations/{org_id}/webhooks", json=body)

    def list(self, org_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/webhooks")

    def get(self, org_id: str, webhook_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/webhooks/{webhook_id}")

    def update(self, org_id: str, webhook_id: str, **kwargs: Any) -> dict:
        return self._http.put(f"/api/organizations/{org_id}/webhooks/{webhook_id}", json=kwargs)

    def delete(self, org_id: str, webhook_id: str) -> dict:
        return self._http.delete(f"/api/organizations/{org_id}/webhooks/{webhook_id}")

    def rotate_secret(self, org_id: str, webhook_id: str) -> dict:
        return self._http.post(f"/api/organizations/{org_id}/webhooks/{webhook_id}/rotate-secret")

    def test(self, org_id: str, webhook_id: str, event_type: Optional[str] = None) -> dict:
        body: dict[str, Any] = {}
        if event_type:
            body["event_type"] = event_type
        return self._http.post(f"/api/organizations/{org_id}/webhooks/{webhook_id}/test", json=body or None)

    def list_deliveries(self, org_id: str, webhook_id: str, **params: Any) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/webhooks/{webhook_id}/deliveries", params=params or None)

    def get_delivery(self, org_id: str, webhook_id: str, delivery_id: str) -> dict:
        return self._http.get(f"/api/organizations/{org_id}/webhooks/{webhook_id}/deliveries/{delivery_id}")

    def retry_delivery(self, org_id: str, webhook_id: str, delivery_id: str) -> dict:
        return self._http.post(f"/api/organizations/{org_id}/webhooks/{webhook_id}/deliveries/{delivery_id}/retry")

    def list_event_types(self) -> dict:
        return self._http.get("/api/webhooks/event-types")
