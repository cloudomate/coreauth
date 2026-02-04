import { useState, useEffect } from 'react';
import api from '../lib/api';

export default function SCIM() {
  const [tokens, setTokens] = useState([]);
  const [logs, setLogs] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showTokenModal, setShowTokenModal] = useState(false);
  const [newToken, setNewToken] = useState(null);
  const [tokenName, setTokenName] = useState('');
  const [activeTab, setActiveTab] = useState('tokens');

  const user = JSON.parse(localStorage.getItem('user') || '{}');
  const tenantId = user.default_organization_id;

  // Generate SCIM endpoint URL
  const scimEndpoint = `${window.location.origin.replace(':3000', ':8000')}/api/scim/v2`;

  useEffect(() => {
    if (tenantId) {
      fetchTokens();
      fetchLogs();
    } else {
      setLoading(false);
    }
  }, [tenantId]);

  const fetchTokens = async () => {
    try {
      const response = await api.get(`/scim/tokens?tenant_id=${tenantId}`);
      setTokens(response.data || []);
    } catch (err) {
      // Silently handle - tokens might not exist yet
      setTokens([]);
    } finally {
      setLoading(false);
    }
  };

  const fetchLogs = async () => {
    try {
      const response = await api.get(`/scim/logs?tenant_id=${tenantId}&limit=50`);
      setLogs(response.data || []);
    } catch (err) {
      // Silently handle
      setLogs([]);
    }
  };

  const handleCreateToken = async (e) => {
    e.preventDefault();
    setError('');

    if (!tokenName.trim()) {
      setError('Please enter a token name');
      return;
    }

    try {
      const response = await api.post('/scim/tokens', {
        tenant_id: tenantId,
        name: tokenName,
      });
      setNewToken(response.data.token);
      setShowCreateModal(false);
      setShowTokenModal(true);
      setTokenName('');
      fetchTokens();
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to create token');
    }
  };

  const handleRevokeToken = async (tokenId) => {
    if (!confirm('Are you sure you want to revoke this token? Any integrations using it will stop working.')) {
      return;
    }

    try {
      await api.delete(`/scim/tokens/${tokenId}`);
      setSuccess('Token revoked successfully');
      fetchTokens();
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to revoke token');
    }
  };

  const copyToClipboard = (text) => {
    navigator.clipboard.writeText(text);
    setSuccess('Copied to clipboard!');
    setTimeout(() => setSuccess(''), 2000);
  };

  const getActionBadge = (action) => {
    const badges = {
      CREATE: 'bg-green-100 text-green-700',
      UPDATE: 'bg-blue-100 text-blue-700',
      DELETE: 'bg-red-100 text-red-700',
      GET: 'bg-slate-100 text-slate-700',
      LIST: 'bg-slate-100 text-slate-700',
    };
    return badges[action] || 'bg-slate-100 text-slate-700';
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-600">Loading SCIM settings...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-slate-900">SCIM Provisioning</h1>
        <p className="text-slate-600 mt-1">
          Automatically sync users and groups from your identity provider
        </p>
      </div>

      {/* Alerts */}
      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded flex items-center justify-between">
          <span>{error}</span>
          <button onClick={() => setError('')} className="text-red-500 hover:text-red-700">
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}

      {success && (
        <div className="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded flex items-center justify-between">
          <span>{success}</span>
          <button onClick={() => setSuccess('')} className="text-green-500 hover:text-green-700">
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}

      {/* SCIM Endpoint Info */}
      <div className="bg-white rounded-lg shadow border border-slate-200 p-6">
        <h2 className="text-lg font-semibold text-slate-900 mb-4">SCIM 2.0 Endpoint</h2>
        <div className="flex items-center space-x-3">
          <code className="flex-1 bg-slate-100 px-4 py-3 rounded-lg text-sm font-mono text-slate-800 overflow-x-auto">
            {scimEndpoint}
          </code>
          <button
            onClick={() => copyToClipboard(scimEndpoint)}
            className="btn-secondary px-4 py-3"
          >
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
        </div>
        <p className="text-sm text-slate-500 mt-2">
          Use this URL when configuring SCIM in your identity provider (Okta, Azure AD, OneLogin, etc.)
        </p>
      </div>

      {/* Tabs */}
      <div className="border-b border-slate-200">
        <nav className="flex space-x-8">
          <button
            onClick={() => setActiveTab('tokens')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'tokens'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
            }`}
          >
            Access Tokens
          </button>
          <button
            onClick={() => setActiveTab('logs')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'logs'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
            }`}
          >
            Provisioning Logs
          </button>
          <button
            onClick={() => setActiveTab('guides')}
            className={`py-4 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'guides'
                ? 'border-primary-500 text-primary-600'
                : 'border-transparent text-slate-500 hover:text-slate-700 hover:border-slate-300'
            }`}
          >
            Setup Guides
          </button>
        </nav>
      </div>

      {/* Tokens Tab */}
      {activeTab === 'tokens' && (
        <div className="bg-white rounded-lg shadow border border-slate-200">
          <div className="px-6 py-4 border-b border-slate-200 flex items-center justify-between">
            <h3 className="text-lg font-semibold text-slate-900">SCIM Access Tokens</h3>
            <button onClick={() => setShowCreateModal(true)} className="btn-primary">
              <svg className="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              Generate Token
            </button>
          </div>

          {tokens.length === 0 ? (
            <div className="p-12 text-center">
              <div className="w-16 h-16 bg-slate-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg className="w-8 h-8 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-slate-900 mb-2">No SCIM tokens</h3>
              <p className="text-slate-600 mb-6">Generate a token to enable SCIM provisioning from your identity provider.</p>
              <button onClick={() => setShowCreateModal(true)} className="btn-primary">
                Generate Your First Token
              </button>
            </div>
          ) : (
            <div className="divide-y divide-slate-200">
              {tokens.map((token) => (
                <div key={token.id} className="px-6 py-4 flex items-center justify-between">
                  <div>
                    <h4 className="font-semibold text-slate-900">{token.name}</h4>
                    <div className="flex items-center space-x-4 mt-1 text-sm text-slate-500">
                      <span>Created: {new Date(token.created_at).toLocaleDateString()}</span>
                      {token.last_used_at && (
                        <span>Last used: {new Date(token.last_used_at).toLocaleDateString()}</span>
                      )}
                      {token.expires_at && (
                        <span className={new Date(token.expires_at) < new Date() ? 'text-red-600' : ''}>
                          Expires: {new Date(token.expires_at).toLocaleDateString()}
                        </span>
                      )}
                    </div>
                  </div>
                  <button
                    onClick={() => handleRevokeToken(token.id)}
                    className="text-red-600 hover:text-red-700 text-sm font-medium"
                  >
                    Revoke
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Logs Tab */}
      {activeTab === 'logs' && (
        <div className="bg-white rounded-lg shadow border border-slate-200">
          <div className="px-6 py-4 border-b border-slate-200 flex items-center justify-between">
            <h3 className="text-lg font-semibold text-slate-900">Provisioning Logs</h3>
            <button onClick={fetchLogs} className="btn-secondary">
              <svg className="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Refresh
            </button>
          </div>

          {logs.length === 0 ? (
            <div className="p-12 text-center">
              <div className="w-16 h-16 bg-slate-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg className="w-8 h-8 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                </svg>
              </div>
              <h3 className="text-lg font-semibold text-slate-900 mb-2">No provisioning activity</h3>
              <p className="text-slate-600">Logs will appear here once your identity provider starts syncing users.</p>
            </div>
          ) : (
            <div className="divide-y divide-slate-200 max-h-[600px] overflow-y-auto">
              {logs.map((log) => (
                <div key={log.id} className="px-6 py-4">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center space-x-3">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${getActionBadge(log.action)}`}>
                        {log.action}
                      </span>
                      <span className="text-sm font-medium text-slate-900">{log.resource_type}</span>
                    </div>
                    <span className="text-xs text-slate-500">
                      {new Date(log.created_at).toLocaleString()}
                    </span>
                  </div>
                  {log.resource_id && (
                    <p className="text-sm text-slate-600">
                      Resource ID: <code className="bg-slate-100 px-1 rounded">{log.resource_id}</code>
                    </p>
                  )}
                  {log.details && (
                    <p className="text-sm text-slate-500 mt-1">{log.details}</p>
                  )}
                  {log.error && (
                    <p className="text-sm text-red-600 mt-1 bg-red-50 px-2 py-1 rounded">{log.error}</p>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Guides Tab */}
      {activeTab === 'guides' && (
        <div className="space-y-6">
          {/* Okta Guide */}
          <div className="bg-white rounded-lg shadow border border-slate-200 p-6">
            <div className="flex items-center space-x-4 mb-4">
              <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
                <span className="text-2xl">🔶</span>
              </div>
              <div>
                <h3 className="text-lg font-semibold text-slate-900">Okta</h3>
                <p className="text-sm text-slate-600">Configure SCIM provisioning with Okta</p>
              </div>
            </div>
            <ol className="space-y-3 text-sm text-slate-700 list-decimal list-inside">
              <li>In Okta Admin Console, go to <strong>Applications</strong> → <strong>Applications</strong></li>
              <li>Select your CoreAuth application or create a new SCIM app</li>
              <li>Go to the <strong>Provisioning</strong> tab and click <strong>Configure API Integration</strong></li>
              <li>Check <strong>Enable API Integration</strong></li>
              <li>Enter the SCIM endpoint URL: <code className="bg-slate-100 px-1 rounded">{scimEndpoint}</code></li>
              <li>Enter your SCIM token in the <strong>API Token</strong> field</li>
              <li>Click <strong>Test API Credentials</strong> to verify the connection</li>
              <li>Save and enable the provisioning features you need (Create Users, Update User Attributes, Deactivate Users)</li>
            </ol>
          </div>

          {/* Azure AD Guide */}
          <div className="bg-white rounded-lg shadow border border-slate-200 p-6">
            <div className="flex items-center space-x-4 mb-4">
              <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
                <span className="text-2xl">🔷</span>
              </div>
              <div>
                <h3 className="text-lg font-semibold text-slate-900">Microsoft Entra ID (Azure AD)</h3>
                <p className="text-sm text-slate-600">Configure SCIM provisioning with Microsoft Entra ID</p>
              </div>
            </div>
            <ol className="space-y-3 text-sm text-slate-700 list-decimal list-inside">
              <li>In Azure Portal, go to <strong>Microsoft Entra ID</strong> → <strong>Enterprise Applications</strong></li>
              <li>Create a new application or select an existing one</li>
              <li>Go to <strong>Provisioning</strong> and set mode to <strong>Automatic</strong></li>
              <li>In Admin Credentials, enter:
                <ul className="ml-6 mt-2 space-y-1 list-disc">
                  <li>Tenant URL: <code className="bg-slate-100 px-1 rounded">{scimEndpoint}</code></li>
                  <li>Secret Token: Your SCIM access token</li>
                </ul>
              </li>
              <li>Click <strong>Test Connection</strong> to verify</li>
              <li>Configure attribute mappings as needed</li>
              <li>Set Provisioning Status to <strong>On</strong> and save</li>
            </ol>
          </div>

          {/* OneLogin Guide */}
          <div className="bg-white rounded-lg shadow border border-slate-200 p-6">
            <div className="flex items-center space-x-4 mb-4">
              <div className="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center">
                <span className="text-2xl">🟣</span>
              </div>
              <div>
                <h3 className="text-lg font-semibold text-slate-900">OneLogin</h3>
                <p className="text-sm text-slate-600">Configure SCIM provisioning with OneLogin</p>
              </div>
            </div>
            <ol className="space-y-3 text-sm text-slate-700 list-decimal list-inside">
              <li>In OneLogin Admin Portal, go to <strong>Applications</strong></li>
              <li>Add or select your CoreAuth application</li>
              <li>Go to the <strong>Provisioning</strong> tab</li>
              <li>Enable provisioning and select <strong>SCIM</strong></li>
              <li>Enter the SCIM endpoint: <code className="bg-slate-100 px-1 rounded">{scimEndpoint}</code></li>
              <li>Enter your SCIM bearer token</li>
              <li>Configure user provisioning rules and attribute mappings</li>
              <li>Save and test the connection</li>
            </ol>
          </div>

          {/* Generic SCIM Info */}
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
            <h4 className="font-semibold text-blue-900 mb-3">SCIM 2.0 Supported Operations</h4>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm text-blue-800">
              <div>
                <h5 className="font-medium mb-2">Users</h5>
                <ul className="space-y-1">
                  <li>• GET /Users - List users with filtering</li>
                  <li>• GET /Users/:id - Get user by ID</li>
                  <li>• POST /Users - Create user</li>
                  <li>• PUT /Users/:id - Replace user</li>
                  <li>• PATCH /Users/:id - Update user</li>
                  <li>• DELETE /Users/:id - Deactivate user</li>
                </ul>
              </div>
              <div>
                <h5 className="font-medium mb-2">Groups</h5>
                <ul className="space-y-1">
                  <li>• GET /Groups - List groups</li>
                  <li>• GET /Groups/:id - Get group by ID</li>
                  <li>• POST /Groups - Create group</li>
                  <li>• PATCH /Groups/:id - Update membership</li>
                  <li>• DELETE /Groups/:id - Delete group</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Create Token Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-md w-full">
            <div className="p-6 border-b border-slate-200">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-bold text-slate-900">Generate SCIM Token</h2>
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="text-slate-400 hover:text-slate-600"
                >
                  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>

            <form onSubmit={handleCreateToken} className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Token Name
                </label>
                <input
                  type="text"
                  value={tokenName}
                  onChange={(e) => setTokenName(e.target.value)}
                  className="input-field"
                  placeholder="e.g., Okta Production"
                />
                <p className="text-xs text-slate-500 mt-1">
                  A descriptive name to identify this token
                </p>
              </div>

              <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                <div className="flex">
                  <svg className="w-5 h-5 text-yellow-600 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                  <p className="text-sm text-yellow-800">
                    The token will only be shown once. Make sure to copy and store it securely.
                  </p>
                </div>
              </div>

              <div className="flex space-x-3">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="flex-1 px-4 py-2 border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50"
                >
                  Cancel
                </button>
                <button type="submit" className="flex-1 btn-primary">
                  Generate Token
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Show Token Modal */}
      {showTokenModal && newToken && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-lg w-full">
            <div className="p-6 border-b border-slate-200">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-bold text-slate-900">Your SCIM Token</h2>
                <button
                  onClick={() => {
                    setShowTokenModal(false);
                    setNewToken(null);
                  }}
                  className="text-slate-400 hover:text-slate-600"
                >
                  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>

            <div className="p-6 space-y-4">
              <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                <div className="flex">
                  <svg className="w-5 h-5 text-red-600 mr-2 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                  <p className="text-sm text-red-800">
                    <strong>Important:</strong> This token will not be shown again. Copy it now and store it securely.
                  </p>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Bearer Token
                </label>
                <div className="flex items-center space-x-2">
                  <code className="flex-1 bg-slate-100 px-4 py-3 rounded-lg text-sm font-mono text-slate-800 break-all">
                    {newToken}
                  </code>
                  <button
                    onClick={() => copyToClipboard(newToken)}
                    className="btn-secondary px-4 py-3"
                  >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                  </button>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Authorization Header
                </label>
                <code className="block bg-slate-100 px-4 py-3 rounded-lg text-sm font-mono text-slate-800">
                  Authorization: Bearer {newToken.substring(0, 20)}...
                </code>
              </div>

              <button
                onClick={() => {
                  setShowTokenModal(false);
                  setNewToken(null);
                }}
                className="w-full btn-primary"
              >
                Done
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
