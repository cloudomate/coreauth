import { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import axios from 'axios';
import api from '../lib/api';

// Create a separate axios instance for public endpoints (no auth interceptor)
const publicApi = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
});

export default function OrgLogin() {
  const { orgSlug } = useParams();
  const navigate = useNavigate();
  const [organization, setOrganization] = useState(null);
  const [connections, setConnections] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [formData, setFormData] = useState({
    email: '',
    password: '',
  });
  const [submitting, setSubmitting] = useState(false);
  const [showPasswordForm, setShowPasswordForm] = useState(false);

  useEffect(() => {
    const loadOrgData = async () => {
      try {
        // Fetch organization by slug (public endpoint)
        const orgRes = await publicApi.get(`/organizations/by-slug/${orgSlug}`);
        setOrganization(orgRes.data);

        // Fetch SSO connections for this organization (public endpoint)
        const connRes = await publicApi.get(`/oidc/providers/public?tenant_id=${orgRes.data.id}`);
        setConnections(connRes.data);
      } catch (err) {
        if (err.response?.status === 404) {
          setError('Organization not found');
        } else {
          setError('Failed to load organization');
        }
      } finally {
        setLoading(false);
      }
    };

    loadOrgData();
  }, [orgSlug]);

  const handleChange = (e) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value,
    });
  };

  const handlePasswordLogin = async (e) => {
    e.preventDefault();
    setError('');
    setSubmitting(true);

    try {
      const response = await api.post('/auth/login-hierarchical', {
        email: formData.email,
        password: formData.password,
        organization_slug: orgSlug,
      });

      const data = response.data;

      if (data.access_token) {
        localStorage.setItem('access_token', data.access_token);
        localStorage.setItem('refresh_token', data.refresh_token);
        localStorage.setItem('user', JSON.stringify(data.user));
        // Dispatch custom event to notify App component of auth change
        window.dispatchEvent(new Event('auth-change'));
        navigate('/dashboard');
      } else if (data.message && data.message.includes('multi-factor authentication')) {
        localStorage.setItem('mfa_enrollment_data', JSON.stringify({
          enrollment_token: data.enrollment_token,
          email: formData.email,
          organizationSlug: orgSlug,
          can_skip: data.can_skip,
          grace_period_expires: data.grace_period_expires,
          message: data.message,
        }));
        navigate('/mfa-setup');
      } else {
        setError('Unexpected response from server');
      }
    } catch (err) {
      setError(err.response?.data?.message || 'Invalid credentials');
    } finally {
      setSubmitting(false);
    }
  };

  const handleSSOLogin = async (connection) => {
    try {
      // Get the callback URL for this app
      const callbackUrl = `${window.location.origin}/sso/callback`;

      // Initiate OIDC login flow (public endpoint)
      const response = await publicApi.get('/oidc/login', {
        params: {
          tenant_id: organization.id,
          provider_id: connection.id,
          redirect_uri: callbackUrl,
        },
      });

      // Redirect to the IdP authorization URL
      window.location.href = response.data.authorization_url;
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to initiate SSO login');
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center">
        <div className="text-slate-600">Loading...</div>
      </div>
    );
  }

  if (error && !organization) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center p-6">
        <div className="w-full max-w-md">
          <div className="bg-white rounded-2xl shadow-xl border border-slate-200 p-8 text-center">
            <div className="text-6xl mb-4">🔒</div>
            <h1 className="text-2xl font-bold text-slate-900 mb-2">Organization Not Found</h1>
            <p className="text-slate-600 mb-6">
              The organization "{orgSlug}" doesn't exist or is not configured for login.
            </p>
            <Link
              to="/login"
              className="inline-block px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
            >
              Go to Main Login
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center p-6">
      <div className="w-full max-w-md">
        {/* Organization Branding */}
        <div className="text-center mb-8">
          <div className="w-16 h-16 bg-primary-600 rounded-xl flex items-center justify-center mx-auto mb-4">
            <span className="text-2xl font-bold text-white">
              {organization?.name?.charAt(0).toUpperCase()}
            </span>
          </div>
          <h1 className="text-2xl font-bold text-slate-900">{organization?.name}</h1>
          <p className="text-slate-500 text-sm">Sign in to continue</p>
        </div>

        {/* Login Card */}
        <div className="bg-white rounded-2xl shadow-xl border border-slate-200 p-8">
          {error && (
            <div className="mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg text-sm">
              {error}
            </div>
          )}

          {/* SSO Buttons */}
          {connections.length > 0 && (
            <div className="space-y-3 mb-6">
              {connections.map((conn) => (
                <button
                  key={conn.id}
                  onClick={() => handleSSOLogin(conn)}
                  className="w-full flex items-center justify-center space-x-3 px-4 py-3 border-2 border-slate-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition-all"
                >
                  <span className="text-xl">{getProviderIcon(conn.provider_type)}</span>
                  <span className="font-medium text-slate-700">
                    Continue with {getProviderDisplayName(conn.provider_type)}
                  </span>
                </button>
              ))}
            </div>
          )}

          {/* Divider */}
          {connections.length > 0 && (
            <div className="relative my-6">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-slate-200"></div>
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-4 bg-white text-slate-500">or sign in with email</span>
              </div>
            </div>
          )}

          {/* Password Login Toggle/Form */}
          {!showPasswordForm && connections.length > 0 ? (
            <button
              onClick={() => setShowPasswordForm(true)}
              className="w-full py-3 text-slate-600 hover:text-slate-900 text-sm font-medium"
            >
              Sign in with password instead
            </button>
          ) : (
            <form onSubmit={handlePasswordLogin} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">
                  Email
                </label>
                <input
                  type="email"
                  name="email"
                  required
                  value={formData.email}
                  onChange={handleChange}
                  className="input-field"
                  placeholder="you@company.com"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">
                  Password
                </label>
                <input
                  type="password"
                  name="password"
                  required
                  value={formData.password}
                  onChange={handleChange}
                  className="input-field"
                  placeholder="••••••••"
                />
              </div>

              <button
                type="submit"
                disabled={submitting}
                className="w-full btn-primary py-3 disabled:opacity-50"
              >
                {submitting ? 'Signing in...' : 'Sign In'}
              </button>

              {connections.length > 0 && (
                <button
                  type="button"
                  onClick={() => setShowPasswordForm(false)}
                  className="w-full py-2 text-slate-500 hover:text-slate-700 text-sm"
                >
                  Back to SSO options
                </button>
              )}
            </form>
          )}
        </div>

        {/* Footer */}
        <div className="mt-6 text-center text-sm text-slate-500">
          <p>
            Not a member of {organization?.name}?{' '}
            <Link to="/login" className="text-primary-600 hover:text-primary-700 font-medium">
              Use a different organization
            </Link>
          </p>
        </div>

        {/* Powered by */}
        <div className="mt-8 text-center">
          <span className="text-xs text-slate-400">Powered by </span>
          <Link to="/" className="text-xs text-slate-500 hover:text-slate-700">
            <span className="font-semibold">core.</span>
            <span>auth</span>
          </Link>
        </div>
      </div>
    </div>
  );
}

function getProviderIcon(providerType) {
  const icons = {
    google: '🔵',
    azuread: '🔷',
    azure_ad: '🔷',
    microsoft: '🔷',
    okta: '🔶',
    auth0: '🟠',
    generic: '🔗',
  };
  return icons[providerType] || '🔗';
}

function getProviderDisplayName(providerType) {
  const names = {
    google: 'Google',
    azuread: 'Microsoft',
    azure_ad: 'Microsoft',
    microsoft: 'Microsoft',
    okta: 'Okta',
    auth0: 'Auth0',
    generic: 'SSO',
  };
  return names[providerType] || 'SSO';
}
