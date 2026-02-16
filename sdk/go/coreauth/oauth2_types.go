package coreauth

// TokenResponse represents an OAuth2 token response.
type TokenResponse struct {
	AccessToken  string  `json:"access_token"`
	TokenType    string  `json:"token_type"`
	ExpiresIn    int     `json:"expires_in"`
	RefreshToken *string `json:"refresh_token,omitempty"`
	IDToken      *string `json:"id_token,omitempty"`
	Scope        *string `json:"scope,omitempty"`
}

// UserInfoResponse represents the OIDC UserInfo endpoint response.
type UserInfoResponse struct {
	Sub           string  `json:"sub"`
	Name          *string `json:"name,omitempty"`
	GivenName     *string `json:"given_name,omitempty"`
	FamilyName    *string `json:"family_name,omitempty"`
	Email         *string `json:"email,omitempty"`
	EmailVerified *bool   `json:"email_verified,omitempty"`
	Picture       *string `json:"picture,omitempty"`
	Locale        *string `json:"locale,omitempty"`
	UpdatedAt     *string `json:"updated_at,omitempty"`
	OrgID         *string `json:"org_id,omitempty"`
	OrgName       *string `json:"org_name,omitempty"`
}

// IntrospectionResponse represents an OAuth2 token introspection response.
type IntrospectionResponse struct {
	Active    bool    `json:"active"`
	Scope     *string `json:"scope,omitempty"`
	ClientID  *string `json:"client_id,omitempty"`
	Username  *string `json:"username,omitempty"`
	TokenType *string `json:"token_type,omitempty"`
	Exp       *int64  `json:"exp,omitempty"`
	Iat       *int64  `json:"iat,omitempty"`
	Sub       *string `json:"sub,omitempty"`
	Aud       *string `json:"aud,omitempty"`
	Iss       *string `json:"iss,omitempty"`
	Jti       *string `json:"jti,omitempty"`
}

// OidcDiscovery represents the OIDC Discovery document.
type OidcDiscovery struct {
	Issuer                string         `json:"issuer"`
	AuthorizationEndpoint string         `json:"authorization_endpoint"`
	TokenEndpoint         string         `json:"token_endpoint"`
	UserinfoEndpoint      *string        `json:"userinfo_endpoint,omitempty"`
	JwksURI               string         `json:"jwks_uri"`
	Additional            map[string]any `json:"-"`
}

// Jwks represents a JSON Web Key Set.
type Jwks struct {
	Keys []map[string]any `json:"keys"`
}
