package coreauth

// Connection represents an authentication connection (database, OIDC, SAML, OAuth2, social).
type Connection struct {
	ID             string         `json:"id"`
	Name           string         `json:"name"`
	ConnectionType string         `json:"connection_type"`
	Scope          string         `json:"scope"`
	OrganizationID *string        `json:"organization_id,omitempty"`
	Config         map[string]any `json:"config,omitempty"`
	IsEnabled      bool           `json:"is_enabled"`
	CreatedAt      *string        `json:"created_at,omitempty"`
	UpdatedAt      *string        `json:"updated_at,omitempty"`
}

// CreateConnectionRequest represents a request to create a connection.
type CreateConnectionRequest struct {
	Name           string         `json:"name"`
	ConnectionType string         `json:"connection_type"`
	Config         map[string]any `json:"config,omitempty"`
}

// UpdateConnectionRequest represents a request to update a connection.
type UpdateConnectionRequest struct {
	Name      *string        `json:"name,omitempty"`
	Config    map[string]any `json:"config,omitempty"`
	IsEnabled *bool          `json:"is_enabled,omitempty"`
}

// AuthMethod represents an available authentication method for login.
type AuthMethod struct {
	ConnectionID string `json:"connection_id"`
	Name         string `json:"name"`
	MethodType   string `json:"method_type"`
	Scope        string `json:"scope"`
}
