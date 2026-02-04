import { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import api from '../lib/api';

export default function SSOCallback() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [error, setError] = useState('');
  const [processing, setProcessing] = useState(true);

  useEffect(() => {
    const handleCallback = async () => {
      const code = searchParams.get('code');
      const state = searchParams.get('state');
      const errorParam = searchParams.get('error');
      const errorDescription = searchParams.get('error_description');

      // Handle error from IdP
      if (errorParam) {
        setError(errorDescription || `Authentication failed: ${errorParam}`);
        setProcessing(false);
        return;
      }

      if (!code || !state) {
        setError('Invalid callback - missing code or state');
        setProcessing(false);
        return;
      }

      try {
        // Exchange code for tokens
        const response = await api.get('/oidc/callback', {
          params: { code, state },
        });

        const data = response.data;

        if (data.access_token) {
          // Successful login
          localStorage.setItem('access_token', data.access_token);
          localStorage.setItem('refresh_token', data.refresh_token);
          localStorage.setItem('user', JSON.stringify(data.user));
          navigate('/dashboard');
        } else {
          setError('Unexpected response from server');
          setProcessing(false);
        }
      } catch (err) {
        console.error('SSO callback error:', err);
        setError(err.response?.data?.message || 'SSO authentication failed');
        setProcessing(false);
      }
    };

    handleCallback();
  }, [searchParams, navigate]);

  if (processing) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto mb-4"></div>
          <p className="text-slate-600">Completing sign in...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center p-6">
      <div className="w-full max-w-md">
        <div className="bg-white rounded-2xl shadow-xl border border-slate-200 p-8 text-center">
          <div className="text-6xl mb-4">⚠️</div>
          <h1 className="text-2xl font-bold text-slate-900 mb-2">Sign In Failed</h1>
          <p className="text-slate-600 mb-6">{error}</p>
          <button
            onClick={() => navigate(-1)}
            className="inline-block px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
          >
            Try Again
          </button>
        </div>
      </div>
    </div>
  );
}
