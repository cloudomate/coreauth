package coreauth

import (
	"net/http"
	"time"
)

// Option configures the CoreAuth client.
type Option func(*Client)

// WithToken sets the initial bearer token.
func WithToken(token string) Option {
	return func(c *Client) {
		c.http.setToken(token)
	}
}

// WithHTTPClient sets a custom http.Client.
func WithHTTPClient(hc *http.Client) Option {
	return func(c *Client) {
		c.http.httpClient = hc
	}
}

// Client is the main CoreAuth SDK client.
type Client struct {
	http         *httpClient
	Auth         *AuthService
	OAuth2       *OAuth2Service
	Mfa          *MfaService
	Tenants      *TenantsService
	Applications *ApplicationsService
	Fga          *FgaService
	Audit        *AuditService
	Webhooks     *WebhooksService
	Groups       *GroupsService
	Scim         *ScimService
	Admin        *AdminService
	Connections  *ConnectionsService
}

// NewClient creates a new CoreAuth client.
func NewClient(baseURL string, opts ...Option) *Client {
	hc := newHTTPClient(baseURL, &http.Client{Timeout: 30 * time.Second})
	c := &Client{http: hc}
	for _, opt := range opts {
		opt(c)
	}
	c.Auth = &AuthService{http: hc}
	c.OAuth2 = &OAuth2Service{http: hc}
	c.Mfa = &MfaService{http: hc}
	c.Tenants = &TenantsService{http: hc}
	c.Applications = &ApplicationsService{http: hc}
	c.Fga = &FgaService{http: hc}
	c.Audit = &AuditService{http: hc}
	c.Webhooks = &WebhooksService{http: hc}
	c.Groups = &GroupsService{http: hc}
	c.Scim = &ScimService{http: hc}
	c.Admin = &AdminService{http: hc}
	c.Connections = &ConnectionsService{http: hc}
	return c
}

// SetToken updates the bearer token used for all requests.
func (c *Client) SetToken(token string) {
	c.http.setToken(token)
}

// ClearToken removes the bearer token.
func (c *Client) ClearToken() {
	c.http.clearToken()
}
