import { useState, useEffect } from 'react';
import { organizationApi } from '../api/client';

export default function Organizations() {
  const [organizations, setOrganizations] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);

  useEffect(() => {
    loadOrganizations();
  }, []);

  const loadOrganizations = async () => {
    try {
      const { data } = await organizationApi.list();
      setOrganizations(data.organizations || data || []);
    } catch (error) {
      console.error('Failed to load organizations:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async (formData) => {
    try {
      await organizationApi.create(formData);
      await loadOrganizations();
      setShowCreateForm(false);
    } catch (error) {
      console.error('Failed to create organization:', error);
      alert('Failed to create organization: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleDelete = async (orgId) => {
    if (!confirm('Are you sure you want to delete this organization?')) return;
    try {
      await organizationApi.delete(orgId);
      await loadOrganizations();
    } catch (error) {
      console.error('Failed to delete organization:', error);
      alert('Failed to delete organization');
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-500">Loading...</div>
      </div>
    );
  }

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold text-slate-900">Organizations</h1>
          <p className="text-sm text-slate-500 mt-0.5">Manage sub-organizations within your tenant</p>
        </div>
        {!showCreateForm && (
          <button onClick={() => setShowCreateForm(true)} className="btn-primary">
            Create Organization
          </button>
        )}
      </div>

      {/* Create Form - Inline */}
      {showCreateForm && (
        <div className="mb-4 p-4 bg-white border border-slate-200 rounded-md">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-medium text-slate-900">New Organization</h2>
            <button onClick={() => setShowCreateForm(false)} className="text-slate-400 hover:text-slate-600">&times;</button>
          </div>
          <OrganizationForm
            organizations={organizations}
            onSubmit={handleCreate}
            onCancel={() => setShowCreateForm(false)}
          />
        </div>
      )}

      {/* Organizations List */}
      {organizations.length === 0 && !showCreateForm ? (
        <div className="text-center py-12 bg-white border border-slate-200 rounded-md">
          <p className="text-slate-500 text-sm mb-3">No organizations yet</p>
          <button onClick={() => setShowCreateForm(true)} className="btn-primary">Create Organization</button>
        </div>
      ) : (
        <div className="space-y-3">
          {organizations.map((org) => (
            <div key={org.id} className="bg-white border border-slate-200 rounded-md p-4 flex items-center justify-between">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <h3 className="text-sm font-medium text-slate-900">{org.name}</h3>
                  <code className="text-xs bg-slate-100 px-1.5 py-0.5 rounded text-slate-600">{org.slug}</code>
                  {org.parent_tenant_id && (
                    <span className="px-1.5 py-0.5 text-xs rounded bg-slate-100 text-slate-500">Sub-org</span>
                  )}
                </div>
                {org.description && (
                  <p className="text-xs text-slate-500 mt-1">{org.description}</p>
                )}
              </div>
              <IconButton onClick={() => handleDelete(org.id)} title="Delete" icon="delete" danger />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function IconButton({ onClick, title, icon, danger }) {
  const icons = {
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

function OrganizationForm({ organizations, onSubmit, onCancel }) {
  const [formData, setFormData] = useState({
    name: '',
    slug: '',
    description: '',
    parent_tenant_id: '',
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit({
      ...formData,
      parent_tenant_id: formData.parent_tenant_id || undefined,
    });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="grid grid-cols-2 gap-3">
        <FormField label="Name" required>
          <input
            type="text"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            className="input-field"
            required
          />
        </FormField>
        <FormField label="Slug" required>
          <input
            type="text"
            value={formData.slug}
            onChange={(e) => setFormData({ ...formData, slug: e.target.value })}
            className="input-field"
            required
          />
        </FormField>
      </div>

      <FormField label="Description">
        <input
          type="text"
          value={formData.description}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          className="input-field"
          placeholder="Optional description"
        />
      </FormField>

      <FormField label="Parent Organization">
        <select
          value={formData.parent_tenant_id}
          onChange={(e) => setFormData({ ...formData, parent_tenant_id: e.target.value })}
          className="input-field"
        >
          <option value="">None (Top-level)</option>
          {organizations.map((org) => (
            <option key={org.id} value={org.id}>{org.name}</option>
          ))}
        </select>
      </FormField>

      <div className="flex justify-end gap-2 pt-2">
        <button type="button" onClick={onCancel} className="btn-secondary">Cancel</button>
        <button type="submit" className="btn-primary">Create</button>
      </div>
    </form>
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
