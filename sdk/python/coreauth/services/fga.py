from __future__ import annotations

from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .._http import HttpClient


class FgaService:
    def __init__(self, http: HttpClient) -> None:
        self._http = http

    # --- Tuples (tenant-scoped) ---

    def create_tuple(
        self,
        tenant_id: str,
        namespace: str,
        object_id: str,
        relation: str,
        subject_type: str,
        subject_id: str,
        subject_relation: Optional[str] = None,
    ) -> dict:
        body: dict[str, Any] = {
            "tenant_id": tenant_id,
            "namespace": namespace,
            "object_id": object_id,
            "relation": relation,
            "subject_type": subject_type,
            "subject_id": subject_id,
        }
        if subject_relation:
            body["subject_relation"] = subject_relation
        return self._http.post("/api/authz/tuples", json=body)

    def delete_tuple(self, **kwargs: Any) -> dict:
        return self._http.delete("/api/authz/tuples", json=kwargs)

    def query_tuples(self, tenant_id: str, **kwargs: Any) -> dict:
        return self._http.post("/api/authz/tuples/query", json={"tenant_id": tenant_id, **kwargs})

    def get_object_tuples(self, tenant_id: str, namespace: str, object_id: str) -> dict:
        return self._http.get(f"/api/authz/tuples/by-object/{tenant_id}/{namespace}/{object_id}")

    def get_subject_tuples(self, tenant_id: str, subject_type: str, subject_id: str) -> dict:
        return self._http.get(f"/api/authz/tuples/by-subject/{tenant_id}/{subject_type}/{subject_id}")

    # --- Checks ---

    def check(
        self,
        tenant_id: str,
        subject_type: str,
        subject_id: str,
        relation: str,
        namespace: str,
        object_id: str,
        context: Optional[dict] = None,
    ) -> dict:
        body: dict[str, Any] = {
            "tenant_id": tenant_id,
            "subject_type": subject_type,
            "subject_id": subject_id,
            "relation": relation,
            "namespace": namespace,
            "object_id": object_id,
        }
        if context:
            body["context"] = context
        return self._http.post("/api/authz/check", json=body)

    def expand(self, tenant_id: str, namespace: str, object_id: str, relation: str) -> dict:
        return self._http.get(f"/api/authz/expand/{tenant_id}/{namespace}/{object_id}/{relation}")

    def forward_auth(self, **kwargs: Any) -> dict:
        return self._http.post("/authz/forward-auth", json=kwargs)

    # --- Stores ---

    def create_store(self, name: str, description: Optional[str] = None) -> dict:
        body: dict[str, Any] = {"name": name}
        if description:
            body["description"] = description
        return self._http.post("/api/fga/stores", json=body)

    def list_stores(self, include_inactive: Optional[bool] = None) -> dict:
        params: dict[str, Any] = {}
        if include_inactive is not None:
            params["include_inactive"] = include_inactive
        return self._http.get("/api/fga/stores", params=params or None)

    def get_store(self, store_id: str) -> dict:
        return self._http.get(f"/api/fga/stores/{store_id}")

    def update_store(self, store_id: str, **kwargs: Any) -> dict:
        return self._http.patch(f"/api/fga/stores/{store_id}", json=kwargs)

    def delete_store(self, store_id: str) -> dict:
        return self._http.delete(f"/api/fga/stores/{store_id}")

    # --- Models ---

    def write_model(self, store_id: str, schema: Any, created_by: Optional[str] = None) -> dict:
        body: dict[str, Any] = {"schema": schema}
        if created_by:
            body["created_by"] = created_by
        return self._http.post(f"/api/fga/stores/{store_id}/models", json=body)

    def list_models(self, store_id: str) -> dict:
        return self._http.get(f"/api/fga/stores/{store_id}/models")

    def get_current_model(self, store_id: str) -> dict:
        return self._http.get(f"/api/fga/stores/{store_id}/models/current")

    def get_model_version(self, store_id: str, version: str) -> dict:
        return self._http.get(f"/api/fga/stores/{store_id}/models/{version}")

    # --- API Keys ---

    def create_api_key(self, store_id: str, name: str, permissions: list[str], **kwargs: Any) -> dict:
        body: dict[str, Any] = {"name": name, "permissions": permissions, **kwargs}
        return self._http.post(f"/api/fga/stores/{store_id}/api-keys", json=body)

    def list_api_keys(self, store_id: str) -> dict:
        return self._http.get(f"/api/fga/stores/{store_id}/api-keys")

    def revoke_api_key(self, store_id: str, key_id: str) -> dict:
        return self._http.delete(f"/api/fga/stores/{store_id}/api-keys/{key_id}")

    # --- Store operations ---

    def store_check(self, store_id: str, **kwargs: Any) -> dict:
        return self._http.post(f"/api/fga/stores/{store_id}/check", json=kwargs)

    def read_store_tuples(self, store_id: str, **params: Any) -> dict:
        return self._http.get(f"/api/fga/stores/{store_id}/tuples", params=params or None)

    def write_store_tuples(self, store_id: str, **kwargs: Any) -> dict:
        return self._http.post(f"/api/fga/stores/{store_id}/tuples", json=kwargs)
