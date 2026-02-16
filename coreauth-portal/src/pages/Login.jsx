import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import api from '../lib/api';

export default function Login() {
  const navigate = useNavigate();
  const [formData, setFormData] = useState({
    email: '',
    password: '',
    organizationSlug: '',
  });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }));
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const response = await api.post('/auth/login-hierarchical', {
        email: formData.email,
        password: formData.password,
        organization_slug: formData.organizationSlug || undefined,
      });

      // Check response type
      const data = response.data;

      if (data.access_token) {
        // Successful login
        localStorage.setItem('access_token', data.access_token);
        localStorage.setItem('refresh_token', data.refresh_token);
        localStorage.setItem('user', JSON.stringify(data.user));
        // Dispatch custom event to notify App component of auth change
        window.dispatchEvent(new Event('auth-change'));
        navigate('/dashboard');
      } else if (data.message && data.message.includes('multi-factor authentication')) {
        // MFA enrollment required
        localStorage.setItem('mfa_enrollment_data', JSON.stringify({
          enrollment_token: data.enrollment_token,
          email: formData.email,
          organizationSlug: formData.organizationSlug,
          can_skip: data.can_skip,
          grace_period_expires: data.grace_period_expires,
          message: data.message,
        }));
        navigate('/mfa-setup');
      } else {
        setError('Unexpected response from server');
      }
    } catch (err) {
      setError(
        err.response?.data?.message || 'Invalid credentials. Please try again.'
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center p-6" style={{ backgroundColor: '#F7F7F7' }}>
      <div className="w-full max-w-md">
        {/* Logo */}
        <Link to="/" className="flex items-center justify-center mb-12">
          <img
            src="/core-auth-logo.svg"
            alt="CoreAuth"
            className="w-48 h-auto"
            style={{ minWidth: '140px' }}
          />
        </Link>

        {/* Card */}
        <div className="bg-white rounded-2xl shadow-lg p-8" style={{ border: '1px solid #EEEEEE' }}>
          <div className="text-center mb-8">
            <h1 className="text-3xl font-bold mb-2" style={{ color: '#111111', letterSpacing: '-1px' }}>Welcome Back</h1>
            <p style={{ color: '#777777' }}>Sign in to your tenant account</p>
          </div>

          {error && (
            <div className="mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg flex items-start space-x-3">
              <svg className="w-5 h-5 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
              </svg>
              <span className="text-sm">{error}</span>
            </div>
          )}

          <form onSubmit={handleSubmit} className="space-y-5">
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                Account Name
                <span className="text-slate-500 font-normal ml-2">(optional)</span>
              </label>
              <input
                type="text"
                name="organizationSlug"
                className="input-field"
                placeholder="my-company"
                value={formData.organizationSlug}
                onChange={handleChange}
              />
              <p className="text-xs text-slate-500 mt-1">
                Your organization's account name.{' '}
                <Link to={`/login/${formData.organizationSlug || 'your-org'}`} className="text-primary-600 hover:text-primary-700">
                  Use SSO login instead
                </Link>
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                Email Address
              </label>
              <input
                type="email"
                name="email"
                required
                className="input-field"
                placeholder="john@acme.com"
                value={formData.email}
                onChange={handleChange}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                Password
              </label>
              <input
                type="password"
                name="password"
                required
                className="input-field"
                placeholder="••••••••"
                value={formData.password}
                onChange={handleChange}
              />
            </div>

            <div className="flex items-center justify-between text-sm">
              <label className="flex items-center space-x-2">
                <input type="checkbox" className="w-4 h-4 text-primary-600 border-slate-300 rounded focus:ring-primary-500" />
                <span className="text-slate-600">Remember me</span>
              </label>
              <a href="#" className="text-primary-600 hover:text-primary-700 font-medium">
                Forgot password?
              </a>
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
                  <span>Signing in...</span>
                </span>
              ) : (
                'Sign In'
              )}
            </button>
          </form>

          <div className="mt-6 text-center text-sm text-slate-600">
            Don't have an account?{' '}
            <Link to="/signup" className="text-primary-600 hover:text-primary-700 font-medium">
              Create one for free
            </Link>
          </div>
        </div>
      </div>
    </div>
  );
}
