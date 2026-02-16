package coreauth

// TenantRegistryResponse represents a tenant entry in the registry.
type TenantRegistryResponse struct {
	ID            string  `json:"id"`
	Slug          string  `json:"slug"`
	Name          string  `json:"name"`
	Status        *string `json:"status,omitempty"`
	IsolationMode *string `json:"isolation_mode,omitempty"`
	CreatedAt     *string `json:"created_at,omitempty"`
	UpdatedAt     *string `json:"updated_at,omitempty"`
}

// CreateRegistryTenantRequest represents a request to create a tenant in the registry.
type CreateRegistryTenantRequest struct {
	Slug          string  `json:"slug"`
	Name          string  `json:"name"`
	IsolationMode *string `json:"isolation_mode,omitempty"`
}

// ConfigureDedicatedDbRequest represents a request to configure a dedicated database for a tenant.
type ConfigureDedicatedDbRequest struct {
	ConnectionString string `json:"connection_string"`
}

// TenantRouterStats represents statistics about the tenant router.
type TenantRouterStats struct {
	TotalTenants     int `json:"total_tenants"`
	ActiveTenants    int `json:"active_tenants"`
	SharedTenants    int `json:"shared_tenants"`
	DedicatedTenants int `json:"dedicated_tenants"`
}

// Action represents a tenant action/hook.
type Action struct {
	ID              string         `json:"id"`
	OrganizationID  string         `json:"organization_id"`
	Name            string         `json:"name"`
	Description     *string        `json:"description,omitempty"`
	TriggerType     string         `json:"trigger_type"`
	Code            string         `json:"code"`
	Runtime         *string        `json:"runtime,omitempty"`
	TimeoutSeconds  *int           `json:"timeout_seconds,omitempty"`
	IsEnabled       bool           `json:"is_enabled"`
	TotalExecutions *int64         `json:"total_executions,omitempty"`
	TotalFailures   *int64         `json:"total_failures,omitempty"`
	LastExecutedAt  *string        `json:"last_executed_at,omitempty"`
	CreatedAt       *string        `json:"created_at,omitempty"`
	UpdatedAt       *string        `json:"updated_at,omitempty"`
}

// CreateActionRequest represents a request to create an action.
type CreateActionRequest struct {
	Name           string         `json:"name"`
	TriggerType    string         `json:"trigger_type"`
	Code           string         `json:"code"`
	Description    *string        `json:"description,omitempty"`
	Runtime        *string        `json:"runtime,omitempty"`
	TimeoutSeconds *int           `json:"timeout_seconds,omitempty"`
	Secrets        map[string]any `json:"secrets,omitempty"`
	ExecutionOrder *int           `json:"execution_order,omitempty"`
}

// UpdateActionRequest represents a request to update an action.
type UpdateActionRequest struct {
	Name           *string        `json:"name,omitempty"`
	Description    *string        `json:"description,omitempty"`
	Code           *string        `json:"code,omitempty"`
	Runtime        *string        `json:"runtime,omitempty"`
	TimeoutSeconds *int           `json:"timeout_seconds,omitempty"`
	Secrets        map[string]any `json:"secrets,omitempty"`
	ExecutionOrder *int           `json:"execution_order,omitempty"`
	IsEnabled      *bool          `json:"is_enabled,omitempty"`
}

// ActionExecution represents a record of an action execution.
type ActionExecution struct {
	ID              string         `json:"id"`
	ActionID        string         `json:"action_id"`
	OrganizationID  string         `json:"organization_id"`
	TriggerType     string         `json:"trigger_type"`
	UserID          *string        `json:"user_id,omitempty"`
	Status          string         `json:"status"`
	ExecutionTimeMs *int64         `json:"execution_time_ms,omitempty"`
	InputData       map[string]any `json:"input_data,omitempty"`
	OutputData      map[string]any `json:"output_data,omitempty"`
	ErrorMessage    *string        `json:"error_message,omitempty"`
	ExecutedAt      *string        `json:"executed_at,omitempty"`
}

// ActionTestResponse represents the result of testing an action.
type ActionTestResponse struct {
	Success bool           `json:"success"`
	Data    map[string]any `json:"data,omitempty"`
	Error   *string        `json:"error,omitempty"`
}

// ConnectionTestResult represents the result of testing a connection.
type ConnectionTestResult struct {
	Success   bool    `json:"success"`
	Message   string  `json:"message"`
	LatencyMs *int64  `json:"latency_ms,omitempty"`
}

// HealthResponse represents the API health check response.
type HealthResponse struct {
	Status  string  `json:"status"`
	Version *string `json:"version,omitempty"`
}
