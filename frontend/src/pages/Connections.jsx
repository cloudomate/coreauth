import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import api from '../lib/api';

export default function Connections() {
  const navigate = useNavigate();
  const [connections, setConnections] = useState([]);
  const [templates, setTemplates] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [showModal, setShowModal] = useState(false);
  const [showSettingsModal, setShowSettingsModal] = useState(false);
  const [selectedConnection, setSelectedConnection] = useState(null);
  const [selectedTemplate, setSelectedTemplate] = useState(null);
  const [formData, setFormData] = useState({
    name: '',
    client_id: '',
    client_secret: '',
    // Azure AD / Microsoft Entra ID
    azure_tenant_id: '',
    domain_hint: '',
    // Okta
    okta_domain: '',
    // Google
    hosted_domain: '',
    // Generic OIDC
    issuer_url: '',
    authorization_endpoint: '',
    token_endpoint: '',
    userinfo_endpoint: '',
  });

  // Get user/tenant info from localStorage
  const user = JSON.parse(localStorage.getItem('user') || '{}');
  // The backend returns default_organization_id, not tenant_id
  const tenantId = user.default_organization_id;

  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      try {
        // Fetch templates (always)
        const templatesRes = await api.get('/oidc/templates');
        setTemplates(templatesRes.data);

        // Fetch connections (only if we have a tenant)
        if (tenantId) {
          const connectionsRes = await api.get(`/oidc/providers?tenant_id=${tenantId}`);
          setConnections(connectionsRes.data);
        }
      } catch (err) {
        setError(err.response?.data?.message || 'Failed to load data');
      } finally {
        setLoading(false);
      }
    };

    loadData();
  }, [tenantId]);


  const handleSelectTemplate = (template) => {
    setSelectedTemplate(template);
    setFormData({
      name: `${template.display_name} SSO`,
      client_id: '',
      client_secret: '',
      azure_tenant_id: '',
      domain_hint: '',
      okta_domain: '',
      hosted_domain: '',
      issuer_url: '',
      authorization_endpoint: '',
      token_endpoint: '',
      userinfo_endpoint: '',
    });
    setShowModal(true);
  };

  const handleChange = (e) => {
    setFormData({
      ...formData,
      [e.target.name]: e.target.value,
    });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');

    const providerType = selectedTemplate.provider_type;

    // Build the full provider config from template + user input
    let issuer = selectedTemplate.issuer_pattern || '';
    let authorizationEndpoint = selectedTemplate.authorization_endpoint || '';
    let tokenEndpoint = selectedTemplate.token_endpoint || '';
    let userinfoEndpoint = selectedTemplate.userinfo_endpoint || null;
    let jwksUri = selectedTemplate.jwks_uri || '';

    // Replace placeholders based on provider type
    if (providerType === 'azuread' || providerType === 'azure_ad' || providerType === 'microsoft') {
      if (!formData.azure_tenant_id) {
        setError('Azure Tenant ID is required');
        return;
      }
      const tenantIdPlaceholder = formData.azure_tenant_id;
      issuer = issuer.replace('{tenant_id}', tenantIdPlaceholder);
      authorizationEndpoint = authorizationEndpoint.replace('{tenant_id}', tenantIdPlaceholder);
      tokenEndpoint = tokenEndpoint.replace('{tenant_id}', tenantIdPlaceholder);
      jwksUri = jwksUri.replace('{tenant_id}', tenantIdPlaceholder);
    } else if (providerType === 'okta') {
      if (!formData.okta_domain) {
        setError('Okta Domain is required');
        return;
      }
      const domain = formData.okta_domain;
      issuer = `https://${domain}`;
      authorizationEndpoint = `https://${domain}/oauth2/v1/authorize`;
      tokenEndpoint = `https://${domain}/oauth2/v1/token`;
      userinfoEndpoint = `https://${domain}/oauth2/v1/userinfo`;
      jwksUri = `https://${domain}/oauth2/v1/keys`;
    } else if (providerType === 'auth0') {
      // Auth0 doesn't need special handling - endpoints are static
      // But the domain placeholder needs to be replaced if present
      // For auth0, the user might need to provide domain separately
      // For now, use the template as-is
    } else if (providerType === 'google') {
      // Google endpoints are static, no replacement needed
    } else if (providerType === 'generic') {
      if (!formData.issuer_url) {
        setError('Issuer URL is required');
        return;
      }
      issuer = formData.issuer_url;
      authorizationEndpoint = formData.authorization_endpoint || `${issuer}/authorize`;
      tokenEndpoint = formData.token_endpoint || `${issuer}/token`;
      userinfoEndpoint = formData.userinfo_endpoint || `${issuer}/userinfo`;
      jwksUri = `${issuer}/.well-known/jwks.json`;
    }

    const requestBody = {
      tenant_id: tenantId,
      provider_type: providerType,
      name: formData.name,
      client_id: formData.client_id,
      client_secret: formData.client_secret,
      issuer: issuer,
      authorization_endpoint: authorizationEndpoint,
      token_endpoint: tokenEndpoint,
      userinfo_endpoint: userinfoEndpoint,
      jwks_uri: jwksUri,
      scopes: selectedTemplate.scopes || ['openid', 'profile', 'email'],
      groups_claim: selectedTemplate.groups_claim || null,
    };

    console.log('Creating OIDC provider with:', JSON.stringify(requestBody, null, 2));

    try {
      await api.post('/oidc/providers', requestBody);

      setShowModal(false);
      // Reload connections
      const connectionsRes = await api.get(`/oidc/providers?tenant_id=${tenantId}`);
      setConnections(connectionsRes.data);
      setFormData({
        name: '',
        client_id: '',
        client_secret: '',
        azure_tenant_id: '',
        domain_hint: '',
        okta_domain: '',
        hosted_domain: '',
        issuer_url: '',
        authorization_endpoint: '',
        token_endpoint: '',
        userinfo_endpoint: '',
      });
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to create connection');
    }
  };

  const handleOpenSettings = (conn) => {
    setSelectedConnection(conn);
    setShowSettingsModal(true);
  };

  const handleDeleteConnection = async (connId) => {
    if (!confirm('Are you sure you want to delete this connection? Users will no longer be able to sign in with this provider.')) {
      return;
    }
    setError('');
    try {
      await api.delete(`/oidc/providers/${connId}`);
      setSuccess('Connection deleted successfully');
      setShowSettingsModal(false);
      // Reload connections
      const connectionsRes = await api.get(`/oidc/providers?tenant_id=${tenantId}`);
      setConnections(connectionsRes.data);
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to delete connection');
    }
  };

  const handleToggleConnection = async (connId, currentStatus) => {
    setError('');
    try {
      await api.patch(`/oidc/providers/${connId}`, {
        is_active: !currentStatus,
      });
      setSuccess(`Connection ${currentStatus ? 'disabled' : 'enabled'} successfully`);
      // Reload connections
      const connectionsRes = await api.get(`/oidc/providers?tenant_id=${tenantId}`);
      setConnections(connectionsRes.data);
      setShowSettingsModal(false);
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to update connection');
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-600">Loading connections...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-slate-900">SSO Connections</h1>
            <p className="text-slate-600 mt-1">
              Configure enterprise SSO providers for your organization
            </p>
          </div>
        </div>

        {error && (
          <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
            {error}
          </div>
        )}

        {success && (
          <div className="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded">
            {success}
          </div>
        )}

        {/* Existing Connections */}
        {connections.length > 0 && (
          <div className="bg-white rounded-lg shadow border border-slate-200">
            <div className="px-6 py-4 border-b border-slate-200">
              <h2 className="text-lg font-semibold text-slate-900">Active Connections</h2>
            </div>
            <div className="divide-y divide-slate-200">
              {connections.map((conn) => (
                <div key={conn.id} className="px-6 py-4 flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className="w-12 h-12 bg-primary-100 rounded-lg flex items-center justify-center">
                      <span className="text-2xl">{getProviderIcon(conn.provider_type)}</span>
                    </div>
                    <div>
                      <h3 className="font-semibold text-slate-900">{conn.name}</h3>
                      <p className="text-sm text-slate-600">{getProviderDisplayName(conn.provider_type)}</p>
                      <div className="text-xs text-slate-500 space-y-0.5 mt-1">
                        {conn.azure_tenant_id && (
                          <p>Tenant: {conn.azure_tenant_id.substring(0, 8)}...</p>
                        )}
                        {conn.okta_domain && (
                          <p>Domain: {conn.okta_domain}</p>
                        )}
                        {conn.hosted_domain && (
                          <p>Hosted Domain: {conn.hosted_domain}</p>
                        )}
                        {conn.issuer_url && (
                          <p>Issuer: {conn.issuer_url}</p>
                        )}
                        {conn.domain_hint && (
                          <p>Domain Hint: {conn.domain_hint}</p>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${
                      conn.is_active
                        ? 'bg-green-100 text-green-700'
                        : 'bg-slate-100 text-slate-700'
                    }`}>
                      {conn.is_active ? 'Active' : 'Inactive'}
                    </span>
                    <button
                      onClick={() => handleOpenSettings(conn)}
                      className="text-slate-600 hover:text-primary-600 p-2"
                      title="Connection Settings"
                    >
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      </svg>
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Add New Connection */}
        <div className="bg-white rounded-lg shadow border border-slate-200">
          <div className="px-6 py-4 border-b border-slate-200">
            <h2 className="text-lg font-semibold text-slate-900">Add Connection</h2>
            <p className="text-sm text-slate-600 mt-1">
              Choose a provider to configure SSO for your organization
            </p>
          </div>
          <div className="p-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {templates.map((template) => (
                <button
                  key={template.provider_type}
                  onClick={() => handleSelectTemplate(template)}
                  className="p-6 border-2 border-slate-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition-all text-left"
                >
                  <div className="text-4xl mb-3">{getProviderIcon(template.provider_type)}</div>
                  <h3 className="font-semibold text-slate-900">{template.display_name}</h3>
                  <p className="text-sm text-slate-600 mt-1 line-clamp-2">{template.instructions}</p>
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Setup Modal */}
        {showModal && selectedTemplate && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div className="bg-white rounded-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto">
              <div className="p-6 border-b border-slate-200">
                <div className="flex items-center justify-between">
                  <h2 className="text-2xl font-bold text-slate-900">
                    Setup {selectedTemplate.display_name}
                  </h2>
                  <button
                    onClick={() => setShowModal(false)}
                    className="text-slate-400 hover:text-slate-600"
                  >
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>

              <form onSubmit={handleSubmit} className="p-6 space-y-6">
                {error && (
                  <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
                    {error}
                  </div>
                )}

                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-2">
                    Connection Name
                  </label>
                  <input
                    type="text"
                    name="name"
                    required
                    value={formData.name}
                    onChange={handleChange}
                    className="input-field"
                    placeholder="e.g., Company SSO"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-2">
                    Client ID
                  </label>
                  <input
                    type="text"
                    name="client_id"
                    required
                    value={formData.client_id}
                    onChange={handleChange}
                    className="input-field"
                    placeholder="From your identity provider"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-2">
                    Client Secret
                  </label>
                  <input
                    type="password"
                    name="client_secret"
                    required
                    value={formData.client_secret}
                    onChange={handleChange}
                    className="input-field"
                    placeholder="From your identity provider"
                  />
                </div>

                {/* Azure AD / Microsoft Entra ID specific fields */}
                {(selectedTemplate.provider_type === 'azuread' || selectedTemplate.provider_type === 'azure_ad' || selectedTemplate.provider_type === 'microsoft') && (
                  <>
                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-2">
                        Azure Tenant ID <span className="text-red-500">*</span>
                      </label>
                      <input
                        type="text"
                        name="azure_tenant_id"
                        required
                        value={formData.azure_tenant_id}
                        onChange={handleChange}
                        className="input-field"
                        placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                      />
                      <p className="text-xs text-slate-500 mt-1">
                        Found in Azure Portal → Microsoft Entra ID → Overview → Tenant ID
                      </p>
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-2">
                        Domain Hint (optional)
                      </label>
                      <input
                        type="text"
                        name="domain_hint"
                        value={formData.domain_hint}
                        onChange={handleChange}
                        className="input-field"
                        placeholder="company.com"
                      />
                      <p className="text-xs text-slate-500 mt-1">
                        Pre-fill the login domain for users
                      </p>
                    </div>
                  </>
                )}

                {/* Okta specific fields */}
                {selectedTemplate.provider_type === 'okta' && (
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-2">
                      Okta Domain <span className="text-red-500">*</span>
                    </label>
                    <input
                      type="text"
                      name="okta_domain"
                      required
                      value={formData.okta_domain}
                      onChange={handleChange}
                      className="input-field"
                      placeholder="dev-12345.okta.com"
                    />
                    <p className="text-xs text-slate-500 mt-1">
                      Your Okta organization domain (without https://)
                    </p>
                  </div>
                )}

                {/* Google specific fields */}
                {selectedTemplate.provider_type === 'google' && (
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-2">
                      Hosted Domain (optional)
                    </label>
                    <input
                      type="text"
                      name="hosted_domain"
                      value={formData.hosted_domain}
                      onChange={handleChange}
                      className="input-field"
                      placeholder="company.com"
                    />
                    <p className="text-xs text-slate-500 mt-1">
                      Restrict login to a specific Google Workspace domain
                    </p>
                  </div>
                )}

                {/* Generic OIDC specific fields */}
                {selectedTemplate.provider_type === 'generic' && (
                  <>
                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-2">
                        Issuer URL <span className="text-red-500">*</span>
                      </label>
                      <input
                        type="url"
                        name="issuer_url"
                        required
                        value={formData.issuer_url}
                        onChange={handleChange}
                        className="input-field"
                        placeholder="https://idp.example.com"
                      />
                      <p className="text-xs text-slate-500 mt-1">
                        The OIDC issuer URL (used for discovery)
                      </p>
                    </div>
                    <div className="bg-slate-50 border border-slate-200 rounded-lg p-4">
                      <h5 className="text-sm font-medium text-slate-700 mb-3">
                        Manual Endpoint Configuration (optional)
                      </h5>
                      <p className="text-xs text-slate-500 mb-3">
                        Only needed if auto-discovery doesn't work
                      </p>
                      <div className="space-y-3">
                        <div>
                          <label className="block text-xs font-medium text-slate-600 mb-1">
                            Authorization Endpoint
                          </label>
                          <input
                            type="url"
                            name="authorization_endpoint"
                            value={formData.authorization_endpoint}
                            onChange={handleChange}
                            className="input-field text-sm"
                            placeholder="https://idp.example.com/authorize"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-slate-600 mb-1">
                            Token Endpoint
                          </label>
                          <input
                            type="url"
                            name="token_endpoint"
                            value={formData.token_endpoint}
                            onChange={handleChange}
                            className="input-field text-sm"
                            placeholder="https://idp.example.com/token"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-slate-600 mb-1">
                            UserInfo Endpoint
                          </label>
                          <input
                            type="url"
                            name="userinfo_endpoint"
                            value={formData.userinfo_endpoint}
                            onChange={handleChange}
                            className="input-field text-sm"
                            placeholder="https://idp.example.com/userinfo"
                          />
                        </div>
                      </div>
                    </div>
                  </>
                )}

                <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                  <h4 className="font-semibold text-blue-900 mb-2">Callback URL</h4>
                  <code className="text-sm text-blue-700 bg-white px-3 py-2 rounded border border-blue-200 block">
                    {window.location.origin.replace(':3000', ':8000')}/api/oidc/callback
                  </code>
                  <p className="text-xs text-blue-600 mt-2">
                    Configure this URL in your identity provider settings
                  </p>
                </div>

                <div className="flex space-x-3">
                  <button
                    type="button"
                    onClick={() => setShowModal(false)}
                    className="flex-1 px-6 py-3 border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50"
                  >
                    Cancel
                  </button>
                  <button
                    type="submit"
                    className="flex-1 btn-primary py-3"
                  >
                    Create Connection
                  </button>
                </div>
              </form>
            </div>
          </div>
        )}

        {/* Settings Modal */}
        {showSettingsModal && selectedConnection && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div className="bg-white rounded-lg max-w-lg w-full">
              <div className="p-6 border-b border-slate-200">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <span className="text-3xl">{getProviderIcon(selectedConnection.provider_type)}</span>
                    <div>
                      <h2 className="text-xl font-bold text-slate-900">
                        {selectedConnection.name}
                      </h2>
                      <p className="text-sm text-slate-600">
                        {getProviderDisplayName(selectedConnection.provider_type)}
                      </p>
                    </div>
                  </div>
                  <button
                    onClick={() => setShowSettingsModal(false)}
                    className="text-slate-400 hover:text-slate-600"
                  >
                    <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>

              <div className="p-6 space-y-4">
                <div className="flex items-center justify-between py-3 border-b border-slate-100">
                  <div>
                    <h3 className="font-medium text-slate-900">Connection Status</h3>
                    <p className="text-sm text-slate-500">
                      {selectedConnection.is_active
                        ? 'Users can sign in with this provider'
                        : 'This connection is disabled'}
                    </p>
                  </div>
                  <span className={`px-3 py-1 rounded-full text-xs font-medium ${
                    selectedConnection.is_active
                      ? 'bg-green-100 text-green-700'
                      : 'bg-slate-100 text-slate-700'
                  }`}>
                    {selectedConnection.is_active ? 'Active' : 'Inactive'}
                  </span>
                </div>

                <div className="py-3 border-b border-slate-100">
                  <h3 className="font-medium text-slate-900 mb-2">Connection ID</h3>
                  <code className="text-sm text-slate-600 bg-slate-100 px-2 py-1 rounded">
                    {selectedConnection.id}
                  </code>
                </div>

                <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                  <h4 className="font-semibold text-blue-900 mb-2">How it works</h4>
                  <p className="text-sm text-blue-700">
                    When users click "Continue with {getProviderDisplayName(selectedConnection.provider_type)}"
                    on your login page, they'll be redirected to authenticate with their corporate identity provider.
                    After successful authentication, they'll be signed into your application.
                  </p>
                </div>
              </div>

              <div className="p-6 border-t border-slate-200 space-y-3">
                <button
                  onClick={() => handleToggleConnection(selectedConnection.id, selectedConnection.is_active)}
                  className={`w-full py-2.5 rounded-lg font-medium transition-colors ${
                    selectedConnection.is_active
                      ? 'bg-amber-100 text-amber-700 hover:bg-amber-200'
                      : 'bg-green-100 text-green-700 hover:bg-green-200'
                  }`}
                >
                  {selectedConnection.is_active ? 'Disable Connection' : 'Enable Connection'}
                </button>
                <button
                  onClick={() => handleDeleteConnection(selectedConnection.id)}
                  className="w-full py-2.5 bg-red-100 text-red-700 hover:bg-red-200 rounded-lg font-medium transition-colors"
                >
                  Delete Connection
                </button>
                <button
                  onClick={() => setShowSettingsModal(false)}
                  className="w-full py-2.5 border border-slate-300 text-slate-700 hover:bg-slate-50 rounded-lg font-medium transition-colors"
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        )}
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
    google: 'Google Workspace',
    azuread: 'Microsoft Entra ID',
    azure_ad: 'Microsoft Entra ID',
    microsoft: 'Microsoft Entra ID',
    okta: 'Okta',
    auth0: 'Auth0',
    generic: 'Generic OIDC',
  };
  return names[providerType] || providerType;
}
