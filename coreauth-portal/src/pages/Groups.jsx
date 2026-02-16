import { useState, useEffect, useCallback } from 'react';
import api from '../lib/api';

export default function Groups() {
  const [groups, setGroups] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [selectedGroup, setSelectedGroup] = useState(null);
  const [error, setError] = useState(null);
  const [tenantId, setTenantId] = useState(null);

  const loadGroups = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const meResponse = await api.get('/auth/me');
      const tid = meResponse.data.default_tenant_id;
      setTenantId(tid);

      if (!tid) {
        setError('No tenant found. Please log in again.');
        return;
      }

      const response = await api.get(`/tenants/${tid}/groups`);
      setGroups(response.data.groups || []);
    } catch (error) {
      console.error('Failed to load groups:', error);
      setError('Failed to load groups: ' + (error.response?.data?.message || error.message));
    } finally {
      setLoading(false);
    }
  }, []);

  const handleCreate = async (formData) => {
    try {
      await api.post(`/tenants/${tenantId}/groups`, formData);
      setShowCreateForm(false);
      loadGroups();
    } catch (error) {
      console.error('Failed to create group:', error);
      alert('Failed to create group: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleUpdate = async (groupId, formData) => {
    try {
      await api.put(`/tenants/${tenantId}/groups/${groupId}`, formData);
      setSelectedGroup(null);
      loadGroups();
    } catch (error) {
      console.error('Failed to update group:', error);
      alert('Failed to update group: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleDelete = async (groupId) => {
    if (!confirm('Are you sure you want to delete this group?')) return;
    try {
      await api.delete(`/tenants/${tenantId}/groups/${groupId}`);
      loadGroups();
    } catch (error) {
      console.error('Failed to delete group:', error);
      alert('Failed to delete group: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleToggleActive = async (group) => {
    try {
      await api.put(`/tenants/${tenantId}/groups/${group.id}`, {
        is_active: !group.is_active,
      });
      loadGroups();
    } catch (error) {
      console.error('Failed to update group:', error);
      alert('Failed to update group status');
    }
  };

  useEffect(() => {
    loadGroups();
  }, [loadGroups]);

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
          <h1 className="text-3xl font-bold text-slate-900">Groups</h1>
          <p className="text-sm text-slate-500 mt-0.5">Organize users into groups and assign collective permissions</p>
        </div>
        {!showCreateForm && (
          <button onClick={() => setShowCreateForm(true)} className="btn-primary">
            Create Group
          </button>
        )}
      </div>

      {/* Create Form */}
      {showCreateForm && (
        <div className="mb-4 p-4 bg-white border border-slate-200 rounded-md">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-medium text-slate-900">Create New Group</h2>
            <button onClick={() => setShowCreateForm(false)} className="text-slate-400 hover:text-slate-600">&times;</button>
          </div>
          <GroupForm onSubmit={handleCreate} onCancel={() => setShowCreateForm(false)} />
        </div>
      )}

      {/* Groups Table */}
      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="text-slate-500">Loading...</div>
        </div>
      ) : groups.length === 0 ? (
        <div className="text-center py-12 bg-white border border-slate-200 rounded-md">
          <div className="mb-4">
            <svg className="w-12 h-12 mx-auto text-slate-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
            </svg>
          </div>
          <p className="text-slate-500 text-sm mb-3">No groups yet</p>
          <p className="text-slate-400 text-xs mb-4">Create groups to organize users and manage permissions</p>
          <button onClick={() => setShowCreateForm(true)} className="btn-primary">Create Group</button>
        </div>
      ) : (
        <div className="bg-white border border-slate-200 rounded-md overflow-hidden">
          <table className="w-full">
            <thead className="bg-slate-50 border-b border-slate-200">
              <tr>
                <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">Group</th>
                <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">Members</th>
                <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">Status</th>
                <th className="text-left text-xs font-medium text-slate-500 px-4 py-2">Created</th>
                <th className="text-right text-xs font-medium text-slate-500 px-4 py-2">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {groups.map((group) => (
                <tr key={group.id} className="hover:bg-slate-50">
                  <td className="px-4 py-3">
                    <div>
                      <div className="text-sm font-medium text-slate-900">{group.name}</div>
                      <div className="text-xs text-slate-500">{group.slug}</div>
                      {group.description && (
                        <div className="text-xs text-slate-400 mt-0.5">{group.description}</div>
                      )}
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <span className="px-2 py-1 text-xs font-medium bg-slate-100 text-slate-600 rounded">
                      {group.member_count} {group.member_count === 1 ? 'member' : 'members'}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <span className={`px-1.5 py-0.5 text-xs rounded ${
                      group.is_active ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-500'
                    }`}>
                      {group.is_active ? 'Active' : 'Inactive'}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-xs text-slate-500">
                    {new Date(group.created_at).toLocaleDateString()}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex justify-end gap-1">
                      <IconButton onClick={() => setSelectedGroup(group)} title="Manage Members" icon="users" />
                      <IconButton onClick={() => handleToggleActive(group)} title={group.is_active ? 'Deactivate' : 'Activate'} icon="toggle" />
                      <IconButton onClick={() => handleDelete(group.id)} title="Delete" icon="delete" danger />
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Group Members Panel */}
      {selectedGroup && (
        <GroupMembersPanel
          group={selectedGroup}
          tenantId={tenantId}
          onClose={() => setSelectedGroup(null)}
          onUpdate={loadGroups}
        />
      )}
    </div>
  );
}

function GroupForm({ onSubmit, onCancel, initialData }) {
  const [formData, setFormData] = useState({
    name: initialData?.name || '',
    slug: initialData?.slug || '',
    description: initialData?.description || '',
  });

  const handleNameChange = (e) => {
    const name = e.target.value;
    const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
    setFormData({ ...formData, name, slug: initialData ? formData.slug : slug });
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(formData);
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <FormField label="Group Name" required>
        <input
          type="text"
          value={formData.name}
          onChange={handleNameChange}
          placeholder="e.g., Engineering Team"
          className="input-field"
          required
        />
      </FormField>

      <FormField label="Slug" required>
        <input
          type="text"
          value={formData.slug}
          onChange={(e) => setFormData({ ...formData, slug: e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, '') })}
          placeholder="e.g., engineering-team"
          className="input-field"
          required
        />
        <p className="text-xs text-slate-400 mt-1">Used in URLs and API calls</p>
      </FormField>

      <FormField label="Description">
        <textarea
          value={formData.description}
          onChange={(e) => setFormData({ ...formData, description: e.target.value })}
          placeholder="Optional description for this group"
          className="input-field"
          rows={2}
        />
      </FormField>

      <div className="flex justify-end gap-2 pt-2">
        <button type="button" onClick={onCancel} className="btn-secondary">Cancel</button>
        <button type="submit" className="btn-primary">{initialData ? 'Update' : 'Create'} Group</button>
      </div>
    </form>
  );
}

function GroupMembersPanel({ group, tenantId, onClose, onUpdate }) {
  const [members, setMembers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showAddMember, setShowAddMember] = useState(false);
  const [allUsers, setAllUsers] = useState([]);
  const [selectedUserId, setSelectedUserId] = useState('');

  const loadMembers = useCallback(async () => {
    try {
      setLoading(true);
      const response = await api.get(`/tenants/${tenantId}/groups/${group.id}/members`);
      setMembers(response.data || []);
    } catch (error) {
      console.error('Failed to load members:', error);
    } finally {
      setLoading(false);
    }
  }, [tenantId, group.id]);

  const loadUsers = useCallback(async () => {
    try {
      const response = await api.get(`/tenants/${tenantId}/users?limit=100`);
      setAllUsers(response.data.users || response.data || []);
    } catch (error) {
      console.error('Failed to load users:', error);
    }
  }, [tenantId]);

  const handleAddMember = async () => {
    if (!selectedUserId) return;
    try {
      await api.post(`/tenants/${tenantId}/groups/${group.id}/members`, {
        user_id: selectedUserId,
      });
      setSelectedUserId('');
      setShowAddMember(false);
      loadMembers();
      onUpdate();
    } catch (error) {
      console.error('Failed to add member:', error);
      alert('Failed to add member: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleRemoveMember = async (userId) => {
    if (!confirm('Remove this member from the group?')) return;
    try {
      await api.delete(`/tenants/${tenantId}/groups/${group.id}/members/${userId}`);
      loadMembers();
      onUpdate();
    } catch (error) {
      console.error('Failed to remove member:', error);
      alert('Failed to remove member');
    }
  };

  useEffect(() => {
    loadMembers();
    loadUsers();
  }, [loadMembers, loadUsers]);

  const memberUserIds = new Set(members.map(m => m.user_id));
  const availableUsers = allUsers.filter(u => !memberUserIds.has(u.id));

  return (
    <div className="fixed inset-0 bg-black/50 flex items-start justify-center z-50 pt-20" onClick={onClose}>
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[80vh] overflow-hidden flex flex-col" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className="px-6 py-4 border-b border-slate-200 flex items-center justify-between">
          <div>
            <h2 className="text-lg font-semibold text-slate-900">{group.name}</h2>
            <p className="text-sm text-slate-500">Manage group members</p>
          </div>
          <button onClick={onClose} className="text-slate-400 hover:text-slate-600 text-2xl">&times;</button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {/* Add Member */}
          {showAddMember ? (
            <div className="mb-4 p-4 bg-slate-50 rounded-md">
              <div className="flex items-center gap-3">
                <select
                  value={selectedUserId}
                  onChange={(e) => setSelectedUserId(e.target.value)}
                  className="input-field flex-1"
                >
                  <option value="">Select a user to add...</option>
                  {availableUsers.map(user => (
                    <option key={user.id} value={user.id}>
                      {user.email} {user.metadata?.first_name ? `(${user.metadata.first_name} ${user.metadata.last_name || ''})` : ''}
                    </option>
                  ))}
                </select>
                <button onClick={handleAddMember} disabled={!selectedUserId} className="btn-primary disabled:opacity-50">
                  Add
                </button>
                <button onClick={() => setShowAddMember(false)} className="btn-secondary">
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            <div className="mb-4">
              <button onClick={() => setShowAddMember(true)} className="btn-secondary">
                + Add Member
              </button>
            </div>
          )}

          {/* Members List */}
          {loading ? (
            <div className="text-center py-8 text-slate-500">Loading...</div>
          ) : members.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-slate-500 text-sm">No members in this group</p>
              <p className="text-slate-400 text-xs mt-1">Add users to this group to manage their permissions collectively</p>
            </div>
          ) : (
            <div className="space-y-2">
              {members.map((member) => (
                <div key={member.id} className="flex items-center justify-between p-3 bg-slate-50 rounded-md">
                  <div className="flex items-center gap-3">
                    <div className="w-8 h-8 bg-primary-100 text-primary-600 rounded-full flex items-center justify-center text-sm font-medium">
                      {member.email?.charAt(0).toUpperCase()}
                    </div>
                    <div>
                      <div className="text-sm font-medium text-slate-900">
                        {member.full_name || member.email}
                      </div>
                      {member.full_name && (
                        <div className="text-xs text-slate-500">{member.email}</div>
                      )}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 text-xs bg-slate-200 text-slate-600 rounded">
                      {member.role}
                    </span>
                    <button
                      onClick={() => handleRemoveMember(member.user_id)}
                      className="p-1 text-slate-400 hover:text-red-500"
                      title="Remove member"
                    >
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-slate-200 bg-slate-50">
          <div className="flex items-center justify-between">
            <span className="text-sm text-slate-500">{members.length} member{members.length !== 1 ? 's' : ''}</span>
            <button onClick={onClose} className="btn-secondary">Close</button>
          </div>
        </div>
      </div>
    </div>
  );
}

function IconButton({ onClick, title, icon, danger }) {
  const icons = {
    users: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />,
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
