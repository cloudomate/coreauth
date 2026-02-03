import { useState, useEffect } from 'react';
import { applicationApi } from '../api/client';

export default function Applications() {
  const [applications, setApplications] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedApp, setSelectedApp] = useState(null);
  const [newSecret, setNewSecret] = useState(null);
  const orgId = 'demo-org-id'; // TODO: Get from context/state

  useEffect(() => {
    loadApplications();
  }, []);

  const loadApplications = async () => {
    try {
      const { data } = await applicationApi.list(orgId);
      setApplications(data.applications || []);
    } catch (error) {
      console.error('Failed to load applications:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async (formData) => {
    try {
      const { data } = await applicationApi.create(orgId, formData);
      setNewSecret(data.client_secret_plain);
      await loadApplications();
      setShowCreateModal(false);
    } catch (error) {
      console.error('Failed to create application:', error);
      alert('Failed to create application: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleDelete = async (appId) => {
    if (!confirm('Are you sure you want to delete this application?')) return;
    try {
      await applicationApi.delete(orgId, appId);
      await loadApplications();
    } catch (error) {
      console.error('Failed to delete application:', error);
      alert('Failed to delete application');
    }
  };

  const handleRotateSecret = async (appId) => {
    if (!confirm('This will invalidate the current secret. Continue?')) return;
    try {
      const { data } = await applicationApi.rotateSecret(orgId, appId);
      setNewSecret(data.client_secret_plain);
      alert('Secret rotated successfully! Make sure to save the new secret.');
    } catch (error) {
      console.error('Failed to rotate secret:', error);
      alert('Failed to rotate secret');
    }
  };

  const handleToggleEnabled = async (app) => {
    try {
      await applicationApi.update(orgId, app.id, { is_enabled: !app.is_enabled });
      await loadApplications();
    } catch (error) {
      console.error('Failed to update application:', error);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-slate-50 flex items-center justify-center">
        <div className="text-slate-600">Loading...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-slate-50">
      <div className="max-w-7xl mx-auto px-6 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-slate-900">Applications</h1>
            <p className="text-slate-600 mt-1">Manage OAuth applications for your organization</p>
          </div>
          <button
            onClick={() => setShowCreateModal(true)}
            className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
          >
            Create Application
          </button>
        </div>

        {/* New Secret Alert */}
        {newSecret && (
          <div className="mb-6 p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
            <h3 className="font-semibold text-yellow-900 mb-2">Save your client secret!</h3>
            <p className="text-sm text-yellow-800 mb-2">
              This is the only time you'll see this secret. Store it securely.
            </p>
            <code className="block p-3 bg-yellow-100 rounded text-sm font-mono break-all">
              {newSecret}
            </code>
            <button
              onClick={() => setNewSecret(null)}
              className="mt-2 text-sm text-yellow-700 hover:text-yellow-900"
            >
              I've saved the secret
            </button>
          </div>
        )}

        {/* Applications Grid */}
        {applications.length === 0 ? (
          <div className="card text-center py-12">
            <svg className="w-16 h-16 text-slate-300 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
            </svg>
            <h3 className="text-lg font-semibold text-slate-900 mb-2">No applications yet</h3>
            <p className="text-slate-600 mb-4">Create your first OAuth application to get started</p>
            <button
              onClick={() => setShowCreateModal(true)}
              className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
            >
              Create Application
            </button>
          </div>
        ) : (
          <div className="grid gap-6">
            {applications.map((app) => (
              <div key={app.id} className="card">
                <div className="flex items-start justify-between mb-4">
                  <div className="flex-1">
                    <div className="flex items-center space-x-3 mb-2">
                      <h3 className="text-xl font-semibold text-slate-900">{app.name}</h3>
                      <span className={`px-2 py-1 text-xs rounded-full ${
                        app.is_enabled
                          ? 'bg-green-100 text-green-700'
                          : 'bg-slate-100 text-slate-600'
                      }`}>
                        {app.is_enabled ? 'Enabled' : 'Disabled'}
                      </span>
                      <span className="px-2 py-1 text-xs rounded-full bg-blue-100 text-blue-700">
                        {app.app_type}
                      </span>
                    </div>
                    {app.description && (
                      <p className="text-slate-600 mb-3">{app.description}</p>
                    )}
                    <div className="space-y-2">
                      <div>
                        <span className="text-sm font-medium text-slate-700">Client ID:</span>
                        <code className="ml-2 text-sm font-mono bg-slate-100 px-2 py-1 rounded">
                          {app.client_id}
                        </code>
                      </div>
                      {app.callback_urls && app.callback_urls.length > 0 && (
                        <div>
                          <span className="text-sm font-medium text-slate-700">Callback URLs:</span>
                          <div className="ml-2 text-sm text-slate-600">
                            {app.callback_urls.map((url, i) => (
                              <div key={i}>{url}</div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    <button
                      onClick={() => handleToggleEnabled(app)}
                      className="p-2 text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors"
                      title={app.is_enabled ? 'Disable' : 'Enable'}
                    >
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleRotateSecret(app.id)}
                      className="p-2 text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors"
                      title="Rotate Secret"
                    >
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                      </svg>
                    </button>
                    <button
                      onClick={() => setSelectedApp(app)}
                      className="p-2 text-slate-600 hover:text-slate-900 hover:bg-slate-100 rounded-lg transition-colors"
                      title="Edit"
                    >
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleDelete(app.id)}
                      className="p-2 text-red-600 hover:text-red-700 hover:bg-red-50 rounded-lg transition-colors"
                      title="Delete"
                    >
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Create Modal */}
        {showCreateModal && (
          <CreateApplicationModal
            onClose={() => setShowCreateModal(false)}
            onCreate={handleCreate}
          />
        )}

        {/* Edit Modal */}
        {selectedApp && (
          <EditApplicationModal
            app={selectedApp}
            onClose={() => setSelectedApp(null)}
            onUpdate={async (updates) => {
              await applicationApi.update(orgId, selectedApp.id, updates);
              await loadApplications();
              setSelectedApp(null);
            }}
          />
        )}
      </div>
    </div>
  );
}

function CreateApplicationModal({ onClose, onCreate }) {
  const [formData, setFormData] = useState({
    name: '',
    slug: '',
    description: '',
    app_type: 'web',
    callback_urls: '',
    logout_urls: '',
    web_origins: '',
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    onCreate({
      ...formData,
      callback_urls: formData.callback_urls.split('\n').filter(Boolean),
      logout_urls: formData.logout_urls.split('\n').filter(Boolean),
      web_origins: formData.web_origins.split('\n').filter(Boolean),
    });
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
        <h2 className="text-2xl font-bold mb-4">Create Application</h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Name</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Slug</label>
            <input
              type="text"
              value={formData.slug}
              onChange={(e) => setFormData({ ...formData, slug: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              rows={2}
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Application Type</label>
            <select
              value={formData.app_type}
              onChange={(e) => setFormData({ ...formData, app_type: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            >
              <option value="web">Web Application</option>
              <option value="spa">Single Page Application</option>
              <option value="native">Native Application</option>
              <option value="api">Machine to Machine</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Callback URLs (one per line)</label>
            <textarea
              value={formData.callback_urls}
              onChange={(e) => setFormData({ ...formData, callback_urls: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent font-mono text-sm"
              rows={3}
              placeholder="https://example.com/callback"
            />
          </div>
          <div className="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-slate-700 hover:bg-slate-100 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
            >
              Create Application
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

function EditApplicationModal({ app, onClose, onUpdate }) {
  const [formData, setFormData] = useState({
    name: app.name,
    description: app.description || '',
    callback_urls: (app.callback_urls || []).join('\n'),
    is_enabled: app.is_enabled,
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    onUpdate({
      ...formData,
      callback_urls: formData.callback_urls.split('\n').filter(Boolean),
    });
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 max-w-2xl w-full mx-4">
        <h2 className="text-2xl font-bold mb-4">Edit Application</h2>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Name</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              rows={2}
            />
          </div>
          <div>
            <label className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={formData.is_enabled}
                onChange={(e) => setFormData({ ...formData, is_enabled: e.target.checked })}
                className="rounded border-slate-300"
              />
              <span className="text-sm font-medium text-slate-700">Enabled</span>
            </label>
          </div>
          <div className="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-slate-700 hover:bg-slate-100 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
            >
              Update
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
