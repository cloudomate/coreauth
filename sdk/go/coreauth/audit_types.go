package coreauth

// AuditLog represents a single audit log entry.
type AuditLog struct {
	ID             string         `json:"id"`
	TenantID       string         `json:"tenant_id"`
	EventType      string         `json:"event_type"`
	EventCategory  *string        `json:"event_category,omitempty"`
	EventAction    *string        `json:"event_action,omitempty"`
	ActorType      *string        `json:"actor_type,omitempty"`
	ActorID        *string        `json:"actor_id,omitempty"`
	ActorName      *string        `json:"actor_name,omitempty"`
	ActorIPAddress *string        `json:"actor_ip_address,omitempty"`
	TargetType     *string        `json:"target_type,omitempty"`
	TargetID       *string        `json:"target_id,omitempty"`
	TargetName     *string        `json:"target_name,omitempty"`
	Description    *string        `json:"description,omitempty"`
	Metadata       map[string]any `json:"metadata,omitempty"`
	Status         *string        `json:"status,omitempty"`
	ErrorMessage   *string        `json:"error_message,omitempty"`
	CreatedAt      *string        `json:"created_at,omitempty"`
}

// AuditLogsResponse represents a paginated list of audit logs.
type AuditLogsResponse struct {
	Logs   []AuditLog `json:"logs"`
	Total  int        `json:"total"`
	Limit  int        `json:"limit"`
	Offset int        `json:"offset"`
}

// AuditStats is a type alias for audit statistics, represented as a flexible map.
type AuditStats = map[string]any
