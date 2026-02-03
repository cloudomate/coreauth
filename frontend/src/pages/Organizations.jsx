import { useState, useEffect } from 'react';
import { organizationApi } from '../api/client';

export default function Organizations() {
  const [organizations, setOrganizations] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreateModal, setShowCreateModal] = useState(false);

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
      setShowCreateModal(false);
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
            <h1 className="text-3xl font-bold text-slate-900">Customer Organizations</h1>
            <p className="text-slate-600 mt-1">Manage organizations for your customers (2-level hierarchy supported)</p>
          </div>
          <button
            onClick={() => setShowCreateModal(true)}
            className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
          >
            Create Organization
          </button>
        </div>

        {/* Organizations Grid */}
        {organizations.length === 0 ? (
          <div className="card text-center py-12">
            <svg className="w-16 h-16 text-slate-300 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
            </svg>
            <h3 className="text-lg font-semibold text-slate-900 mb-2">No customer organizations yet</h3>
            <p className="text-slate-600 mb-4">Create organizations for your customers to start managing their authentication</p>
            <button
              onClick={() => setShowCreateModal(true)}
              className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
            >
              Create Organization
            </button>
          </div>
        ) : (
          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {organizations.map((org) => (
              <div key={org.id} className="card hover:shadow-lg transition-shadow">
                <div className="flex items-start justify-between mb-4">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2 mb-2">
                      <h3 className="text-xl font-semibold text-slate-900">{org.name}</h3>
                      <span className="px-2 py-1 text-xs rounded-full bg-primary-100 text-primary-700">
                        Level {org.hierarchy_level || 0}
                      </span>
                    </div>
                    <p className="text-sm text-slate-600 mb-2">
                      <span className="font-mono bg-slate-100 px-2 py-1 rounded">{org.slug}</span>
                    </p>
                    {org.parent_organization_id && (
                      <p className="text-xs text-slate-500">Has parent organization</p>
                    )}
                  </div>
                  <button
                    onClick={() => handleDelete(org.id)}
                    className="p-2 text-red-600 hover:text-red-700 hover:bg-red-50 rounded-lg transition-colors"
                    title="Delete"
                  >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>

                <div className="grid grid-cols-2 gap-4 pt-4 border-t border-slate-200">
                  <div>
                    <div className="text-2xl font-bold text-slate-900">0</div>
                    <div className="text-xs text-slate-600">Users</div>
                  </div>
                  <div>
                    <div className="text-2xl font-bold text-slate-900">0</div>
                    <div className="text-xs text-slate-600">Applications</div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Create Modal */}
        {showCreateModal && (
          <CreateOrganizationModal
            organizations={organizations}
            onClose={() => setShowCreateModal(false)}
            onCreate={handleCreate}
          />
        )}
      </div>
    </div>
  );
}

function CreateOrganizationModal({ organizations, onClose, onCreate }) {
  const [formData, setFormData] = useState({
    name: '',
    slug: '',
    parent_organization_id: '',
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    const data = {
      ...formData,
      parent_organization_id: formData.parent_organization_id || undefined,
    };
    onCreate(data);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg p-6 max-w-lg w-full mx-4">
        <h2 className="text-2xl font-bold mb-4">Create Organization</h2>
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
              onChange={(e) => setFormData({ ...formData, slug: e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '-') })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent font-mono"
              placeholder="my-organization"
              required
            />
            <p className="text-xs text-slate-500 mt-1">Used in URLs and API calls</p>
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Parent Organization (Optional)</label>
            <select
              value={formData.parent_organization_id}
              onChange={(e) => setFormData({ ...formData, parent_organization_id: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            >
              <option value="">None (Root Organization)</option>
              {organizations.filter(org => org.hierarchy_level < 1).map(org => (
                <option key={org.id} value={org.id}>{org.name}</option>
              ))}
            </select>
            <p className="text-xs text-slate-500 mt-1">Max 2 levels supported</p>
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
              Create Organization
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
