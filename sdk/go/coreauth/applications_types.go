package coreauth

// Application represents an OAuth2/OIDC application.
type Application struct {
	ID                          string   `json:"id"`
	OrganizationID              *string  `json:"organization_id,omitempty"`
	Name                        string   `json:"name"`
	Slug                        *string  `json:"slug,omitempty"`
	Description                 *string  `json:"description,omitempty"`
	AppType                     *string  `json:"app_type,omitempty"`
	ClientID                    string   `json:"client_id"`
	CallbackURLs                []string `json:"callback_urls"`
	LogoutURLs                  *string  `json:"logout_urls,omitempty"`
	WebOrigins                  *string  `json:"web_origins,omitempty"`
	GrantTypes                  *string  `json:"grant_types,omitempty"`
	AllowedScopes               *string  `json:"allowed_scopes,omitempty"`
	IsEnabled                   *bool    `json:"is_enabled,omitempty"`
	IsFirstParty                *bool    `json:"is_first_party,omitempty"`
	AccessTokenLifetimeSeconds  *int     `json:"access_token_lifetime_seconds,omitempty"`
	RefreshTokenLifetimeSeconds *int     `json:"refresh_token_lifetime_seconds,omitempty"`
	CreatedAt                   *string  `json:"created_at,omitempty"`
	UpdatedAt                   *string  `json:"updated_at,omitempty"`
}

// ApplicationWithSecret represents an application with its client secret exposed.
type ApplicationWithSecret struct {
	Application
	ClientSecretPlain string `json:"client_secret_plain"`
}

// CreateOAuthAppRequest represents a request to create an OAuth2 application.
type CreateOAuthAppRequest struct {
	Name                        string   `json:"name"`
	Slug                        string   `json:"slug"`
	AppType                     string   `json:"app_type"`
	CallbackURLs                []string `json:"callback_urls"`
	Description                 *string  `json:"description,omitempty"`
	LogoURL                     *string  `json:"logo_url,omitempty"`
	LogoutURLs                  *string  `json:"logout_urls,omitempty"`
	WebOrigins                  *string  `json:"web_origins,omitempty"`
	AccessTokenLifetimeSeconds  *int     `json:"access_token_lifetime_seconds,omitempty"`
	RefreshTokenLifetimeSeconds *int     `json:"refresh_token_lifetime_seconds,omitempty"`
	GrantTypes                  *string  `json:"grant_types,omitempty"`
	AllowedScopes               *string  `json:"allowed_scopes,omitempty"`
}

// UpdateApplicationRequest represents a request to update an application.
type UpdateApplicationRequest struct {
	Name         *string  `json:"name,omitempty"`
	Description  *string  `json:"description,omitempty"`
	CallbackURLs []string `json:"callback_urls,omitempty"`
	LogoutURLs   *string  `json:"logout_urls,omitempty"`
	IsEnabled    *bool    `json:"is_enabled,omitempty"`
}

// EmailTemplate represents a customizable email template.
type EmailTemplate struct {
	TemplateType string         `json:"template_type"`
	Subject      *string        `json:"subject,omitempty"`
	HTMLBody     *string        `json:"html_body,omitempty"`
	TextBody     *string        `json:"text_body,omitempty"`
	Variables    map[string]any `json:"variables,omitempty"`
	IsCustom     *bool          `json:"is_custom,omitempty"`
	CreatedAt    *string        `json:"created_at,omitempty"`
	UpdatedAt    *string        `json:"updated_at,omitempty"`
}

// AuthenticateAppRequest represents a request to authenticate an application via client credentials.
type AuthenticateAppRequest struct {
	ClientID     string `json:"client_id"`
	ClientSecret string `json:"client_secret"`
}
