package coreauth

import (
	"context"
	"encoding/json"
	"fmt"
)

// FgaService provides Fine-Grained Authorization (OpenFGA-compatible) operations.
type FgaService struct {
	http *httpClient
}

// --- Tuples ---

// CreateTuple creates a new authorization tuple (relationship).
func (s *FgaService) CreateTuple(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/fga/tuples", data)
}

// DeleteTuple removes an authorization tuple.
func (s *FgaService) DeleteTuple(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/fga/tuples/delete", data)
}

// QueryTuples queries authorization tuples with optional filters.
func (s *FgaService) QueryTuples(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/fga/tuples", params)
}

// GetObjectTuples returns all tuples for a specific object.
func (s *FgaService) GetObjectTuples(ctx context.Context, objectType, objectID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/fga/objects/%s:%s/tuples", objectType, objectID), nil)
}

// GetSubjectTuples returns all tuples for a specific subject.
func (s *FgaService) GetSubjectTuples(ctx context.Context, subjectType, subjectID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/fga/subjects/%s:%s/tuples", subjectType, subjectID), nil)
}

// --- Checks ---

// Check evaluates whether a subject has a specific relation on an object.
func (s *FgaService) Check(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/fga/check", data)
}

// Expand returns the expansion tree for a relation on an object.
func (s *FgaService) Expand(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/fga/expand", data)
}

// ForwardAuth performs a permission check optimized for reverse-proxy forward-auth patterns.
func (s *FgaService) ForwardAuth(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/fga/forward-auth", data)
}

// --- Stores ---

// CreateStore creates a new FGA store.
func (s *FgaService) CreateStore(ctx context.Context, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, "/api/fga/stores", data)
}

// ListStores returns all FGA stores.
func (s *FgaService) ListStores(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/api/fga/stores", nil)
}

// GetStore retrieves an FGA store by ID.
func (s *FgaService) GetStore(ctx context.Context, storeID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/fga/stores/%s", storeID), nil)
}

// UpdateStore updates an FGA store.
func (s *FgaService) UpdateStore(ctx context.Context, storeID string, data map[string]any) (json.RawMessage, error) {
	return s.http.put(ctx, fmt.Sprintf("/api/fga/stores/%s", storeID), data)
}

// DeleteStore removes an FGA store.
func (s *FgaService) DeleteStore(ctx context.Context, storeID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/fga/stores/%s", storeID), nil)
	return err
}

// --- Models ---

// WriteModel writes an authorization model to a store.
func (s *FgaService) WriteModel(ctx context.Context, storeID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/fga/stores/%s/models", storeID), data)
}

// ListModels returns all authorization model versions for a store.
func (s *FgaService) ListModels(ctx context.Context, storeID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/fga/stores/%s/models", storeID), nil)
}

// GetCurrentModel retrieves the current (active) authorization model for a store.
func (s *FgaService) GetCurrentModel(ctx context.Context, storeID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/fga/stores/%s/models/current", storeID), nil)
}

// GetModelVersion retrieves a specific authorization model version.
func (s *FgaService) GetModelVersion(ctx context.Context, storeID, modelID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/fga/stores/%s/models/%s", storeID, modelID), nil)
}

// --- API Keys ---

// CreateAPIKey creates a new API key for an FGA store.
func (s *FgaService) CreateAPIKey(ctx context.Context, storeID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/fga/stores/%s/api-keys", storeID), data)
}

// ListAPIKeys returns all API keys for an FGA store.
func (s *FgaService) ListAPIKeys(ctx context.Context, storeID string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/fga/stores/%s/api-keys", storeID), nil)
}

// RevokeAPIKey revokes an API key for an FGA store.
func (s *FgaService) RevokeAPIKey(ctx context.Context, storeID, keyID string) error {
	_, err := s.http.del(ctx, fmt.Sprintf("/api/fga/stores/%s/api-keys/%s", storeID, keyID), nil)
	return err
}

// --- Store-scoped Operations ---

// StoreCheck performs an authorization check within a specific store context.
func (s *FgaService) StoreCheck(ctx context.Context, storeID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/fga/stores/%s/check", storeID), data)
}

// ReadStoreTuples reads tuples from a specific store.
func (s *FgaService) ReadStoreTuples(ctx context.Context, storeID string, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, fmt.Sprintf("/api/fga/stores/%s/tuples", storeID), params)
}

// WriteStoreTuples writes tuples to a specific store.
func (s *FgaService) WriteStoreTuples(ctx context.Context, storeID string, data map[string]any) (json.RawMessage, error) {
	return s.http.post(ctx, fmt.Sprintf("/api/fga/stores/%s/tuples", storeID), data)
}
