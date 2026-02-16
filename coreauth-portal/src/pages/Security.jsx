import { useState, useEffect } from 'react';
import api from '../lib/api';

export default function Security() {
  const [settings, setSettings] = useState(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [orgId, setOrgId] = useState(null);

  useEffect(() => {
    loadSecuritySettings();
  }, []);

  const loadSecuritySettings = async () => {
    try {
      setLoading(true);
      setError('');

      // Get current user to find tenant/org ID
      const meResponse = await api.get('/auth/me');
      const tenantId = meResponse.data.default_tenant_id;
      setOrgId(tenantId);

      // Fetch security settings
      const response = await api.get(`/organizations/${tenantId}/security`);
      setSettings(response.data);
    } catch (err) {
      setError('Failed to load security settings');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateSettings = async (updates) => {
    try {
      setSaving(true);
      setError('');
      setSuccess('');

      await api.put(`/organizations/${orgId}/security`, updates);

      setSuccess('Security settings updated successfully');
      await loadSecuritySettings();

      // Clear success message after 3 seconds
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to update security settings');
    } finally {
      setSaving(false);
    }
  };

  const handleToggleMFA = async () => {
    const newValue = !settings.mfa_required;
    if (newValue) {
      if (!confirm('Enabling MFA will require all users to set up multi-factor authentication. Continue?')) {
        return;
      }
    }
    await handleUpdateSettings({ mfa_required: newValue });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-600">Loading security settings...</div>
      </div>
    );
  }

  if (!settings) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-red-600">Failed to load security settings</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-slate-900">Security Settings</h1>
        <p className="text-slate-600 mt-1">Manage authentication and security policies for your organization</p>
      </div>

      {/* Messages */}
      {error && (
        <div className="p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
          {error}
        </div>
      )}
      {success && (
        <div className="p-4 bg-green-50 border border-green-200 text-green-700 rounded-lg">
          {success}
        </div>
      )}

      {/* MFA Settings */}
      <div className="card">
        <div className="flex items-start justify-between mb-6">
          <div>
            <h2 className="text-lg font-semibold text-slate-900 mb-1">Multi-Factor Authentication (MFA)</h2>
            <p className="text-sm text-slate-600">Require users to use MFA to enhance account security</p>
          </div>
          <button
            onClick={handleToggleMFA}
            disabled={saving}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
              settings.mfa_required ? 'bg-primary-600' : 'bg-slate-300'
            } ${saving ? 'opacity-50 cursor-not-allowed' : ''}`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                settings.mfa_required ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {settings.mfa_required && (
          <div className="space-y-4 pt-4 border-t border-slate-200">
            {settings.mfa_enforcement_date && (
              <div className="p-3 bg-primary-50 border border-primary-200 rounded-lg">
                <p className="text-sm text-primary-900">
                  <strong>MFA Enforced:</strong> {new Date(settings.mfa_enforcement_date).toLocaleDateString()}
                </p>
              </div>
            )}

            {/* Grace Period */}
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                Grace Period (days)
              </label>
              <input
                type="number"
                min="0"
                max="90"
                value={settings.mfa_grace_period_days}
                onChange={(e) => handleUpdateSettings({ mfa_grace_period_days: parseInt(e.target.value) })}
                disabled={saving}
                className="input-field w-32"
              />
              <p className="text-xs text-slate-500 mt-1">
                Days users have to set up MFA after enforcement
              </p>
            </div>

            {/* Allowed MFA Methods */}
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-2">
                Allowed MFA Methods
              </label>
              <div className="space-y-2">
                <label className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    checked={settings.allowed_mfa_methods.includes('totp')}
                    onChange={(e) => {
                      const methods = e.target.checked
                        ? [...settings.allowed_mfa_methods, 'totp']
                        : settings.allowed_mfa_methods.filter(m => m !== 'totp');
                      handleUpdateSettings({ allowed_mfa_methods: methods });
                    }}
                    disabled={saving}
                    className="rounded border-slate-300 text-primary-600 focus:ring-primary-500"
                  />
                  <span className="text-sm text-slate-700">TOTP (Authenticator App)</span>
                </label>
                <label className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    checked={settings.allowed_mfa_methods.includes('sms')}
                    onChange={(e) => {
                      const methods = e.target.checked
                        ? [...settings.allowed_mfa_methods, 'sms']
                        : settings.allowed_mfa_methods.filter(m => m !== 'sms');
                      handleUpdateSettings({ allowed_mfa_methods: methods });
                    }}
                    disabled={saving}
                    className="rounded border-slate-300 text-primary-600 focus:ring-primary-500"
                  />
                  <span className="text-sm text-slate-700">SMS</span>
                </label>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Password Policy */}
      <div className="card">
        <h2 className="text-lg font-semibold text-slate-900 mb-4">Password Policy</h2>

        <div className="space-y-4">
          {/* Minimum Length */}
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Minimum Password Length
            </label>
            <input
              type="number"
              min="8"
              max="128"
              value={settings.password_min_length}
              onChange={(e) => handleUpdateSettings({ password_min_length: parseInt(e.target.value) })}
              disabled={saving}
              className="input-field w-32"
            />
          </div>

          {/* Complexity Requirements */}
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Complexity Requirements
            </label>
            <div className="space-y-2">
              <label className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  checked={settings.password_require_uppercase}
                  onChange={(e) => handleUpdateSettings({ password_require_uppercase: e.target.checked })}
                  disabled={saving}
                  className="rounded border-slate-300 text-primary-600 focus:ring-primary-500"
                />
                <span className="text-sm text-slate-700">Require uppercase letters (A-Z)</span>
              </label>
              <label className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  checked={settings.password_require_lowercase}
                  onChange={(e) => handleUpdateSettings({ password_require_lowercase: e.target.checked })}
                  disabled={saving}
                  className="rounded border-slate-300 text-primary-600 focus:ring-primary-500"
                />
                <span className="text-sm text-slate-700">Require lowercase letters (a-z)</span>
              </label>
              <label className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  checked={settings.password_require_number}
                  onChange={(e) => handleUpdateSettings({ password_require_number: e.target.checked })}
                  disabled={saving}
                  className="rounded border-slate-300 text-primary-600 focus:ring-primary-500"
                />
                <span className="text-sm text-slate-700">Require numbers (0-9)</span>
              </label>
              <label className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  checked={settings.password_require_special}
                  onChange={(e) => handleUpdateSettings({ password_require_special: e.target.checked })}
                  disabled={saving}
                  className="rounded border-slate-300 text-primary-600 focus:ring-primary-500"
                />
                <span className="text-sm text-slate-700">Require special characters (!@#$%...)</span>
              </label>
            </div>
          </div>
        </div>
      </div>

      {/* Session & Account Security */}
      <div className="card">
        <h2 className="text-lg font-semibold text-slate-900 mb-4">Session & Account Security</h2>

        <div className="space-y-4">
          {/* Session Timeout */}
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Session Timeout (hours)
            </label>
            <input
              type="number"
              min="1"
              max="720"
              value={settings.session_timeout_hours}
              onChange={(e) => handleUpdateSettings({ session_timeout_hours: parseInt(e.target.value) })}
              disabled={saving}
              className="input-field w-32"
            />
            <p className="text-xs text-slate-500 mt-1">
              Auto-logout users after this period of inactivity
            </p>
          </div>

          {/* Login Attempts */}
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Max Failed Login Attempts
            </label>
            <input
              type="number"
              min="3"
              max="20"
              value={settings.max_login_attempts}
              onChange={(e) => handleUpdateSettings({ max_login_attempts: parseInt(e.target.value) })}
              disabled={saving}
              className="input-field w-32"
            />
            <p className="text-xs text-slate-500 mt-1">
              Lock account after this many failed attempts
            </p>
          </div>

          {/* Lockout Duration */}
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-2">
              Account Lockout Duration (minutes)
            </label>
            <input
              type="number"
              min="5"
              max="1440"
              value={settings.lockout_duration_minutes}
              onChange={(e) => handleUpdateSettings({ lockout_duration_minutes: parseInt(e.target.value) })}
              disabled={saving}
              className="input-field w-32"
            />
            <p className="text-xs text-slate-500 mt-1">
              How long to lock account after max failed attempts
            </p>
          </div>
        </div>
      </div>

      {/* Current Settings Summary */}
      <div className="card bg-slate-50 border border-slate-200">
        <h3 className="text-sm font-semibold text-slate-900 mb-3">Current Security Configuration</h3>
        <div className="grid md:grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-slate-600">MFA Required:</span>
            <span className={`ml-2 font-medium ${settings.mfa_required ? 'text-green-600' : 'text-slate-700'}`}>
              {settings.mfa_required ? 'Yes' : 'No'}
            </span>
          </div>
          <div>
            <span className="text-slate-600">Password Min Length:</span>
            <span className="ml-2 font-medium text-slate-900">{settings.password_min_length} characters</span>
          </div>
          <div>
            <span className="text-slate-600">Session Timeout:</span>
            <span className="ml-2 font-medium text-slate-900">{settings.session_timeout_hours} hours</span>
          </div>
          <div>
            <span className="text-slate-600">Max Login Attempts:</span>
            <span className="ml-2 font-medium text-slate-900">{settings.max_login_attempts} attempts</span>
          </div>
        </div>
      </div>
    </div>
  );
}
