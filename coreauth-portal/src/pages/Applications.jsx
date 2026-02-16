import { useState, useEffect } from 'react';
import { applicationApi } from '../api/client';

export default function Applications() {
  const [applications, setApplications] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [editingApp, setEditingApp] = useState(null);
  const [newSecret, setNewSecret] = useState(null);
  const [error, setError] = useState(null);

  const getOrgId = () => {
    try {
      const userData = localStorage.getItem('user');
      if (userData) {
        const user = JSON.parse(userData);
        return user.default_tenant_id || user.tenant_id;
      }
    } catch (e) {
      console.error('Failed to parse user data:', e);
    }
    return null;
  };

  const orgId = getOrgId();

  useEffect(() => {
    if (orgId) {
      loadApplications();
    } else {
      setError('No organization found. Please log in again.');
      setLoading(false);
    }
  }, [orgId]);

  const loadApplications = async () => {
    try {
      setError(null);
      const { data } = await applicationApi.list(orgId);
      setApplications(data.applications || data || []);
    } catch (error) {
      console.error('Failed to load applications:', error);
      setError('Failed to load applications: ' + (error.response?.data?.message || error.message));
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async (formData) => {
    try {
      const { data } = await applicationApi.create(orgId, formData);
      setNewSecret(data.client_secret_plain);
      await loadApplications();
      setShowCreateForm(false);
    } catch (error) {
      console.error('Failed to create application:', error);
      alert('Failed to create application: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleUpdate = async (appId, updates) => {
    try {
      await applicationApi.update(orgId, appId, updates);
      await loadApplications();
      setEditingApp(null);
    } catch (error) {
      console.error('Failed to update application:', error);
      alert('Failed to update application: ' + (error.response?.data?.message || error.message));
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
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-500">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div>
        <div className="p-4 bg-red-50 border border-red-200 rounded-md text-red-700 text-sm">{error}</div>
      </div>
    );
  }

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold text-slate-900">Applications</h1>
          <p className="text-sm text-slate-500 mt-0.5">Register and configure applications that use CoreAuth for authentication</p>
        </div>
        {!showCreateForm && !editingApp && (
          <button onClick={() => setShowCreateForm(true)} className="btn-primary">
            Create Application
          </button>
        )}
      </div>

      {/* Secret Alert */}
      {newSecret && (
        <div className="mb-4 p-3 bg-amber-50 border border-amber-200 rounded-md">
          <div className="flex items-start justify-between gap-3">
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-amber-800">Save your client secret</p>
              <p className="text-xs text-amber-700 mt-0.5">This is the only time you'll see this secret.</p>
              <code className="block mt-2 p-2 bg-amber-100 rounded text-xs font-mono break-all">{newSecret}</code>
            </div>
            <button onClick={() => setNewSecret(null)} className="text-amber-600 hover:text-amber-800 text-lg leading-none">&times;</button>
          </div>
        </div>
      )}

      {/* Create Form */}
      {showCreateForm && (
        <div className="mb-4 p-4 bg-white border border-slate-200 rounded-md">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-medium text-slate-900">New Application</h2>
            <button onClick={() => setShowCreateForm(false)} className="text-slate-400 hover:text-slate-600">&times;</button>
          </div>
          <ApplicationForm onSubmit={handleCreate} onCancel={() => setShowCreateForm(false)} />
        </div>
      )}

      {/* Applications List */}
      {applications.length === 0 && !showCreateForm ? (
        <div className="text-center py-12 bg-white border border-slate-200 rounded-md">
          <p className="text-slate-500 text-sm mb-3">No applications yet</p>
          <button onClick={() => setShowCreateForm(true)} className="btn-primary">Create Application</button>
        </div>
      ) : (
        <div className="space-y-3">
          {applications.map((app) => (
            <div key={app.id} className="bg-white border border-slate-200 rounded-md">
              {editingApp?.id === app.id ? (
                <div className="p-4">
                  <div className="flex items-center justify-between mb-4">
                    <h2 className="text-sm font-medium text-slate-900">Edit Application</h2>
                    <button onClick={() => setEditingApp(null)} className="text-slate-400 hover:text-slate-600">&times;</button>
                  </div>
                  <div className="mb-3 py-2 px-3 bg-slate-50 rounded text-xs">
                    <span className="text-slate-500">Client ID:</span>
                    <code className="ml-2 font-mono">{app.client_id}</code>
                  </div>
                  <ApplicationForm
                    app={app}
                    onSubmit={(updates) => handleUpdate(app.id, updates)}
                    onCancel={() => setEditingApp(null)}
                    isEdit
                  />
                </div>
              ) : (
                <div className="p-4 flex items-start justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <h3 className="text-sm font-medium text-slate-900">{app.name}</h3>
                      <span className={`px-1.5 py-0.5 text-xs rounded ${app.is_enabled ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-500'}`}>
                        {app.is_enabled ? 'Active' : 'Inactive'}
                      </span>
                      <span className="px-1.5 py-0.5 text-xs rounded bg-slate-100 text-slate-600">{app.app_type}</span>
                    </div>
                    {app.description && <p className="text-xs text-slate-500 mt-1">{app.description}</p>}
                    <div className="mt-2 space-y-1 text-xs">
                      <div className="flex items-center gap-2">
                        <span className="text-slate-400 w-16">Client ID</span>
                        <code className="font-mono text-slate-600">{app.client_id}</code>
                      </div>
                      {app.callback_urls?.length > 0 && (
                        <div className="flex items-start gap-2">
                          <span className="text-slate-400 w-16">Callbacks</span>
                          <span className="text-slate-600">{app.callback_urls.join(', ')}</span>
                        </div>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <IconButton onClick={() => handleToggleEnabled(app)} title={app.is_enabled ? 'Disable' : 'Enable'} icon="toggle" />
                    <IconButton onClick={() => handleRotateSecret(app.id)} title="Rotate Secret" icon="refresh" />
                    <IconButton onClick={() => setEditingApp(app)} title="Edit" icon="edit" />
                    <IconButton onClick={() => handleDelete(app.id)} title="Delete" icon="delete" danger />
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function IconButton({ onClick, title, icon, danger }) {
  const icons = {
    toggle: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />,
    refresh: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />,
    edit: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />,
    delete: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />,
  };
  return (
    <button
      onClick={onClick}
      title={title}
      className={`p-1.5 rounded hover:bg-slate-100 ${danger ? 'text-red-500 hover:text-red-600 hover:bg-red-50' : 'text-slate-400 hover:text-slate-600'}`}
    >
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">{icons[icon]}</svg>
    </button>
  );
}

function ApplicationForm({ app, onSubmit, onCancel, isEdit }) {
  const [formData, setFormData] = useState({
    name: app?.name || '',
    description: app?.description || '',
    app_type: app?.app_type || 'webapp',
    callback_urls: app?.callback_urls?.length ? app.callback_urls : [''],
    logout_urls: app?.logout_urls?.length ? app.logout_urls : [''],
    grant_types: app?.grant_types || ['authorization_code', 'refresh_token'],
    allowed_scopes: app?.allowed_scopes || ['openid', 'profile', 'email'],
    is_enabled: app?.is_enabled ?? true,
  });

  // Auto-generate slug from name
  const generateSlug = (name) => {
    return 'coreauth-' + name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
  };

  const generatedSlug = formData.name ? generateSlug(formData.name) : '';

  const grantTypes = [
    { value: 'authorization_code', label: 'Auth Code' },
    { value: 'refresh_token', label: 'Refresh' },
    { value: 'client_credentials', label: 'Client Creds' },
  ];

  const scopes = ['openid', 'profile', 'email', 'offline_access'];

  const toggle = (arr, val) => arr.includes(val) ? arr.filter(v => v !== val) : [...arr, val];

  const updateUrl = (field, index, value) => {
    const urls = [...formData[field]];
    urls[index] = value;
    setFormData({ ...formData, [field]: urls });
  };

  const addUrl = (field) => {
    setFormData({ ...formData, [field]: [...formData[field], ''] });
  };

  const removeUrl = (field, index) => {
    const urls = formData[field].filter((_, i) => i !== index);
    setFormData({ ...formData, [field]: urls.length ? urls : [''] });
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit({
      ...formData,
      slug: isEdit ? undefined : generatedSlug,
      callback_urls: formData.callback_urls.filter(Boolean),
      logout_urls: formData.logout_urls.filter(Boolean),
    });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="grid grid-cols-2 gap-3">
        <FormField label="Name" required>
          <input type="text" value={formData.name} onChange={(e) => setFormData({ ...formData, name: e.target.value })} className="input-field" required />
          {!isEdit && generatedSlug && (
            <p className="text-xs text-slate-500 mt-1">Slug: <code className="bg-slate-100 px-1 rounded">{generatedSlug}</code></p>
          )}
        </FormField>
        <FormField label="Type">
          <select value={formData.app_type} onChange={(e) => setFormData({ ...formData, app_type: e.target.value })} className="input-field" disabled={isEdit}>
            <option value="webapp">Web</option>
            <option value="spa">SPA</option>
            <option value="native">Native</option>
            <option value="m2m">M2M</option>
          </select>
        </FormField>
      </div>

      <FormField label="Description">
        <input type="text" value={formData.description} onChange={(e) => setFormData({ ...formData, description: e.target.value })} className="input-field" placeholder="Optional description" />
      </FormField>

      <div className="grid grid-cols-2 gap-3">
        <FormField label="Grant Types">
          <div className="flex flex-wrap gap-1">
            {grantTypes.map(g => (
              <Chip key={g.value} active={formData.grant_types.includes(g.value)} onClick={() => setFormData({ ...formData, grant_types: toggle(formData.grant_types, g.value) })}>{g.label}</Chip>
            ))}
          </div>
        </FormField>
        <FormField label="Scopes">
          <div className="flex flex-wrap gap-1">
            {scopes.map(s => (
              <Chip key={s} active={formData.allowed_scopes.includes(s)} onClick={() => setFormData({ ...formData, allowed_scopes: toggle(formData.allowed_scopes, s) })}>{s}</Chip>
            ))}
          </div>
        </FormField>
      </div>

      <div className="grid grid-cols-2 gap-3">
        <FormField label="Callback URLs">
          <UrlList urls={formData.callback_urls} onChange={(i, v) => updateUrl('callback_urls', i, v)} onAdd={() => addUrl('callback_urls')} onRemove={(i) => removeUrl('callback_urls', i)} />
        </FormField>
        <FormField label="Logout URLs">
          <UrlList urls={formData.logout_urls} onChange={(i, v) => updateUrl('logout_urls', i, v)} onAdd={() => addUrl('logout_urls')} onRemove={(i) => removeUrl('logout_urls', i)} />
        </FormField>
      </div>

      {isEdit && (
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={formData.is_enabled} onChange={(e) => setFormData({ ...formData, is_enabled: e.target.checked })} className="rounded border-slate-300" />
          <span className="text-slate-700">Enabled</span>
        </label>
      )}

      <div className="flex justify-end gap-2 pt-2">
        <button type="button" onClick={onCancel} className="btn-secondary">Cancel</button>
        <button type="submit" className="btn-primary">{isEdit ? 'Update' : 'Create'}</button>
      </div>
    </form>
  );
}

function UrlList({ urls, onChange, onAdd, onRemove }) {
  return (
    <div className="space-y-1">
      {urls.map((url, i) => (
        <div key={i} className="flex gap-1">
          <input
            type="url"
            value={url}
            onChange={(e) => onChange(i, e.target.value)}
            className="input-field font-mono text-xs flex-1"
            placeholder="https://..."
          />
          <button type="button" onClick={() => onRemove(i)} className="p-1.5 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded" title="Remove">
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      ))}
      <button type="button" onClick={onAdd} className="text-xs text-primary-600 hover:text-primary-700 flex items-center gap-1">
        <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
        </svg>
        Add URL
      </button>
    </div>
  );
}

function FormField({ label, required, children }) {
  return (
    <div>
      <label className="block text-xs font-medium text-slate-600 mb-1">
        {label}{required && <span className="text-red-500 ml-0.5">*</span>}
      </label>
      {children}
    </div>
  );
}

function Chip({ active, onClick, children }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`px-2 py-1 text-xs rounded border transition-colors ${active ? 'bg-primary-50 border-primary-300 text-primary-700' : 'bg-white border-slate-200 text-slate-500 hover:bg-slate-50'}`}
    >
      {children}
    </button>
  );
}
