package coreauth

// CreateTupleRequest represents a request to create a relationship tuple.
type CreateTupleRequest struct {
	TenantID        string  `json:"tenant_id"`
	Namespace       string  `json:"namespace"`
	ObjectID        string  `json:"object_id"`
	Relation        string  `json:"relation"`
	SubjectType     string  `json:"subject_type"`
	SubjectID       string  `json:"subject_id"`
	SubjectRelation *string `json:"subject_relation,omitempty"`
}

// QueryTuplesRequest represents a request to query relationship tuples.
type QueryTuplesRequest struct {
	TenantID    string  `json:"tenant_id"`
	Namespace   *string `json:"namespace,omitempty"`
	ObjectID    *string `json:"object_id,omitempty"`
	Relation    *string `json:"relation,omitempty"`
	SubjectType *string `json:"subject_type,omitempty"`
	SubjectID   *string `json:"subject_id,omitempty"`
}

// RelationTuple represents a stored relationship tuple.
type RelationTuple struct {
	ID              string  `json:"id"`
	TenantID        string  `json:"tenant_id"`
	Namespace       string  `json:"namespace"`
	ObjectID        string  `json:"object_id"`
	Relation        string  `json:"relation"`
	SubjectType     string  `json:"subject_type"`
	SubjectID       string  `json:"subject_id"`
	SubjectRelation *string `json:"subject_relation,omitempty"`
	CreatedAt       *string `json:"created_at,omitempty"`
}

// CheckRequest represents a request to check a permission.
type CheckRequest struct {
	TenantID    string         `json:"tenant_id"`
	SubjectType string         `json:"subject_type"`
	SubjectID   string         `json:"subject_id"`
	Relation    string         `json:"relation"`
	Namespace   string         `json:"namespace"`
	ObjectID    string         `json:"object_id"`
	Context     map[string]any `json:"context,omitempty"`
}

// CheckResponse represents the result of a permission check.
type CheckResponse struct {
	Allowed bool    `json:"allowed"`
	Reason  *string `json:"reason,omitempty"`
}

// ExpandResponse represents the result of expanding a relation.
type ExpandResponse struct {
	Tree map[string]any `json:"tree"`
}

// ForwardAuthRequest represents a request for forward-auth style permission checks.
type ForwardAuthRequest struct {
	TenantID    string `json:"tenant_id"`
	SubjectType string `json:"subject_type"`
	SubjectID   string `json:"subject_id"`
	Relation    string `json:"relation"`
	Namespace   string `json:"namespace"`
	ObjectID    string `json:"object_id"`
}

// FgaStore represents an FGA store.
type FgaStore struct {
	ID                  string         `json:"id"`
	Name                string         `json:"name"`
	Description         *string        `json:"description,omitempty"`
	CurrentModelVersion int            `json:"current_model_version"`
	IsActive            bool           `json:"is_active"`
	TupleCount          int64          `json:"tuple_count"`
	Settings            map[string]any `json:"settings,omitempty"`
	CreatedAt           *string        `json:"created_at,omitempty"`
	UpdatedAt           *string        `json:"updated_at,omitempty"`
}

// CreateStoreRequest represents a request to create an FGA store.
type CreateStoreRequest struct {
	Name        string  `json:"name"`
	Description *string `json:"description,omitempty"`
}

// UpdateStoreRequest represents a request to update an FGA store.
type UpdateStoreRequest struct {
	Name        *string `json:"name,omitempty"`
	Description *string `json:"description,omitempty"`
	IsActive    *bool   `json:"is_active,omitempty"`
}

// WriteModelRequest represents a request to write an authorization model to an FGA store.
type WriteModelRequest struct {
	Schema    map[string]any `json:"schema"`
	CreatedBy *string        `json:"created_by,omitempty"`
}

// AuthorizationModel represents a stored authorization model.
type AuthorizationModel struct {
	ID               string         `json:"id"`
	StoreID          string         `json:"store_id"`
	Version          int            `json:"version"`
	SchemaJSON       map[string]any `json:"schema_json"`
	SchemaDSL        *string        `json:"schema_dsl,omitempty"`
	IsValid          bool           `json:"is_valid"`
	ValidationErrors []string       `json:"validation_errors,omitempty"`
	CreatedBy        *string        `json:"created_by,omitempty"`
	CreatedAt        *string        `json:"created_at,omitempty"`
}

// CreateApiKeyRequest represents a request to create an API key for an FGA store.
type CreateApiKeyRequest struct {
	Name               string   `json:"name"`
	Permissions        []string `json:"permissions"`
	RateLimitPerMinute *int     `json:"rate_limit_per_minute,omitempty"`
	ExpiresAt          *string  `json:"expires_at,omitempty"`
}

// FgaStoreApiKey represents an API key for an FGA store.
type FgaStoreApiKey struct {
	ID                 string   `json:"id"`
	StoreID            string   `json:"store_id"`
	Name               string   `json:"name"`
	KeyPrefix          string   `json:"key_prefix"`
	Permissions        []string `json:"permissions"`
	RateLimitPerMinute int      `json:"rate_limit_per_minute"`
	IsActive           bool     `json:"is_active"`
	LastUsedAt         *string  `json:"last_used_at,omitempty"`
	ExpiresAt          *string  `json:"expires_at,omitempty"`
	CreatedAt          *string  `json:"created_at,omitempty"`
	UpdatedAt          *string  `json:"updated_at,omitempty"`
}

// FgaStoreApiKeyWithSecret represents an API key with its secret exposed (returned only on creation).
type FgaStoreApiKeyWithSecret struct {
	FgaStoreApiKey
	Key string `json:"key"`
}

// WriteTuplesRequest represents a batch request to write and/or delete relationship tuples.
type WriteTuplesRequest struct {
	Writes  []map[string]any `json:"writes,omitempty"`
	Deletes []map[string]any `json:"deletes,omitempty"`
}
