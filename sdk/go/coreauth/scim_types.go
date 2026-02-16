package coreauth

// ScimUser represents a SCIM 2.0 user resource.
type ScimUser struct {
	Schemas      []string           `json:"schemas,omitempty"`
	ID           string             `json:"id"`
	ExternalID   *string            `json:"externalId,omitempty"`
	UserName     string             `json:"userName"`
	Name         map[string]any     `json:"name,omitempty"`
	DisplayName  *string            `json:"displayName,omitempty"`
	Emails       []map[string]any   `json:"emails,omitempty"`
	PhoneNumbers []map[string]any   `json:"phoneNumbers,omitempty"`
	Active       *bool              `json:"active,omitempty"`
	Groups       []map[string]any   `json:"groups,omitempty"`
	Meta         map[string]any     `json:"meta,omitempty"`
}

// CreateScimUserRequest represents a request to create a SCIM user.
type CreateScimUserRequest struct {
	Schemas      []string         `json:"schemas,omitempty"`
	ExternalID   *string          `json:"externalId,omitempty"`
	UserName     string           `json:"userName"`
	Name         map[string]any   `json:"name"`
	DisplayName  *string          `json:"displayName,omitempty"`
	Emails       []map[string]any `json:"emails"`
	PhoneNumbers []map[string]any `json:"phoneNumbers,omitempty"`
	Active       *bool            `json:"active,omitempty"`
	Password     *string          `json:"password,omitempty"`
}

// ScimGroup represents a SCIM 2.0 group resource.
type ScimGroup struct {
	Schemas     []string         `json:"schemas,omitempty"`
	ID          string           `json:"id"`
	ExternalID  *string          `json:"externalId,omitempty"`
	DisplayName string           `json:"displayName"`
	Members     []map[string]any `json:"members,omitempty"`
	Meta        map[string]any   `json:"meta,omitempty"`
}

// CreateScimGroupRequest represents a request to create a SCIM group.
type CreateScimGroupRequest struct {
	Schemas     []string         `json:"schemas,omitempty"`
	ExternalID  *string          `json:"externalId,omitempty"`
	DisplayName string           `json:"displayName"`
	Members     []map[string]any `json:"members,omitempty"`
}

// ScimPatchRequest represents a SCIM PATCH operation request.
type ScimPatchRequest struct {
	Schemas    []string      `json:"schemas,omitempty"`
	Operations []ScimPatchOp `json:"Operations"`
}

// ScimPatchOp represents a single SCIM PATCH operation.
type ScimPatchOp struct {
	Op    string  `json:"op"`
	Path  *string `json:"path,omitempty"`
	Value any     `json:"value,omitempty"`
}

// ScimListResponse represents a SCIM 2.0 list response.
type ScimListResponse struct {
	Schemas      []string         `json:"schemas,omitempty"`
	TotalResults int              `json:"totalResults"`
	ItemsPerPage int              `json:"itemsPerPage"`
	StartIndex   int              `json:"startIndex"`
	Resources    []map[string]any `json:"Resources"`
}

// ScimTokenResponse represents a SCIM provisioning token.
type ScimTokenResponse struct {
	ID          string  `json:"id"`
	Name        string  `json:"name"`
	TokenPrefix string  `json:"token_prefix"`
	ExpiresAt   *string `json:"expires_at,omitempty"`
	CreatedAt   *string `json:"created_at,omitempty"`
}

// ScimTokenWithSecret represents a SCIM token with its secret exposed (returned only on creation).
type ScimTokenWithSecret struct {
	ScimTokenResponse
	Secret string `json:"secret"`
}

// CreateScimTokenRequest represents a request to create a SCIM provisioning token.
type CreateScimTokenRequest struct {
	Name      string  `json:"name"`
	ExpiresAt *string `json:"expires_at,omitempty"`
}

// SessionInfo represents information about a user session.
type SessionInfo struct {
	ID              string  `json:"id"`
	UserID          *string `json:"user_id,omitempty"`
	IPAddress       *string `json:"ip_address,omitempty"`
	UserAgent       *string `json:"user_agent,omitempty"`
	AuthenticatedAt *string `json:"authenticated_at,omitempty"`
	LastActiveAt    *string `json:"last_active_at,omitempty"`
	ExpiresAt       *string `json:"expires_at,omitempty"`
	CreatedAt       *string `json:"created_at,omitempty"`
}

// OidcProvider represents an OIDC identity provider configuration.
type OidcProvider struct {
	ID                    string         `json:"id"`
	TenantID              string         `json:"tenant_id"`
	Name                  string         `json:"name"`
	ProviderType          string         `json:"provider_type"`
	Issuer                string         `json:"issuer"`
	ClientID              string         `json:"client_id"`
	AuthorizationEndpoint *string        `json:"authorization_endpoint,omitempty"`
	TokenEndpoint         *string        `json:"token_endpoint,omitempty"`
	UserinfoEndpoint      *string        `json:"userinfo_endpoint,omitempty"`
	JwksURI               *string        `json:"jwks_uri,omitempty"`
	Scopes                []string       `json:"scopes,omitempty"`
	GroupsClaim           *string        `json:"groups_claim,omitempty"`
	GroupRoleMappings     map[string]any `json:"group_role_mappings,omitempty"`
	AllowedGroupID        *string        `json:"allowed_group_id,omitempty"`
	IsEnabled             bool           `json:"is_enabled"`
	CreatedAt             *string        `json:"created_at,omitempty"`
	UpdatedAt             *string        `json:"updated_at,omitempty"`
}

// CreateOidcProviderRequest represents a request to create an OIDC provider.
type CreateOidcProviderRequest struct {
	TenantID              string         `json:"tenant_id"`
	Name                  string         `json:"name"`
	ProviderType          string         `json:"provider_type"`
	Issuer                string         `json:"issuer"`
	ClientID              string         `json:"client_id"`
	ClientSecret          string         `json:"client_secret"`
	AuthorizationEndpoint string         `json:"authorization_endpoint"`
	TokenEndpoint         string         `json:"token_endpoint"`
	UserinfoEndpoint      *string        `json:"userinfo_endpoint,omitempty"`
	JwksURI               string         `json:"jwks_uri"`
	Scopes                []string       `json:"scopes,omitempty"`
	GroupsClaim           *string        `json:"groups_claim,omitempty"`
	GroupRoleMappings     map[string]any `json:"group_role_mappings,omitempty"`
	AllowedGroupID        *string        `json:"allowed_group_id,omitempty"`
}

// UpdateOidcProviderRequest represents a request to update an OIDC provider.
type UpdateOidcProviderRequest struct {
	IsEnabled    *bool   `json:"is_enabled,omitempty"`
	Name         *string `json:"name,omitempty"`
	ClientID     *string `json:"client_id,omitempty"`
	ClientSecret *string `json:"client_secret,omitempty"`
}

// SsoCheckResponse represents the result of an SSO availability check.
type SsoCheckResponse struct {
	HasSSO    bool               `json:"has_sso"`
	Providers []map[string]any   `json:"providers"`
}
