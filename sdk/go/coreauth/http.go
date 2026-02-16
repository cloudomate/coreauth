package coreauth

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
)

type httpClient struct {
	baseURL    string
	token      string
	httpClient *http.Client
}

func newHTTPClient(baseURL string, hc *http.Client) *httpClient {
	return &httpClient{
		baseURL:    strings.TrimRight(baseURL, "/"),
		httpClient: hc,
	}
}

func (c *httpClient) setToken(token string) {
	c.token = token
}

func (c *httpClient) clearToken() {
	c.token = ""
}

func (c *httpClient) doRequest(ctx context.Context, method, path string, body io.Reader, contentType string) (json.RawMessage, error) {
	u := c.baseURL + path
	req, err := http.NewRequestWithContext(ctx, method, u, body)
	if err != nil {
		return nil, &CoreAuthError{Message: fmt.Sprintf("failed to create request: %v", err)}
	}
	if contentType != "" {
		req.Header.Set("Content-Type", contentType)
	}
	if c.token != "" {
		req.Header.Set("Authorization", "Bearer "+c.token)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, &CoreAuthError{Message: fmt.Sprintf("request failed: %v", err)}
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, &CoreAuthError{Message: fmt.Sprintf("failed to read response: %v", err)}
	}

	if resp.StatusCode == 204 {
		return nil, nil
	}
	if resp.StatusCode >= 200 && resp.StatusCode < 300 {
		if len(respBody) == 0 {
			return nil, nil
		}
		return json.RawMessage(respBody), nil
	}

	// Parse error
	apiErr := &ApiError{StatusCode: resp.StatusCode}
	var errBody struct {
		Error   string `json:"error"`
		Message string `json:"message"`
	}
	if json.Unmarshal(respBody, &errBody) == nil {
		apiErr.ErrorCode = errBody.Error
		apiErr.Message = errBody.Message
	} else {
		apiErr.Message = string(respBody)
	}
	return nil, apiErr
}

func (c *httpClient) get(ctx context.Context, path string, params map[string]string) (json.RawMessage, error) {
	if len(params) > 0 {
		v := url.Values{}
		for k, val := range params {
			if val != "" {
				v.Set(k, val)
			}
		}
		if encoded := v.Encode(); encoded != "" {
			path = path + "?" + encoded
		}
	}
	return c.doRequest(ctx, http.MethodGet, path, nil, "application/json")
}

func (c *httpClient) post(ctx context.Context, path string, payload any) (json.RawMessage, error) {
	var body io.Reader
	if payload != nil {
		b, err := json.Marshal(payload)
		if err != nil {
			return nil, &CoreAuthError{Message: fmt.Sprintf("failed to marshal request: %v", err)}
		}
		body = bytes.NewReader(b)
	}
	return c.doRequest(ctx, http.MethodPost, path, body, "application/json")
}

func (c *httpClient) postForm(ctx context.Context, path string, data url.Values) (json.RawMessage, error) {
	return c.doRequest(ctx, http.MethodPost, path, strings.NewReader(data.Encode()), "application/x-www-form-urlencoded")
}

func (c *httpClient) put(ctx context.Context, path string, payload any) (json.RawMessage, error) {
	var body io.Reader
	if payload != nil {
		b, err := json.Marshal(payload)
		if err != nil {
			return nil, &CoreAuthError{Message: fmt.Sprintf("failed to marshal request: %v", err)}
		}
		body = bytes.NewReader(b)
	}
	return c.doRequest(ctx, http.MethodPut, path, body, "application/json")
}

func (c *httpClient) patch(ctx context.Context, path string, payload any) (json.RawMessage, error) {
	var body io.Reader
	if payload != nil {
		b, err := json.Marshal(payload)
		if err != nil {
			return nil, &CoreAuthError{Message: fmt.Sprintf("failed to marshal request: %v", err)}
		}
		body = bytes.NewReader(b)
	}
	return c.doRequest(ctx, http.MethodPatch, path, body, "application/json")
}

func (c *httpClient) del(ctx context.Context, path string, payload any) (json.RawMessage, error) {
	var body io.Reader
	if payload != nil {
		b, err := json.Marshal(payload)
		if err != nil {
			return nil, &CoreAuthError{Message: fmt.Sprintf("failed to marshal request: %v", err)}
		}
		body = bytes.NewReader(b)
	}
	return c.doRequest(ctx, http.MethodDelete, path, body, "application/json")
}
