import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import api from '../lib/api';

export default function Signup() {
  const navigate = useNavigate();
  const [formData, setFormData] = useState({
    organizationName: '',
    organizationSlug: '',
    adminEmail: '',
    adminPassword: '',
    adminFullName: '',
  });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }));

    // Auto-generate slug from organization name
    if (name === 'organizationName') {
      const slug = value
        .toLowerCase()
        .replace(/[^a-z0-9-]/g, '-')
        .replace(/-+/g, '-')
        .replace(/^-|-$/g, '');
      setFormData((prev) => ({
        ...prev,
        organizationSlug: slug,
      }));
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const response = await api.post('/tenants', {
        name: formData.organizationName,
        slug: formData.organizationSlug,
        admin_email: formData.adminEmail,
        admin_password: formData.adminPassword,
        admin_full_name: formData.adminFullName,
      });

      // Auto-login after signup
      const loginResponse = await api.post('/auth/login-hierarchical', {
        email: formData.adminEmail,
        password: formData.adminPassword,
        organization_slug: formData.organizationSlug,
      });

      localStorage.setItem('access_token', loginResponse.data.access_token);
      localStorage.setItem('refresh_token', loginResponse.data.refresh_token);
      navigate('/dashboard');
    } catch (err) {
      console.error('Signup error:', err);
      setError(
        err.response?.data?.message || 'Failed to create tenant. Please try again.'
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center p-6">
      <div className="w-full max-w-2xl">
        {/* Logo */}
        <Link to="/" className="flex items-center justify-center mb-8">
          <span className="text-3xl">
            <span className="font-bold text-slate-900">core.</span>
            <span className="font-normal text-slate-500">auth</span>
          </span>
        </Link>

        {/* Card */}
        <div className="bg-white rounded-2xl shadow-xl border border-slate-200 p-8">
          <div className="text-center mb-8">
            <h1 className="text-3xl font-bold mb-2">Create Your Tenant Account</h1>
            <p className="text-slate-600">Start managing authentication for your customers</p>
          </div>

          {error && (
            <div className="mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg flex items-start space-x-3">
              <svg className="w-5 h-5 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
              <span className="text-sm">{error}</span>
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Organization Details */}
            <div className="space-y-4">
              <div className="flex items-center space-x-2 text-sm font-semibold text-slate-700 mb-4">
                <div className="w-6 h-6 bg-primary-600 text-white rounded-full flex items-center justify-center text-xs">
                  1
                </div>
                <span>Tenant Details</span>
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Company Name
                </label>
                <input
                  type="text"
                  name="organizationName"
                  required
                  className="input-field"
                  placeholder="Acme Corporation"
                  value={formData.organizationName}
                  onChange={handleChange}
                />
                <p className="text-xs text-slate-500 mt-1">
                  Your company that will use CoreAuth to manage customer authentication
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Tenant Slug
                  <span className="text-slate-500 font-normal ml-2">
                    (used in URLs and API calls)
                  </span>
                </label>
                <div className="flex items-center space-x-2">
                  <span className="text-slate-500 text-sm">https://api.coreauth.dev/</span>
                  <input
                    type="text"
                    name="organizationSlug"
                    required
                    pattern="[a-z0-9-]+"
                    className="input-field"
                    placeholder="acme"
                    value={formData.organizationSlug}
                    onChange={handleChange}
                  />
                </div>
                <p className="text-xs text-slate-500 mt-1">
                  Only lowercase letters, numbers, and hyphens
                </p>
              </div>
            </div>

            {/* Admin Account */}
            <div className="space-y-4 pt-6 border-t border-slate-200">
              <div className="flex items-center space-x-2 text-sm font-semibold text-slate-700 mb-4">
                <div className="w-6 h-6 bg-primary-600 text-white rounded-full flex items-center justify-center text-xs">
                  2
                </div>
                <span>Admin Account</span>
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Full Name
                </label>
                <input
                  type="text"
                  name="adminFullName"
                  required
                  className="input-field"
                  placeholder="John Doe"
                  value={formData.adminFullName}
                  onChange={handleChange}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Email Address
                </label>
                <input
                  type="email"
                  name="adminEmail"
                  required
                  className="input-field"
                  placeholder="john@acme.com"
                  value={formData.adminEmail}
                  onChange={handleChange}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Password
                </label>
                <input
                  type="password"
                  name="adminPassword"
                  required
                  minLength={8}
                  className="input-field"
                  placeholder="••••••••"
                  value={formData.adminPassword}
                  onChange={handleChange}
                />
                <p className="text-xs text-slate-500 mt-1">
                  At least 8 characters with uppercase, lowercase, and number
                </p>
              </div>
            </div>

            <button
              type="submit"
              disabled={loading}
              className="w-full btn-primary py-3 text-lg disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? (
                <span className="flex items-center justify-center space-x-2">
                  <svg className="animate-spin h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                  <span>Creating Tenant...</span>
                </span>
              ) : (
                'Create Tenant Account'
              )}
            </button>
          </form>

          <div className="mt-6 text-center text-sm text-slate-600">
            Already have an account?{' '}
            <Link to="/login" className="text-primary-600 hover:text-primary-700 font-medium">
              Sign in
            </Link>
          </div>
        </div>

        <p className="text-center text-sm text-slate-500 mt-6">
          By signing up, you agree to our Terms of Service and Privacy Policy
        </p>
      </div>
    </div>
  );
}
