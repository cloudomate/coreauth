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
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [selectedAction, setSelectedAction] = useState(null);
  const [testResult, setTestResult] = useState(null);
  const orgId = 'demo-org-id'; // TODO: Get from context/state

  useEffect(() => {
    loadActions();
  }, []);

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
      setShowCreateModal(false);
    } catch (error) {
      console.error('Failed to create action:', error);
      alert('Failed to create action: ' + (error.response?.data?.message || error.message));
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
      <div className="min-h-screen bg-slate-50 flex items-center justify-center">
        <div className="text-slate-600">Loading...</div>
      </div>
    );
  }

  // Group actions by trigger type
  const actionsByTrigger = TRIGGER_TYPES.map(trigger => ({
    ...trigger,
    actions: actions.filter(a => a.trigger_type === trigger.value),
  }));

  return (
    <div className="min-h-screen bg-slate-50">
      <div className="max-w-7xl mx-auto px-6 py-8">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-slate-900">Actions & Hooks</h1>
            <p className="text-slate-600 mt-1">Extend authentication flows with custom JavaScript code</p>
          </div>
          <button
            onClick={() => setShowCreateModal(true)}
            className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
          >
            Create Action
          </button>
        </div>

        {/* Test Result Alert */}
        {testResult && (
          <div className={`mb-6 p-4 rounded-lg ${testResult.error ? 'bg-red-50 border border-red-200' : 'bg-green-50 border border-green-200'}`}>
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h3 className={`font-semibold mb-2 ${testResult.error ? 'text-red-900' : 'text-green-900'}`}>
                  Test Result: {testResult.action}
                </h3>
                <pre className={`text-sm p-3 rounded overflow-x-auto ${testResult.error ? 'bg-red-100' : 'bg-green-100'}`}>
                  {JSON.stringify(testResult.error || testResult.result, null, 2)}
                </pre>
              </div>
              <button
                onClick={() => setTestResult(null)}
                className={`ml-4 ${testResult.error ? 'text-red-700 hover:text-red-900' : 'text-green-700 hover:text-green-900'}`}
              >
                ×
              </button>
            </div>
          </div>
        )}

        {/* Actions by Trigger Type */}
        <div className="space-y-6">
          {actionsByTrigger.map(trigger => (
            <div key={trigger.value} className="card">
              <div className="mb-4">
                <h2 className="text-xl font-semibold text-slate-900">{trigger.label}</h2>
                <p className="text-sm text-slate-600">{trigger.description}</p>
              </div>

              {trigger.actions.length === 0 ? (
                <div className="text-center py-8 bg-slate-50 rounded-lg">
                  <p className="text-slate-600">No actions configured for this trigger</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {trigger.actions.map((action) => (
                    <div key={action.id} className="p-4 bg-slate-50 rounded-lg hover:bg-slate-100 transition-colors">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center space-x-3 mb-2">
                            <h3 className="font-semibold text-slate-900">{action.name}</h3>
                            <span className={`px-2 py-1 text-xs rounded-full ${
                              action.is_enabled
                                ? 'bg-green-100 text-green-700'
                                : 'bg-slate-200 text-slate-600'
                            }`}>
                              {action.is_enabled ? 'Enabled' : 'Disabled'}
                            </span>
                          </div>
                          {action.description && (
                            <p className="text-sm text-slate-600 mb-2">{action.description}</p>
                          )}
                          <div className="flex items-center space-x-4 text-xs text-slate-500">
                            <span>Order: {action.execution_order}</span>
                            <span>Timeout: {action.timeout_seconds}s</span>
                            <span>Executions: {action.total_executions}</span>
                            {action.total_failures > 0 && (
                              <span className="text-red-600">Failures: {action.total_failures}</span>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center space-x-2">
                          <button
                            onClick={() => handleTest(action)}
                            className="p-2 text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded-lg transition-colors"
                            title="Test Action"
                          >
                            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                          </button>
                          <button
                            onClick={() => handleToggleEnabled(action)}
                            className="p-2 text-slate-600 hover:text-slate-900 hover:bg-slate-200 rounded-lg transition-colors"
                            title={action.is_enabled ? 'Disable' : 'Enable'}
                          >
                            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
                            </svg>
                          </button>
                          <button
                            onClick={() => setSelectedAction(action)}
                            className="p-2 text-slate-600 hover:text-slate-900 hover:bg-slate-200 rounded-lg transition-colors"
                            title="Edit"
                          >
                            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                            </svg>
                          </button>
                          <button
                            onClick={() => handleDelete(action.id)}
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
            </div>
          ))}
        </div>

        {/* Create/Edit Modal */}
        {(showCreateModal || selectedAction) && (
          <ActionEditorModal
            action={selectedAction}
            onClose={() => {
              setShowCreateModal(false);
              setSelectedAction(null);
            }}
            onSave={async (data) => {
              if (selectedAction) {
                await actionApi.update(orgId, selectedAction.id, data);
              } else {
                await handleCreate(data);
              }
              await loadActions();
              setShowCreateModal(false);
              setSelectedAction(null);
            }}
          />
        )}
      </div>
    </div>
  );
}

function ActionEditorModal({ action, onClose, onSave }) {
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
    onSave(formData);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg w-full max-w-4xl max-h-[90vh] overflow-y-auto">
        <div className="sticky top-0 bg-white border-b border-slate-200 px-6 py-4">
          <h2 className="text-2xl font-bold">{action ? 'Edit Action' : 'Create Action'}</h2>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          <div className="grid md:grid-cols-2 gap-4">
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
              <label className="block text-sm font-medium text-slate-700 mb-1">Trigger Type</label>
              <select
                value={formData.trigger_type}
                onChange={(e) => setFormData({ ...formData, trigger_type: e.target.value })}
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                disabled={!!action}
              >
                {TRIGGER_TYPES.map(t => (
                  <option key={t.value} value={t.value}>{t.label}</option>
                ))}
              </select>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">Description</label>
            <input
              type="text"
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-700 mb-1">JavaScript Code</label>
            <div className="relative">
              <textarea
                value={formData.code}
                onChange={(e) => setFormData({ ...formData, code: e.target.value })}
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent font-mono text-sm"
                rows={15}
                required
                style={{ tabSize: 2 }}
              />
              <div className="absolute top-2 right-2 text-xs text-slate-500 bg-slate-100 px-2 py-1 rounded">
                JavaScript
              </div>
            </div>
            <p className="text-xs text-slate-600 mt-1">
              Available globals: <code className="bg-slate-100 px-1 rounded">context</code>, <code className="bg-slate-100 px-1 rounded">secrets</code>
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">Timeout (seconds)</label>
              <input
                type="number"
                min="1"
                max="30"
                value={formData.timeout_seconds}
                onChange={(e) => setFormData({ ...formData, timeout_seconds: parseInt(e.target.value) })}
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">Execution Order</label>
              <input
                type="number"
                min="0"
                value={formData.execution_order}
                onChange={(e) => setFormData({ ...formData, execution_order: parseInt(e.target.value) })}
                className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-700 mb-1">Status</label>
              <label className="flex items-center space-x-2 px-3 py-2 border border-slate-300 rounded-lg">
                <input
                  type="checkbox"
                  checked={formData.is_enabled}
                  onChange={(e) => setFormData({ ...formData, is_enabled: e.target.checked })}
                  className="rounded border-slate-300"
                />
                <span className="text-sm">Enabled</span>
              </label>
            </div>
          </div>

          <div className="flex justify-end space-x-3 pt-4 border-t">
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
              {action ? 'Update Action' : 'Create Action'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
