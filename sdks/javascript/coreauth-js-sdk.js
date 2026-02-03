/**
 * CoreAuth JavaScript SDK
 *
 * Client-side SDK for integrating CoreAuth authentication into web applications
 *
 * @example
 * const auth = new CoreAuth({
 *   domain: 'http://localhost:8000',
 *   clientId: 'your-client-id',
 *   redirectUri: 'http://yourapp.com/callback'
 * });
 */

class CoreAuth {
  constructor(config) {
    this.domain = config.domain;
    this.clientId = config.clientId;
    this.redirectUri = config.redirectUri;
    this.organization = config.organization;
    this.scope = config.scope || 'openid profile email';
    this.storage = window.localStorage;
  }

  /**
   * Redirect to CoreAuth login page
   * @param {Object} options - Login options
   */
  login(options = {}) {
    const params = new URLSearchParams({
      client_id: this.clientId,
      redirect_uri: this.redirectUri,
      response_type: 'code',
      scope: this.scope,
      organization: options.organization || this.organization,
      state: this._generateState(),
    });

    if (options.loginHint) {
      params.append('login_hint', options.loginHint);
    }

    window.location.href = `${this.domain}/api/auth/authorize?${params}`;
  }

  /**
   * Login with email/password (embedded)
   * @param {string} email - User email
   * @param {string} password - User password
   * @param {string} organization - Organization slug
   */
  async loginWithCredentials(email, password, organization) {
    const response = await fetch(`${this.domain}/api/auth/login-hierarchical`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email,
        password,
        organization_slug: organization,
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Login failed');
    }

    const data = await response.json();

    // Handle MFA enrollment required
    if (data.message && data.message.includes('multi-factor')) {
      return {
        requiresMFA: true,
        enrollmentToken: data.enrollment_token,
        canSkip: data.can_skip,
        gracePeriod: data.grace_period_expires,
      };
    }

    // Successful login
    this._setSession(data);
    return {
      success: true,
      user: data.user,
    };
  }

  /**
   * Login with SSO provider
   * @param {string} organization - Organization slug
   * @param {string} connection - Connection name (optional)
   */
  loginWithSSO(organization, connection = null) {
    const params = new URLSearchParams({
      organization,
      redirect_uri: this.redirectUri,
      state: this._generateState(),
    });

    if (connection) {
      params.append('connection', connection);
    }

    window.location.href = `${this.domain}/api/oidc/login?${params}`;
  }

  /**
   * Handle callback after redirect
   * @returns {Promise<Object>} User info and tokens
   */
  async handleCallback() {
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    const state = params.get('state');

    if (!code) {
      throw new Error('No authorization code found');
    }

    // Verify state
    const savedState = this.storage.getItem('auth_state');
    if (state !== savedState) {
      throw new Error('Invalid state parameter');
    }

    // Exchange code for tokens
    const response = await fetch(`${this.domain}/api/auth/token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        code,
        client_id: this.clientId,
        redirect_uri: this.redirectUri,
        grant_type: 'authorization_code',
      }),
    });

    if (!response.ok) {
      throw new Error('Token exchange failed');
    }

    const data = await response.json();
    this._setSession(data);

    // Clean up URL
    window.history.replaceState({}, document.title, window.location.pathname);

    return {
      user: data.user,
      accessToken: data.access_token,
    };
  }

  /**
   * Logout user
   */
  async logout(returnTo = null) {
    const accessToken = this.getAccessToken();

    if (accessToken) {
      await fetch(`${this.domain}/api/auth/logout`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${accessToken}`,
        },
      });
    }

    this._clearSession();

    if (returnTo) {
      window.location.href = returnTo;
    }
  }

  /**
   * Get current user
   * @returns {Promise<Object>} User object
   */
  async getUser() {
    const accessToken = this.getAccessToken();

    if (!accessToken) {
      return null;
    }

    const response = await fetch(`${this.domain}/api/auth/me`, {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
      },
    });

    if (!response.ok) {
      if (response.status === 401) {
        // Try to refresh token
        return this._refreshTokenAndRetry(() => this.getUser());
      }
      return null;
    }

    return response.json();
  }

  /**
   * Check if user is authenticated
   * @returns {boolean}
   */
  isAuthenticated() {
    const token = this.getAccessToken();
    if (!token) return false;

    // Check if token is expired
    const payload = this._parseJWT(token);
    if (!payload) return false;

    return payload.exp * 1000 > Date.now();
  }

  /**
   * Get access token
   * @returns {string|null}
   */
  getAccessToken() {
    return this.storage.getItem('access_token');
  }

  /**
   * Get refresh token
   * @returns {string|null}
   */
  getRefreshToken() {
    return this.storage.getItem('refresh_token');
  }

  /**
   * Refresh access token
   * @returns {Promise<Object>}
   */
  async refreshToken() {
    const refreshToken = this.getRefreshToken();

    if (!refreshToken) {
      throw new Error('No refresh token available');
    }

    const response = await fetch(`${this.domain}/api/auth/refresh`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        refresh_token: refreshToken,
      }),
    });

    if (!response.ok) {
      this._clearSession();
      throw new Error('Token refresh failed');
    }

    const data = await response.json();
    this._setSession(data);

    return data;
  }

  /**
   * Enroll MFA (TOTP)
   * @param {string} enrollmentToken - Enrollment token from login
   * @returns {Promise<Object>} QR code and secret
   */
  async enrollMFA(enrollmentToken) {
    const response = await fetch(`${this.domain}/api/mfa/enroll-with-token/totp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        enrollment_token: enrollmentToken,
      }),
    });

    if (!response.ok) {
      throw new Error('MFA enrollment failed');
    }

    return response.json();
  }

  /**
   * Verify MFA code
   * @param {string} enrollmentToken - Enrollment token
   * @param {string} methodId - MFA method ID
   * @param {string} code - TOTP code
   * @returns {Promise<Object>}
   */
  async verifyMFA(enrollmentToken, methodId, code) {
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
      throw new Error('MFA verification failed');
    }

    const data = await response.json();
    this._setSession(data);

    return {
      success: true,
      user: data.user,
    };
  }

  /**
   * Create authenticated fetch wrapper
   * @returns {Function}
   */
  createAuthenticatedFetch() {
    return async (url, options = {}) => {
      const accessToken = this.getAccessToken();

      const headers = {
        ...options.headers,
        'Authorization': `Bearer ${accessToken}`,
      };

      const response = await fetch(url, {
        ...options,
        headers,
      });

      // Handle token expiration
      if (response.status === 401) {
        await this.refreshToken();
        // Retry with new token
        const newToken = this.getAccessToken();
        headers['Authorization'] = `Bearer ${newToken}`;
        return fetch(url, { ...options, headers });
      }

      return response;
    };
  }

  // Private methods

  _setSession(data) {
    this.storage.setItem('access_token', data.access_token);
    this.storage.setItem('refresh_token', data.refresh_token);
    if (data.user) {
      this.storage.setItem('user', JSON.stringify(data.user));
    }
  }

  _clearSession() {
    this.storage.removeItem('access_token');
    this.storage.removeItem('refresh_token');
    this.storage.removeItem('user');
    this.storage.removeItem('auth_state');
  }

  _generateState() {
    const state = Math.random().toString(36).substring(7);
    this.storage.setItem('auth_state', state);
    return state;
  }

  _parseJWT(token) {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(
        atob(base64)
          .split('')
          .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
          .join('')
      );
      return JSON.parse(jsonPayload);
    } catch (e) {
      return null;
    }
  }

  async _refreshTokenAndRetry(fn) {
    try {
      await this.refreshToken();
      return fn();
    } catch (e) {
      this._clearSession();
      throw e;
    }
  }
}

// Export for different module systems
if (typeof module !== 'undefined' && module.exports) {
  module.exports = CoreAuth;
}
if (typeof window !== 'undefined') {
  window.CoreAuth = CoreAuth;
}
