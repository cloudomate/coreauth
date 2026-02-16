import { useState, useEffect, useCallback } from 'react';
import api from '../lib/api';

export default function Users() {
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showInviteForm, setShowInviteForm] = useState(false);
  const [selectedUser, setSelectedUser] = useState(null);
  const [error, setError] = useState(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [totalUsers, setTotalUsers] = useState(0);
  const usersPerPage = 20;

  const loadUsers = useCallback(async (page = 1, search = '') => {
    try {
      setLoading(true);
      setError(null);
      const meResponse = await api.get('/auth/me');
      const tenantId = meResponse.data.default_tenant_id;

      if (!tenantId) {
        setError('No tenant found. Please log in again.');
        return;
      }

      const params = new URLSearchParams({
        limit: usersPerPage.toString(),
        offset: ((page - 1) * usersPerPage).toString(),
      });

      if (search) {
        params.append('search', search);
      }

      const response = await api.get(`/tenants/${tenantId}/users?${params}`);
      setUsers(response.data.users || response.data || []);
      setTotalUsers(response.data.total || response.data.length || 0);
      setCurrentPage(page);
    } catch (error) {
      console.error('Failed to load users:', error);
      setError('Failed to load users');
    } finally {
      setLoading(false);
    }
  }, []);

  const handleInvite = async (formData) => {
    try {
      const meResponse = await api.get('/auth/me');
      const tenantId = meResponse.data.default_tenant_id;
      await api.post(`/tenants/${tenantId}/invitations`, formData);
      alert('Invitation sent successfully!');
      setShowInviteForm(false);
      loadUsers(currentPage, searchQuery);
    } catch (error) {
      console.error('Failed to send invitation:', error);
      alert('Failed to send invitation: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleToggleStatus = async (user) => {
    try {
      const meResponse = await api.get('/auth/me');
      const tenantId = meResponse.data.default_tenant_id;
      await api.patch(`/tenants/${tenantId}/users/${user.id}`, {
        is_active: !user.is_active,
      });
      loadUsers(currentPage, searchQuery);
    } catch (error) {
      console.error('Failed to update user:', error);
      alert('Failed to update user status');
    }
  };

  const handleDelete = async (userId) => {
    if (!confirm('Are you sure you want to delete this user?')) return;
    try {
      const meResponse = await api.get('/auth/me');
      const tenantId = meResponse.data.default_tenant_id;
      await api.delete(`/tenants/${tenantId}/users/${userId}`);
      loadUsers(currentPage, searchQuery);
    } catch (error) {
      console.error('Failed to delete user:', error);
      alert('Failed to delete user');
    }
  };

  useEffect(() => {
    loadUsers();
  }, [loadUsers]);

  useEffect(() => {
    const delayDebounceFn = setTimeout(() => {
      loadUsers(1, searchQuery);
    }, 300);
    return () => clearTimeout(delayDebounceFn);
  }, [searchQuery, loadUsers]);

  const totalPages = Math.ceil(totalUsers / usersPerPage);

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
          <h1 className="text-3xl font-bold text-slate-900">Users</h1>
          <p className="text-sm text-slate-500 mt-0.5">Manage users in your tenant</p>
        </div>
        {!showInviteForm && (
          <button onClick={() => setShowInviteForm(true)} className="btn-primary">
            Invite User
          </button>
        )}
      </div>

      {/* Invite Form - Inline */}
      {showInviteForm && (
        <div className="mb-4 p-4 bg-white border border-slate-200 rounded-md">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-medium text-slate-900">Invite New User</h2>
            <button onClick={() => setShowInviteForm(false)} className="text-slate-400 hover:text-slate-600">&times;</button>
          </div>
          <InviteForm onSubmit={handleInvite} onCancel={() => setShowInviteForm(false)} />
        </div>
      )}

      {/* Search */}
      <div className="mb-4">
        <input
          type="text"
          placeholder="Search by email or name..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="input-field max-w-xs"
        />
      </div>

      {/* Users Table */}
      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="text-slate-500">Loading...</div>
        </div>
      ) : users.length === 0 ? (
        <div className="text-center py-12 bg-white border border-slate-200 rounded-md">
          <p className="text-slate-500 text-sm mb-3">No users yet</p>
          <button onClick={() => setShowInviteForm(true)} className="btn-primary">Invite User</button>
        </div>
      ) : (
        <>
          <div className="bg-white border border-slate-200 rounded-md overflow-hidden">
            <table className="w-full">
              <thead className="bg-slate-50 border-b border-slate-200">
                <tr>
                  <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">User</th>
                  <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">Status</th>
                  <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">MFA</th>
                  <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">Created</th>
                  <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">Last Login</th>
                  <th className="text-right text-xs font-medium text-slate-500 px-4 py-2">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {users.map((user) => (
                  <tr key={user.id} className="hover:bg-slate-50">
                    <td className="px-4 py-3">
                      <div>
                        <div className="text-sm font-medium text-slate-900">
                          {user.metadata?.first_name || user.metadata?.last_name
                            ? `${user.metadata?.first_name || ''} ${user.metadata?.last_name || ''}`.trim()
                            : '—'}
                        </div>
                        <div className="text-xs text-slate-500">{user.email}</div>
                      </div>
                    </td>
                    <td className="px-4 py-3">
                      <span className={`px-1.5 py-0.5 text-xs rounded ${
                        user.is_active ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'
                      }`}>
                        {user.is_active ? 'Active' : 'Inactive'}
                      </span>
                      {user.email_verified && (
                        <span className="ml-1 px-1.5 py-0.5 text-xs rounded bg-blue-100 text-blue-700">Verified</span>
                      )}
                    </td>
                    <td className="px-4 py-3">
                      <span className={`px-1.5 py-0.5 text-xs rounded ${
                        user.mfa_enabled ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-500'
                      }`}>
                        {user.mfa_enabled ? 'Enabled' : 'Disabled'}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-xs text-slate-500">
                      {new Date(user.created_at).toLocaleDateString()}
                    </td>
                    <td className="px-4 py-3 text-xs text-slate-500">
                      {user.last_login_at ? new Date(user.last_login_at).toLocaleDateString() : '—'}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex justify-end gap-1">
                        <IconButton onClick={() => setSelectedUser(user)} title="View" icon="view" />
                        <IconButton onClick={() => handleToggleStatus(user)} title={user.is_active ? 'Deactivate' : 'Activate'} icon="toggle" />
                        <IconButton onClick={() => handleDelete(user.id)} title="Delete" icon="delete" danger />
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="flex items-center justify-between mt-4 text-sm">
              <div className="text-slate-500">
                {(currentPage - 1) * usersPerPage + 1} - {Math.min(currentPage * usersPerPage, totalUsers)} of {totalUsers}
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => loadUsers(currentPage - 1, searchQuery)}
                  disabled={currentPage === 1}
                  className="btn-secondary text-xs disabled:opacity-50"
                >
                  Previous
                </button>
                <button
                  onClick={() => loadUsers(currentPage + 1, searchQuery)}
                  disabled={currentPage === totalPages}
                  className="btn-secondary text-xs disabled:opacity-50"
                >
                  Next
                </button>
              </div>
            </div>
          )}
        </>
      )}

      {/* User Details Panel */}
      {selectedUser && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setSelectedUser(null)}>
          <div className="bg-white rounded-md p-4 max-w-md w-full mx-4" onClick={(e) => e.stopPropagation()}>
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-sm font-medium text-slate-900">User Details</h2>
              <button onClick={() => setSelectedUser(null)} className="text-slate-400 hover:text-slate-600">&times;</button>
            </div>
            <div className="space-y-2 text-sm">
              <div><span className="text-slate-500 w-24 inline-block">Email:</span> {selectedUser.email}</div>
              <div><span className="text-slate-500 w-24 inline-block">Name:</span> {selectedUser.metadata?.first_name} {selectedUser.metadata?.last_name}</div>
              <div><span className="text-slate-500 w-24 inline-block">Phone:</span> {selectedUser.phone || 'Not set'}</div>
              <div><span className="text-slate-500 w-24 inline-block">Verified:</span> {selectedUser.email_verified ? 'Yes' : 'No'}</div>
              <div><span className="text-slate-500 w-24 inline-block">MFA:</span> {selectedUser.mfa_enabled ? 'Enabled' : 'Disabled'}</div>
              <div><span className="text-slate-500 w-24 inline-block">Created:</span> {new Date(selectedUser.created_at).toLocaleString()}</div>
              <div><span className="text-slate-500 w-24 inline-block">Last Login:</span> {selectedUser.last_login_at ? new Date(selectedUser.last_login_at).toLocaleString() : 'Never'}</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function IconButton({ onClick, title, icon, danger }) {
  const icons = {
    view: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />,
    toggle: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />,
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

function InviteForm({ onSubmit, onCancel }) {
  const [formData, setFormData] = useState({
    email: '',
    role: 'member',
    first_name: '',
    last_name: '',
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(formData);
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="grid grid-cols-2 gap-3">
        <FormField label="First Name">
          <input
            type="text"
            value={formData.first_name}
            onChange={(e) => setFormData({ ...formData, first_name: e.target.value })}
            className="input-field"
          />
        </FormField>
        <FormField label="Last Name">
          <input
            type="text"
            value={formData.last_name}
            onChange={(e) => setFormData({ ...formData, last_name: e.target.value })}
            className="input-field"
          />
        </FormField>
      </div>

      <FormField label="Email" required>
        <input
          type="email"
          value={formData.email}
          onChange={(e) => setFormData({ ...formData, email: e.target.value })}
          className="input-field"
          required
        />
      </FormField>

      <FormField label="Role">
        <select
          value={formData.role}
          onChange={(e) => setFormData({ ...formData, role: e.target.value })}
          className="input-field"
        >
          <option value="member">Member</option>
          <option value="admin">Admin</option>
        </select>
      </FormField>

      <div className="flex justify-end gap-2 pt-2">
        <button type="button" onClick={onCancel} className="btn-secondary">Cancel</button>
        <button type="submit" className="btn-primary">Send Invitation</button>
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
