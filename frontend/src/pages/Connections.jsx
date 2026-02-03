import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import Layout from '../components/Layout';
import api from '../lib/api';

export default function Connections() {
  const navigate = useNavigate();
  const [connections, setConnections] = useState([]);
  const [templates, setTemplates] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showModal, setShowModal] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState(null);
  const [formData, setFormData] = useState({
    name: '',
    client_id: '',
    client_secret: '',
    domain: '',
  });

  useEffect(() => {
    fetchConnections();
    fetchTemplates();
  }, []);

  const fetchConnections = async () => {
    try {
      const response = await api.get('/oidc/providers');
      setConnections(response.data);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to fetch connections');
    } finally {
      setLoading(false);
    }
  };

  const fetchTemplates = async () => {
    try {
      const response = await api.get('/oidc/templates');
      setTemplates(response.data);
    } catch (err) {
      console.error('Failed to fetch templates:', err);
    }
  };

  const handleSelectTemplate = (template) => {
    setSelectedTemplate(template);
    setFormData({
      name: `${template.name} SSO`,
      client_id: '',
      client_secret: '',
      domain: '',
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

    try {
      await api.post('/oidc/providers', {
        provider_type: selectedTemplate.provider_type,
        provider_name: formData.name,
        client_id: formData.client_id,
        client_secret: formData.client_secret,
        domain_hint: formData.domain || undefined,
      });

      setShowModal(false);
      fetchConnections();
      setFormData({
        name: '',
        client_id: '',
        client_secret: '',
        domain: '',
      });
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to create connection');
    }
  };

  if (loading) {
    return (
      <Layout>
        <div className="flex items-center justify-center h-64">
          <div className="text-slate-600">Loading connections...</div>
        </div>
      </Layout>
    );
  }

  return (
    <Layout>
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
                      <h3 className="font-semibold text-slate-900">{conn.provider_name}</h3>
                      <p className="text-sm text-slate-600">{conn.provider_type}</p>
                      {conn.domain_hint && (
                        <p className="text-xs text-slate-500">Domain: {conn.domain_hint}</p>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${
                      conn.is_enabled
                        ? 'bg-green-100 text-green-700'
                        : 'bg-slate-100 text-slate-700'
                    }`}>
                      {conn.is_enabled ? 'Active' : 'Inactive'}
                    </span>
                    <button className="text-slate-600 hover:text-primary-600 p-2">
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
                  <h3 className="font-semibold text-slate-900">{template.name}</h3>
                  <p className="text-sm text-slate-600 mt-1">{template.description}</p>
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
                    Setup {selectedTemplate.name}
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

                {selectedTemplate.provider_type === 'azure_ad' && (
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-2">
                      Domain (optional)
                    </label>
                    <input
                      type="text"
                      name="domain"
                      value={formData.domain}
                      onChange={handleChange}
                      className="input-field"
                      placeholder="company.com"
                    />
                    <p className="text-xs text-slate-500 mt-1">
                      Domain hint for Azure AD login
                    </p>
                  </div>
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
      </div>
    </Layout>
  );
}

function getProviderIcon(providerType) {
  const icons = {
    google: '🔵',
    azure_ad: '🔷',
    microsoft: '🔷',
    okta: '🔶',
    auth0: '🟠',
    generic: '🔗',
  };
  return icons[providerType] || '🔗';
}
