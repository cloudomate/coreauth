/**
 * CoreAuth JavaScript SDK
 *
 * A complete OAuth2/OIDC client SDK with PKCE support for integrating
 * CoreAuth authentication into web applications.
 *
 * @version 2.0.0
 *
 * @example
 * // Initialize the SDK
 * const auth = new CoreAuth({
 *   domain: 'https://auth.example.com',
 *   clientId: 'your-client-id',
 *   redirectUri: 'https://yourapp.com/callback'
 * });
 *
 * // Login with redirect
 * await auth.loginWithRedirect();
 *
 * // Handle callback
 * await auth.handleRedirectCallback();
 *
 * // Get user
 * const user = await auth.getUser();
 */

class CoreAuth {
  /**
   * Create a CoreAuth client
   * @param {Object} config - Configuration options
   * @param {string} config.domain - CoreAuth domain (e.g., 'https://auth.example.com')
   * @param {string} config.clientId - OAuth2 client ID
   * @param {string} config.redirectUri - OAuth2 redirect URI
   * @param {string} [config.audience] - API audience for access tokens
   * @param {string} [config.scope='openid profile email'] - OAuth2 scopes
   * @param {string} [config.organization] - Default organization
   * @param {boolean} [config.useRefreshTokens=true] - Use refresh tokens
   * @param {number} [config.cacheExpiryBuffer=60] - Seconds before expiry to refresh
   */
  constructor(config) {
    if (!config.domain) throw new Error('domain is required');
    if (!config.clientId) throw new Error('clientId is required');
    if (!config.redirectUri) throw new Error('redirectUri is required');

    this.domain = config.domain.replace(/\/$/, ''); // Remove trailing slash
    this.clientId = config.clientId;
    this.redirectUri = config.redirectUri;
    this.audience = config.audience;
    this.scope = config.scope || 'openid profile email';
    this.organization = config.organization;
    this.useRefreshTokens = config.useRefreshTokens !== false;
    this.cacheExpiryBuffer = config.cacheExpiryBuffer || 60;

    // Storage keys
    this._storagePrefix = `coreauth_${this.clientId}`;

    // Token cache
    this._tokenCache = null;
  }

  // ============================================================================
  // Public API - Authentication
  // ============================================================================

  /**
   * Redirect to the CoreAuth Universal Login page
   * @param {Object} [options] - Login options
   * @param {string} [options.organization] - Organization slug
   * @param {string} [options.connection] - Connection name to use
   * @param {string} [options.loginHint] - Pre-fill email
   * @param {string} [options.prompt] - 'login', 'consent', 'none'
   * @param {Object} [options.appState] - State to pass through callback
   */
  async loginWithRedirect(options = {}) {
    const { codeVerifier, codeChallenge } = await this._generatePKCE();
    const state = this._generateState();
    const nonce = this._generateNonce();

    // Store PKCE and state for callback
    this._setStorageItem('pkce_code_verifier', codeVerifier);
    this._setStorageItem('auth_state', state);
    this._setStorageItem('auth_nonce', nonce);

    if (options.appState) {
      this._setStorageItem('app_state', JSON.stringify(options.appState));
    }

    const params = new URLSearchParams({
      client_id: this.clientId,
      redirect_uri: this.redirectUri,
      response_type: 'code',
      scope: this._getScope(options),
      state,
      nonce,
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
    });

    // Optional parameters
    if (options.organization || this.organization) {
      params.append('organization', options.organization || this.organization);
    }
    if (options.connection) {
      params.append('connection', options.connection);
    }
    if (options.loginHint) {
      params.append('login_hint', options.loginHint);
    }
    if (options.prompt) {
      params.append('prompt', options.prompt);
    }
    if (this.audience) {
      params.append('audience', this.audience);
    }

    window.location.href = `${this.domain}/authorize?${params}`;
  }

  /**
   * Handle the OAuth2 callback after redirect
   * @param {string} [url] - Optional URL to parse (defaults to current URL)
   * @returns {Promise<Object>} Result with appState
   */
  async handleRedirectCallback(url = window.location.href) {
    const urlObj = new URL(url);
    const params = new URLSearchParams(urlObj.search);

    const code = params.get('code');
    const state = params.get('state');
    const error = params.get('error');
    const errorDescription = params.get('error_description');

    // Handle errors
    if (error) {
      this._clearAuthState();
      throw new AuthenticationError(error, errorDescription);
    }

    if (!code) {
      throw new AuthenticationError('missing_code', 'No authorization code in callback URL');
    }

    // Verify state
    const savedState = this._getStorageItem('auth_state');
    if (!savedState || state !== savedState) {
      this._clearAuthState();
      throw new AuthenticationError('state_mismatch', 'State parameter does not match');
    }

    // Get PKCE verifier
    const codeVerifier = this._getStorageItem('pkce_code_verifier');
    if (!codeVerifier) {
      this._clearAuthState();
      throw new AuthenticationError('missing_verifier', 'PKCE code verifier not found');
    }

    // Exchange code for tokens
    const tokens = await this._exchangeCodeForTokens(code, codeVerifier);

    // Verify ID token nonce
    const savedNonce = this._getStorageItem('auth_nonce');
    if (tokens.id_token && savedNonce) {
      const payload = this._parseJWT(tokens.id_token);
      if (payload && payload.nonce !== savedNonce) {
        this._clearAuthState();
        throw new AuthenticationError('nonce_mismatch', 'ID token nonce does not match');
      }
    }

    // Cache tokens
    this._cacheTokens(tokens);

    // Get app state
    const appStateStr = this._getStorageItem('app_state');
    const appState = appStateStr ? JSON.parse(appStateStr) : undefined;

    // Clear auth state
    this._clearAuthState();

    // Clean up URL
    window.history.replaceState({}, document.title, urlObj.pathname);

    return { appState };
  }

  /**
   * Check if the user is authenticated
   * @returns {Promise<boolean>}
   */
  async isAuthenticated() {
    try {
      const token = await this.getAccessToken();
      return !!token;
    } catch {
      return false;
    }
  }

  /**
   * Get the current access token (refreshing if needed)
   * @param {Object} [options] - Options
   * @param {boolean} [options.ignoreCache] - Force token refresh
   * @returns {Promise<string|null>}
   */
  async getAccessToken(options = {}) {
    // Check cache
    if (!options.ignoreCache && this._tokenCache) {
      const expiresAt = this._tokenCache.expires_at || 0;
      const buffer = this.cacheExpiryBuffer * 1000;

      if (Date.now() + buffer < expiresAt) {
        return this._tokenCache.access_token;
      }
    }

    // Try to refresh
    if (this.useRefreshTokens) {
      const refreshToken = this._getStorageItem('refresh_token');
      if (refreshToken) {
        try {
          const tokens = await this._refreshAccessToken(refreshToken);
          this._cacheTokens(tokens);
          return tokens.access_token;
        } catch (e) {
          // Refresh failed, clear cache
          this._clearTokenCache();
        }
      }
    }

    return this._tokenCache?.access_token || null;
  }

  /**
   * Get the ID token claims
   * @returns {Promise<Object|null>}
   */
  async getIdTokenClaims() {
    const idToken = this._tokenCache?.id_token;
    if (!idToken) return null;
    return this._parseJWT(idToken);
  }

  /**
   * Get the current user
   * @returns {Promise<Object|null>}
   */
  async getUser() {
    // First try ID token claims
    const claims = await this.getIdTokenClaims();
    if (claims) {
      return {
        sub: claims.sub,
        email: claims.email,
        email_verified: claims.email_verified,
        name: claims.name,
        picture: claims.picture,
        org_id: claims.org_id,
        ...claims,
      };
    }

    // Fall back to userinfo endpoint
    const accessToken = await this.getAccessToken();
    if (!accessToken) return null;

    try {
      const response = await fetch(`${this.domain}/userinfo`, {
        headers: {
          Authorization: `Bearer ${accessToken}`,
        },
      });

      if (!response.ok) {
        if (response.status === 401) {
          this._clearTokenCache();
        }
        return null;
      }

      return response.json();
    } catch {
      return null;
    }
  }

  /**
   * Logout the user
   * @param {Object} [options] - Logout options
   * @param {string} [options.returnTo] - URL to redirect after logout
   * @param {boolean} [options.federated] - Also logout from IdP
   */
  async logout(options = {}) {
    // Clear local state
    this._clearTokenCache();

    // Build logout URL
    const params = new URLSearchParams({
      client_id: this.clientId,
    });

    if (options.returnTo) {
      params.append('returnTo', options.returnTo);
    }

    // Redirect to logout endpoint
    window.location.href = `${this.domain}/logout?${params}`;
  }

  // ============================================================================
  // Public API - Direct Authentication (for native apps)
  // ============================================================================

  /**
   * Login with email and password directly
   * Note: This bypasses the Universal Login and should only be used
   * for native applications or when specifically required.
   *
   * @param {Object} credentials - Login credentials
   * @param {string} credentials.email - User email
   * @param {string} credentials.password - User password
   * @param {string} [credentials.organization] - Organization slug
   * @returns {Promise<Object>}
   */
  async loginWithCredentials(credentials) {
    const response = await fetch(`${this.domain}/api/auth/login-hierarchical`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: credentials.email,
        password: credentials.password,
        organization_slug: credentials.organization || this.organization,
      }),
    });

    const data = await response.json();

    if (!response.ok) {
      throw new AuthenticationError(
        data.error || 'login_failed',
        data.message || 'Login failed'
      );
    }

    // Handle MFA required
    if (data.enrollment_token) {
      return {
        requiresMFA: true,
        enrollmentToken: data.enrollment_token,
        canSkip: data.can_skip,
        gracePeriodExpires: data.grace_period_expires,
      };
    }

    // Cache tokens
    this._cacheTokens(data);

    return {
      user: data.user,
    };
  }

  /**
   * Enroll in TOTP MFA
   * @param {string} enrollmentToken - Token from loginWithCredentials
   * @returns {Promise<Object>} QR code URL and secret
   */
  async enrollTOTP(enrollmentToken) {
    const response = await fetch(`${this.domain}/api/mfa/enroll-with-token/totp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ enrollment_token: enrollmentToken }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new AuthenticationError('mfa_enrollment_failed', error.message);
    }

    return response.json();
  }

  /**
   * Verify TOTP code
   * @param {string} enrollmentToken - Enrollment token
   * @param {string} methodId - MFA method ID
   * @param {string} code - TOTP code
   * @returns {Promise<Object>}
   */
  async verifyTOTP(enrollmentToken, methodId, code) {
    const response = await fetch(
      `${this.domain}/api/mfa/verify-with-token/totp/${methodId}`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          enrollment_token: enrollmentToken,
          code,
        }),
      }
    );

    if (!response.ok) {
      const error = await response.json();
      throw new AuthenticationError('mfa_verification_failed', error.message);
    }

    const data = await response.json();
    this._cacheTokens(data);

    return { user: data.user };
  }

  // ============================================================================
  // Public API - Utilities
  // ============================================================================

  /**
   * Create an authenticated fetch function
   * @returns {Function} Fetch function with automatic token handling
   */
  createAuthenticatedFetch() {
    return async (url, options = {}) => {
      const accessToken = await this.getAccessToken();

      if (!accessToken) {
        throw new AuthenticationError('not_authenticated', 'No access token available');
      }

      const response = await fetch(url, {
        ...options,
        headers: {
          ...options.headers,
          Authorization: `Bearer ${accessToken}`,
        },
      });

      // Handle 401 by refreshing and retrying once
      if (response.status === 401 && this.useRefreshTokens) {
        const newToken = await this.getAccessToken({ ignoreCache: true });
        if (newToken) {
          return fetch(url, {
            ...options,
            headers: {
              ...options.headers,
              Authorization: `Bearer ${newToken}`,
            },
          });
        }
      }

      return response;
    };
  }

  /**
   * Get the authorization URL without redirecting
   * @param {Object} [options] - Same options as loginWithRedirect
   * @returns {Promise<Object>} URL and state information
   */
  async buildAuthorizeUrl(options = {}) {
    const { codeVerifier, codeChallenge } = await this._generatePKCE();
    const state = this._generateState();
    const nonce = this._generateNonce();

    const params = new URLSearchParams({
      client_id: this.clientId,
      redirect_uri: this.redirectUri,
      response_type: 'code',
      scope: this._getScope(options),
      state,
      nonce,
      code_challenge: codeChallenge,
      code_challenge_method: 'S256',
    });

    if (options.organization || this.organization) {
      params.append('organization', options.organization || this.organization);
    }
    if (options.connection) {
      params.append('connection', options.connection);
    }
    if (this.audience) {
      params.append('audience', this.audience);
    }

    return {
      url: `${this.domain}/authorize?${params}`,
      state,
      nonce,
      codeVerifier,
    };
  }

  // ============================================================================
  // Private Methods - PKCE
  // ============================================================================

  /**
   * Generate PKCE code verifier and challenge
   * @returns {Promise<Object>}
   */
  async _generatePKCE() {
    // Generate 32 random bytes for code verifier
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    const codeVerifier = this._base64UrlEncode(array);

    // Generate code challenge (SHA-256 hash of verifier)
    const encoder = new TextEncoder();
    const data = encoder.encode(codeVerifier);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const codeChallenge = this._base64UrlEncode(new Uint8Array(hashBuffer));

    return { codeVerifier, codeChallenge };
  }

  /**
   * Base64 URL encode
   * @param {Uint8Array} buffer
   * @returns {string}
   */
  _base64UrlEncode(buffer) {
    let binary = '';
    for (let i = 0; i < buffer.length; i++) {
      binary += String.fromCharCode(buffer[i]);
    }
    return btoa(binary)
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=+$/, '');
  }

  // ============================================================================
  // Private Methods - Token Exchange
  // ============================================================================

  /**
   * Exchange authorization code for tokens
   * @param {string} code - Authorization code
   * @param {string} codeVerifier - PKCE code verifier
   * @returns {Promise<Object>}
   */
  async _exchangeCodeForTokens(code, codeVerifier) {
    const response = await fetch(`${this.domain}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'authorization_code',
        code,
        redirect_uri: this.redirectUri,
        client_id: this.clientId,
        code_verifier: codeVerifier,
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new AuthenticationError(
        error.error || 'token_exchange_failed',
        error.error_description || 'Failed to exchange code for tokens'
      );
    }

    return response.json();
  }

  /**
   * Refresh access token
   * @param {string} refreshToken
   * @returns {Promise<Object>}
   */
  async _refreshAccessToken(refreshToken) {
    const response = await fetch(`${this.domain}/oauth/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: this.clientId,
      }),
    });

    if (!response.ok) {
      throw new AuthenticationError('refresh_failed', 'Failed to refresh token');
    }

    return response.json();
  }

  // ============================================================================
  // Private Methods - Token Cache
  // ============================================================================

  /**
   * Cache tokens in memory and storage
   * @param {Object} tokens
   */
  _cacheTokens(tokens) {
    const expiresIn = tokens.expires_in || 3600;
    const expiresAt = Date.now() + expiresIn * 1000;

    this._tokenCache = {
      access_token: tokens.access_token,
      id_token: tokens.id_token,
      expires_at: expiresAt,
      scope: tokens.scope,
    };

    // Store refresh token if present
    if (tokens.refresh_token) {
      this._setStorageItem('refresh_token', tokens.refresh_token);
    }

    // Store access token expiry for page reloads
    this._setStorageItem('token_expires_at', expiresAt.toString());
    this._setStorageItem('access_token', tokens.access_token);
    if (tokens.id_token) {
      this._setStorageItem('id_token', tokens.id_token);
    }
  }

  /**
   * Clear token cache
   */
  _clearTokenCache() {
    this._tokenCache = null;
    this._removeStorageItem('refresh_token');
    this._removeStorageItem('access_token');
    this._removeStorageItem('id_token');
    this._removeStorageItem('token_expires_at');
  }

  /**
   * Load cached tokens on initialization
   */
  _loadCachedTokens() {
    const accessToken = this._getStorageItem('access_token');
    const idToken = this._getStorageItem('id_token');
    const expiresAt = parseInt(this._getStorageItem('token_expires_at') || '0', 10);

    if (accessToken && expiresAt > Date.now()) {
      this._tokenCache = {
        access_token: accessToken,
        id_token: idToken,
        expires_at: expiresAt,
      };
    }
  }

  // ============================================================================
  // Private Methods - Storage
  // ============================================================================

  _setStorageItem(key, value) {
    try {
      localStorage.setItem(`${this._storagePrefix}_${key}`, value);
    } catch {
      // localStorage not available
    }
  }

  _getStorageItem(key) {
    try {
      return localStorage.getItem(`${this._storagePrefix}_${key}`);
    } catch {
      return null;
    }
  }

  _removeStorageItem(key) {
    try {
      localStorage.removeItem(`${this._storagePrefix}_${key}`);
    } catch {
      // localStorage not available
    }
  }

  _clearAuthState() {
    this._removeStorageItem('pkce_code_verifier');
    this._removeStorageItem('auth_state');
    this._removeStorageItem('auth_nonce');
    this._removeStorageItem('app_state');
  }

  // ============================================================================
  // Private Methods - Utilities
  // ============================================================================

  _generateState() {
    const array = new Uint8Array(16);
    crypto.getRandomValues(array);
    return this._base64UrlEncode(array);
  }

  _generateNonce() {
    const array = new Uint8Array(16);
    crypto.getRandomValues(array);
    return this._base64UrlEncode(array);
  }

  _getScope(options) {
    const scopes = new Set((this.scope || '').split(' '));
    if (options.scope) {
      options.scope.split(' ').forEach((s) => scopes.add(s));
    }
    if (this.useRefreshTokens) {
      scopes.add('offline_access');
    }
    return Array.from(scopes).join(' ');
  }

  _parseJWT(token) {
    try {
      const parts = token.split('.');
      if (parts.length !== 3) return null;

      const base64Url = parts[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(
        atob(base64)
          .split('')
          .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
          .join('')
      );
      return JSON.parse(jsonPayload);
    } catch {
      return null;
    }
  }
}

/**
 * Authentication Error
 */
class AuthenticationError extends Error {
  constructor(code, description) {
    super(description || code);
    this.name = 'AuthenticationError';
    this.code = code;
    this.description = description;
  }
}

// ============================================================================
// React Hook (if React is available)
// ============================================================================

/**
 * React hook for CoreAuth
 * @param {CoreAuth} client - CoreAuth client instance
 * @returns {Object} Auth state and methods
 */
function useCoreAuth(client) {
  if (typeof window === 'undefined' || !window.React) {
    throw new Error('useCoreAuth requires React');
  }

  const React = window.React;
  const [state, setState] = React.useState({
    isLoading: true,
    isAuthenticated: false,
    user: null,
    error: null,
  });

  React.useEffect(() => {
    const checkAuth = async () => {
      try {
        // Handle callback if present
        if (window.location.search.includes('code=')) {
          await client.handleRedirectCallback();
        }

        const isAuthenticated = await client.isAuthenticated();
        const user = isAuthenticated ? await client.getUser() : null;

        setState({
          isLoading: false,
          isAuthenticated,
          user,
          error: null,
        });
      } catch (error) {
        setState({
          isLoading: false,
          isAuthenticated: false,
          user: null,
          error,
        });
      }
    };

    checkAuth();
  }, [client]);

  return {
    ...state,
    loginWithRedirect: (opts) => client.loginWithRedirect(opts),
    logout: (opts) => client.logout(opts),
    getAccessToken: (opts) => client.getAccessToken(opts),
    getUser: () => client.getUser(),
  };
}

// ============================================================================
// Exports
// ============================================================================

if (typeof module !== 'undefined' && module.exports) {
  module.exports = { CoreAuth, AuthenticationError, useCoreAuth };
}

if (typeof window !== 'undefined') {
  window.CoreAuth = CoreAuth;
  window.AuthenticationError = AuthenticationError;
  window.useCoreAuth = useCoreAuth;
}

// ES Module export
if (typeof exports !== 'undefined') {
  exports.CoreAuth = CoreAuth;
  exports.AuthenticationError = AuthenticationError;
  exports.useCoreAuth = useCoreAuth;
}
