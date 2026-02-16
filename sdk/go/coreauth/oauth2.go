package coreauth

import (
	"context"
	"encoding/json"
	"net/url"
)

// OAuth2Service provides OAuth2 and OpenID Connect operations.
type OAuth2Service struct {
	http *httpClient
}

// Discovery retrieves the OpenID Connect discovery document.
func (s *OAuth2Service) Discovery(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/.well-known/openid-configuration", nil)
}

// JWKS retrieves the JSON Web Key Set used for token verification.
func (s *OAuth2Service) JWKS(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/.well-known/jwks.json", nil)
}

// AuthorizeURL constructs an OAuth2 authorization URL. This method does not
// make an HTTP request; it returns the fully-formed URL string.
func (s *OAuth2Service) AuthorizeURL(clientID, redirectURI string, params map[string]string) string {
	v := url.Values{}
	v.Set("client_id", clientID)
	v.Set("redirect_uri", redirectURI)
	v.Set("response_type", "code")
	for k, val := range params {
		if val != "" {
			v.Set(k, val)
		}
	}
	return s.http.baseURL + "/authorize?" + v.Encode()
}

// Token exchanges an authorization code or refresh token for tokens.
func (s *OAuth2Service) Token(ctx context.Context, data url.Values) (json.RawMessage, error) {
	return s.http.postForm(ctx, "/oauth/token", data)
}

// Userinfo retrieves the authenticated user's claims from the UserInfo endpoint.
func (s *OAuth2Service) Userinfo(ctx context.Context) (json.RawMessage, error) {
	return s.http.get(ctx, "/userinfo", nil)
}

// Revoke revokes an access or refresh token.
func (s *OAuth2Service) Revoke(ctx context.Context, token string, tokenTypeHint *string) (json.RawMessage, error) {
	data := url.Values{}
	data.Set("token", token)
	if tokenTypeHint != nil {
		data.Set("token_type_hint", *tokenTypeHint)
	}
	return s.http.postForm(ctx, "/oauth/revoke", data)
}

// Introspect inspects a token and returns its metadata.
func (s *OAuth2Service) Introspect(ctx context.Context, token string, tokenTypeHint *string) (json.RawMessage, error) {
	data := url.Values{}
	data.Set("token", token)
	if tokenTypeHint != nil {
		data.Set("token_type_hint", *tokenTypeHint)
	}
	return s.http.postForm(ctx, "/oauth/introspect", data)
}

// OidcLogout initiates an OIDC RP-Initiated Logout flow.
func (s *OAuth2Service) OidcLogout(ctx context.Context, params map[string]string) (json.RawMessage, error) {
	return s.http.get(ctx, "/logout", params)
}
