import { useState, useEffect } from 'react';
import { actionApi } from '../api/client';

const TRIGGER_TYPES = [
  { value: 'pre_login', label: 'Pre-Login', description: 'Execute before user login' },
  { value: 'post_login', label: 'Post-Login', description: 'Execute after successful login' },
  { value: 'pre_registration', label: 'Pre-Registration', description: 'Execute before user registration' },
  { value: 'post_registration', label: 'Post-Registration', description: 'Execute after user registration' },
  { value: 'pre_token_issue', label: 'Pre-Token Issue', description: 'Execute before token generation' },
  { value: 'post_token_issue', label: 'Post-Token Issue', description: 'Execute after token generation' },
];

const DEFAULT_CODE = `// Action code - return a value to pass data forward
// Available globals: context, secrets

// Example: Add custom claims
return {
  success: true,
  custom_claims: {
    department: context.user.metadata?.department,
    role: "admin"
  }
};`;

export default function Actions() {
  const [actions, setActions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [editingAction, setEditingAction] = useState(null);
  const [testResult, setTestResult] = useState(null);
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
      loadActions();
    } else {
      setError('No organization found. Please log in again.');
      setLoading(false);
    }
  }, [orgId]);

  const loadActions = async () => {
    try {
      const { data } = await actionApi.list(orgId);
      setActions(data.actions || []);
    } catch (error) {
      console.error('Failed to load actions:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async (formData) => {
    try {
      await actionApi.create(orgId, formData);
      await loadActions();
      setShowCreateForm(false);
    } catch (error) {
      console.error('Failed to create action:', error);
      alert('Failed to create action: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleUpdate = async (actionId, formData) => {
    try {
      await actionApi.update(orgId, actionId, formData);
      await loadActions();
      setEditingAction(null);
    } catch (error) {
      console.error('Failed to update action:', error);
      alert('Failed to update action: ' + (error.response?.data?.message || error.message));
    }
  };

  const handleDelete = async (actionId) => {
    if (!confirm('Are you sure you want to delete this action?')) return;
    try {
      await actionApi.delete(orgId, actionId);
      await loadActions();
    } catch (error) {
      console.error('Failed to delete action:', error);
      alert('Failed to delete action');
    }
  };

  const handleToggleEnabled = async (action) => {
    try {
      await actionApi.update(orgId, action.id, { is_enabled: !action.is_enabled });
      await loadActions();
    } catch (error) {
      console.error('Failed to update action:', error);
    }
  };

  const handleTest = async (action) => {
    const testContext = {
      user: { id: 'test-user-123', email: 'test@example.com', metadata: {} },
      organization: { id: orgId, name: 'Test Org' },
      event: 'test',
      metadata: {},
    };
    try {
      const { data } = await actionApi.test(orgId, action.id, testContext);
      setTestResult({ action: action.name, result: data });
    } catch (error) {
      console.error('Failed to test action:', error);
      setTestResult({ action: action.name, error: error.response?.data || error.message });
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

  const actionsByTrigger = TRIGGER_TYPES.map(trigger => ({
    ...trigger,
    actions: actions.filter(a => a.trigger_type === trigger.value),
  }));

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold text-slate-900">Actions & Hooks</h1>
          <p className="text-sm text-slate-500 mt-0.5">Extend authentication flows with custom JavaScript</p>
        </div>
        {!showCreateForm && !editingAction && (
          <button onClick={() => setShowCreateForm(true)} className="btn-primary">
            Create Action
          </button>
        )}
      </div>

      {/* Test Result */}
      {testResult && (
        <div className={`mb-4 p-3 rounded-md ${testResult.error ? 'bg-red-50 border border-red-200' : 'bg-green-50 border border-green-200'}`}>
          <div className="flex items-start justify-between gap-3">
            <div className="flex-1 min-w-0">
              <p className={`text-sm font-medium ${testResult.error ? 'text-red-800' : 'text-green-800'}`}>
                Test: {testResult.action}
              </p>
              <pre className={`text-xs mt-2 p-2 rounded overflow-x-auto ${testResult.error ? 'bg-red-100' : 'bg-green-100'}`}>
                {JSON.stringify(testResult.error || testResult.result, null, 2)}
              </pre>
            </div>
            <button onClick={() => setTestResult(null)} className="text-slate-400 hover:text-slate-600">&times;</button>
          </div>
        </div>
      )}

      {/* Create Form - Inline */}
      {showCreateForm && (
        <div className="mb-4 p-4 bg-white border border-slate-200 rounded-md">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-sm font-medium text-slate-900">New Action</h2>
            <button onClick={() => setShowCreateForm(false)} className="text-slate-400 hover:text-slate-600">&times;</button>
          </div>
          <ActionForm onSubmit={handleCreate} onCancel={() => setShowCreateForm(false)} />
        </div>
      )}

      {/* Actions by Trigger Type */}
      <div className="space-y-4">
        {actionsByTrigger.map(trigger => (
          <div key={trigger.value} className="bg-white border border-slate-200 rounded-md">
            <div className="px-4 py-3 border-b border-slate-100">
              <h2 className="text-sm font-medium text-slate-900">{trigger.label}</h2>
              <p className="text-xs text-slate-500">{trigger.description}</p>
            </div>

            {trigger.actions.length === 0 ? (
              <div className="p-4 text-center">
                <p className="text-slate-400 text-xs">No actions configured</p>
              </div>
            ) : (
              <div className="divide-y divide-slate-100">
                {trigger.actions.map((action) => (
                  <div key={action.id}>
                    {editingAction?.id === action.id ? (
                      <div className="p-4">
                        <div className="flex items-center justify-between mb-4">
                          <h3 className="text-sm font-medium text-slate-900">Edit Action</h3>
                          <button onClick={() => setEditingAction(null)} className="text-slate-400 hover:text-slate-600">&times;</button>
                        </div>
                        <ActionForm
                          action={action}
                          onSubmit={(data) => handleUpdate(action.id, data)}
                          onCancel={() => setEditingAction(null)}
                          isEdit
                        />
                      </div>
                    ) : (
                      <div className="p-4 flex items-start justify-between gap-4">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 flex-wrap">
                            <h3 className="text-sm font-medium text-slate-900">{action.name}</h3>
                            <span className={`px-1.5 py-0.5 text-xs rounded ${
                              action.is_enabled ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-500'
                            }`}>
                              {action.is_enabled ? 'Enabled' : 'Disabled'}
                            </span>
                          </div>
                          {action.description && (
                            <p className="text-xs text-slate-500 mt-1">{action.description}</p>
                          )}
                          <div className="flex items-center gap-3 mt-2 text-xs text-slate-400">
                            <span>Order: {action.execution_order}</span>
                            <span>Timeout: {action.timeout_seconds}s</span>
                            <span>Runs: {action.total_executions}</span>
                            {action.total_failures > 0 && (
                              <span className="text-red-500">Failures: {action.total_failures}</span>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-1">
                          <IconButton onClick={() => handleTest(action)} title="Test" icon="play" />
                          <IconButton onClick={() => handleToggleEnabled(action)} title={action.is_enabled ? 'Disable' : 'Enable'} icon="toggle" />
                          <IconButton onClick={() => setEditingAction(action)} title="Edit" icon="edit" />
                          <IconButton onClick={() => handleDelete(action.id)} title="Delete" icon="delete" danger />
                        </div>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

function IconButton({ onClick, title, icon, danger }) {
  const icons = {
    play: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />,
    toggle: <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />,
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

function ActionForm({ action, onSubmit, onCancel, isEdit }) {
  const [formData, setFormData] = useState({
    name: action?.name || '',
    description: action?.description || '',
    trigger_type: action?.trigger_type || 'post_login',
    code: action?.code || DEFAULT_CODE,
    timeout_seconds: action?.timeout_seconds || 10,
    execution_order: action?.execution_order || 0,
    is_enabled: action?.is_enabled ?? true,
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    onSubmit(formData);
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
        <FormField label="Trigger Type">
          <select
            value={formData.trigger_type}
            onChange={(e) => setFormData({ ...formData, trigger_type: e.target.value })}
            className="input-field"
            disabled={isEdit}
          >
            {TRIGGER_TYPES.map(t => (
              <option key={t.value} value={t.value}>{t.label}</option>
            ))}
          </select>
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

      <FormField label="JavaScript Code" required>
        <textarea
          value={formData.code}
          onChange={(e) => setFormData({ ...formData, code: e.target.value })}
          className="input-field font-mono text-xs"
          rows={8}
          required
        />
        <p className="text-xs text-slate-400 mt-1">
          Available: <code className="bg-slate-100 px-1 rounded">context</code>, <code className="bg-slate-100 px-1 rounded">secrets</code>
        </p>
      </FormField>

      <div className="grid grid-cols-3 gap-3">
        <FormField label="Timeout (sec)">
          <input
            type="number"
            min="1"
            max="30"
            value={formData.timeout_seconds}
            onChange={(e) => setFormData({ ...formData, timeout_seconds: parseInt(e.target.value) })}
            className="input-field"
          />
        </FormField>
        <FormField label="Order">
          <input
            type="number"
            min="0"
            value={formData.execution_order}
            onChange={(e) => setFormData({ ...formData, execution_order: parseInt(e.target.value) })}
            className="input-field"
          />
        </FormField>
        <FormField label="Status">
          <label className="flex items-center gap-2 h-[34px]">
            <input
              type="checkbox"
              checked={formData.is_enabled}
              onChange={(e) => setFormData({ ...formData, is_enabled: e.target.checked })}
              className="rounded border-slate-300"
            />
            <span className="text-sm text-slate-700">Enabled</span>
          </label>
        </FormField>
      </div>

      <div className="flex justify-end gap-2 pt-2">
        <button type="button" onClick={onCancel} className="btn-secondary">Cancel</button>
        <button type="submit" className="btn-primary">{isEdit ? 'Update' : 'Create'}</button>
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
