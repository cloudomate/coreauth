import { useState, useEffect } from 'react';
import api from '../lib/api';

export default function FgaStores() {
  const [stores, setStores] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  // Modal states
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showModelModal, setShowModelModal] = useState(false);
  const [showApiKeyModal, setShowApiKeyModal] = useState(false);
  const [showTuplesModal, setShowTuplesModal] = useState(false);
  const [selectedStore, setSelectedStore] = useState(null);

  // Form states
  const [createForm, setCreateForm] = useState({ name: '', description: '' });
  const [modelForm, setModelForm] = useState({ schema_dsl: '' });
  const [apiKeyForm, setApiKeyForm] = useState({ name: '', permissions: ['read', 'write', 'check'] });
  const [newApiKey, setNewApiKey] = useState(null);

  // Store details
  const [currentModel, setCurrentModel] = useState(null);
  const [apiKeys, setApiKeys] = useState([]);
  const [tuples, setTuples] = useState([]);

  useEffect(() => {
    loadStores();
  }, []);

  const loadStores = async () => {
    setLoading(true);
    try {
      const response = await api.get('/fga/stores');
      setStores(response.data);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to load FGA stores');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateStore = async (e) => {
    e.preventDefault();
    setError('');
    try {
      const response = await api.post('/fga/stores', createForm);
      setStores([response.data, ...stores]);
      setShowCreateModal(false);
      setCreateForm({ name: '', description: '' });
      setSuccess('Store created successfully');
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to create store');
    }
  };

  const handleDeleteStore = async (storeId) => {
    if (!window.confirm('Are you sure you want to delete this store? This action cannot be undone.')) {
      return;
    }
    try {
      await api.delete(`/fga/stores/${storeId}?hard_delete=false`);
      setStores(stores.filter(s => s.id !== storeId));
      setSuccess('Store deleted successfully');
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to delete store');
    }
  };

  const openModelModal = async (store) => {
    setSelectedStore(store);
    setError('');
    try {
      const response = await api.get(`/fga/stores/${store.id}/models/current`);
      setCurrentModel(response.data);
      setModelForm({ schema_dsl: response.data.schema_dsl || '' });
    } catch (err) {
      // No model yet
      setCurrentModel(null);
      setModelForm({ schema_dsl: getDefaultModel() });
    }
    setShowModelModal(true);
  };

  const handleWriteModel = async (e) => {
    e.preventDefault();
    setError('');
    try {
      // Parse the DSL into schema JSON
      const schema = parseDslToSchema(modelForm.schema_dsl);
      const response = await api.post(`/fga/stores/${selectedStore.id}/models`, {
        schema,
        created_by: 'admin',
      });
      setCurrentModel(response.data);
      setSuccess(`Model v${response.data.version} saved successfully`);
      setTimeout(() => setSuccess(''), 3000);

      // Update store's model version in the list
      setStores(stores.map(s =>
        s.id === selectedStore.id
          ? { ...s, current_model_version: response.data.version }
          : s
      ));
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to save model');
    }
  };

  const openApiKeyModal = async (store) => {
    setSelectedStore(store);
    setNewApiKey(null);
    setError('');
    try {
      const response = await api.get(`/fga/stores/${store.id}/api-keys`);
      setApiKeys(response.data);
    } catch (err) {
      setApiKeys([]);
    }
    setShowApiKeyModal(true);
  };

  const handleCreateApiKey = async (e) => {
    e.preventDefault();
    setError('');
    try {
      const response = await api.post(`/fga/stores/${selectedStore.id}/api-keys`, apiKeyForm);
      setNewApiKey(response.data);
      setApiKeys([response.data, ...apiKeys]);
      setApiKeyForm({ name: '', permissions: ['read', 'write', 'check'] });
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to create API key');
    }
  };

  const handleRevokeApiKey = async (keyId) => {
    if (!window.confirm('Are you sure you want to revoke this API key?')) {
      return;
    }
    try {
      await api.delete(`/fga/stores/${selectedStore.id}/api-keys/${keyId}`);
      setApiKeys(apiKeys.filter(k => k.id !== keyId));
      setSuccess('API key revoked');
      setTimeout(() => setSuccess(''), 3000);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to revoke API key');
    }
  };

  const openTuplesModal = async (store) => {
    setSelectedStore(store);
    setError('');
    try {
      const response = await api.get(`/fga/stores/${store.id}/tuples`);
      setTuples(response.data);
    } catch (err) {
      setTuples([]);
    }
    setShowTuplesModal(true);
  };

  // Helper: Get default authorization model template
  const getDefaultModel = () => {
    return `model
  schema 1.1

type user

type document
  relations
    define owner: [user]
    define editor: [user] or owner
    define viewer: [user] or editor

type folder
  relations
    define owner: [user]
    define viewer: [user] or owner`;
  };

  // Helper: Parse DSL to schema JSON (simplified parser)
  const parseDslToSchema = (dsl) => {
    const lines = dsl.split('\n').map(l => l.trim()).filter(l => l && !l.startsWith('#'));
    const schema = {
      schema_version: '1.1',
      type_definitions: [],
    };

    let currentType = null;
    let inRelations = false;

    for (const line of lines) {
      if (line.startsWith('model') || line.startsWith('schema')) continue;

      if (line.startsWith('type ')) {
        if (currentType) {
          schema.type_definitions.push(currentType);
        }
        currentType = {
          type: line.replace('type ', '').trim(),
          relations: {},
        };
        inRelations = false;
      } else if (line === 'relations') {
        inRelations = true;
      } else if (line.startsWith('define ') && currentType && inRelations) {
        const match = line.match(/define\s+(\w+):\s*(.+)/);
        if (match) {
          const [, relName, relDef] = match;
          currentType.relations[relName] = parseRelationDef(relDef);
        }
      }
    }

    if (currentType) {
      schema.type_definitions.push(currentType);
    }

    return schema;
  };

  // Helper: Parse a relation definition
  const parseRelationDef = (def) => {
    def = def.trim();

    // Check for union (or)
    if (def.includes(' or ')) {
      const parts = def.split(' or ').map(p => p.trim());
      return {
        union: parts.map(p => parseRelationDef(p)),
      };
    }

    // Check for direct assignment [type1, type2]
    if (def.startsWith('[') && def.endsWith(']')) {
      const types = def.slice(1, -1).split(',').map(t => t.trim());
      return {
        this: { types },
      };
    }

    // Check for computed userset (from)
    if (def.includes(' from ')) {
      const [computed, tupleset] = def.split(' from ').map(p => p.trim());
      return {
        tuple_to_userset: {
          tupleset: { relation: tupleset },
          computed_userset: { relation: computed },
        },
      };
    }

    // Simple computed userset
    return {
      computed_userset: { relation: def },
    };
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-600">Loading...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-slate-900">FGA Stores</h1>
          <p className="text-slate-500">Manage Fine-Grained Authorization stores and models</p>
        </div>
        <button
          onClick={() => setShowCreateModal(true)}
          className="btn-primary"
        >
          + Create Store
        </button>
      </div>

      {/* Alerts */}
      {error && (
        <div className="p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
          {error}
          <button onClick={() => setError('')} className="ml-2 text-red-500 hover:text-red-700">x</button>
        </div>
      )}
      {success && (
        <div className="p-4 bg-green-50 border border-green-200 text-green-700 rounded-lg">
          {success}
        </div>
      )}

      {/* Stores List */}
      <div className="bg-white rounded-xl shadow-sm border border-slate-200">
        {stores.length === 0 ? (
          <div className="p-12 text-center">
            <div className="text-4xl mb-4">📦</div>
            <h3 className="text-lg font-medium text-slate-900">No FGA Stores</h3>
            <p className="text-slate-500 mt-1">Create your first store to start managing authorization</p>
            <button
              onClick={() => setShowCreateModal(true)}
              className="btn-primary mt-4"
            >
              Create Store
            </button>
          </div>
        ) : (
          <div className="divide-y divide-slate-200">
            {stores.map((store) => (
              <div key={store.id} className="p-6 hover:bg-slate-50">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-4">
                    <div className={`w-3 h-3 rounded-full ${store.is_active ? 'bg-green-500' : 'bg-slate-300'}`} />
                    <div>
                      <h3 className="font-medium text-slate-900">{store.name}</h3>
                      <p className="text-sm text-slate-500">{store.description || 'No description'}</p>
                    </div>
                  </div>
                  <div className="flex items-center space-x-6">
                    <div className="text-right text-sm">
                      <div className="text-slate-500">Model Version</div>
                      <div className="font-medium">{store.current_model_version || '-'}</div>
                    </div>
                    <div className="text-right text-sm">
                      <div className="text-slate-500">Tuples</div>
                      <div className="font-medium">{store.tuple_count.toLocaleString()}</div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <button
                        onClick={() => openModelModal(store)}
                        className="px-3 py-1.5 text-sm bg-blue-50 text-blue-600 rounded-lg hover:bg-blue-100"
                        title="Edit Model"
                      >
                        Model
                      </button>
                      <button
                        onClick={() => openApiKeyModal(store)}
                        className="px-3 py-1.5 text-sm bg-purple-50 text-purple-600 rounded-lg hover:bg-purple-100"
                        title="API Keys"
                      >
                        Keys
                      </button>
                      <button
                        onClick={() => openTuplesModal(store)}
                        className="px-3 py-1.5 text-sm bg-slate-100 text-slate-600 rounded-lg hover:bg-slate-200"
                        title="View Tuples"
                      >
                        Tuples
                      </button>
                      <button
                        onClick={() => handleDeleteStore(store.id)}
                        className="px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 rounded-lg"
                        title="Delete"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                </div>
                <div className="mt-3 text-xs text-slate-400">
                  ID: {store.id}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Create Store Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-md w-full mx-4">
            <div className="p-6 border-b border-slate-200">
              <h2 className="text-xl font-semibold">Create FGA Store</h2>
            </div>
            <form onSubmit={handleCreateStore} className="p-6 space-y-4">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">Name</label>
                <input
                  type="text"
                  required
                  value={createForm.name}
                  onChange={(e) => setCreateForm({ ...createForm, name: e.target.value })}
                  className="input-field"
                  placeholder="e.g., Production Store"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-1">Description</label>
                <textarea
                  value={createForm.description}
                  onChange={(e) => setCreateForm({ ...createForm, description: e.target.value })}
                  className="input-field"
                  placeholder="Optional description"
                  rows={3}
                />
              </div>
              <div className="flex justify-end space-x-3 pt-4">
                <button
                  type="button"
                  onClick={() => setShowCreateModal(false)}
                  className="btn-secondary"
                >
                  Cancel
                </button>
                <button type="submit" className="btn-primary">
                  Create Store
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Authorization Model Modal */}
      {showModelModal && selectedStore && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
            <div className="p-6 border-b border-slate-200">
              <h2 className="text-xl font-semibold">Authorization Model - {selectedStore.name}</h2>
              {currentModel && (
                <p className="text-sm text-slate-500 mt-1">
                  Version {currentModel.version} | {currentModel.is_valid ? 'Valid' : 'Invalid'}
                </p>
              )}
            </div>
            <form onSubmit={handleWriteModel} className="flex-1 overflow-hidden flex flex-col">
              <div className="p-6 flex-1 overflow-auto">
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Model Definition (DSL)
                </label>
                <textarea
                  value={modelForm.schema_dsl}
                  onChange={(e) => setModelForm({ ...modelForm, schema_dsl: e.target.value })}
                  className="w-full h-96 font-mono text-sm p-4 border border-slate-200 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                  placeholder="Enter your authorization model..."
                />
                <div className="mt-4 p-4 bg-slate-50 rounded-lg">
                  <h4 className="font-medium text-slate-700 mb-2">Quick Reference</h4>
                  <ul className="text-sm text-slate-600 space-y-1">
                    <li><code className="bg-slate-200 px-1 rounded">[user]</code> - Direct relation to users</li>
                    <li><code className="bg-slate-200 px-1 rounded">owner</code> - Computed from another relation</li>
                    <li><code className="bg-slate-200 px-1 rounded">[user] or owner</code> - Union of relations</li>
                    <li><code className="bg-slate-200 px-1 rounded">viewer from parent</code> - Inherited from related object</li>
                  </ul>
                </div>
              </div>
              <div className="p-6 border-t border-slate-200 flex justify-end space-x-3">
                <button
                  type="button"
                  onClick={() => setShowModelModal(false)}
                  className="btn-secondary"
                >
                  Close
                </button>
                <button type="submit" className="btn-primary">
                  Save Model
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* API Keys Modal */}
      {showApiKeyModal && selectedStore && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
            <div className="p-6 border-b border-slate-200">
              <h2 className="text-xl font-semibold">API Keys - {selectedStore.name}</h2>
            </div>
            <div className="flex-1 overflow-auto p-6 space-y-6">
              {/* New API Key Alert */}
              {newApiKey && (
                <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                  <h4 className="font-medium text-green-800">API Key Created</h4>
                  <p className="text-sm text-green-600 mt-1">
                    Copy this key now. It won't be shown again.
                  </p>
                  <div className="mt-2 p-2 bg-white border border-green-300 rounded font-mono text-sm break-all">
                    {newApiKey.key}
                  </div>
                  <button
                    onClick={() => {
                      navigator.clipboard.writeText(newApiKey.key);
                      setSuccess('Copied to clipboard');
                      setTimeout(() => setSuccess(''), 2000);
                    }}
                    className="mt-2 text-sm text-green-600 hover:text-green-800"
                  >
                    Copy to clipboard
                  </button>
                </div>
              )}

              {/* Create API Key Form */}
              <form onSubmit={handleCreateApiKey} className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-1">Key Name</label>
                  <input
                    type="text"
                    required
                    value={apiKeyForm.name}
                    onChange={(e) => setApiKeyForm({ ...apiKeyForm, name: e.target.value })}
                    className="input-field"
                    placeholder="e.g., Production API Key"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-700 mb-1">Permissions</label>
                  <div className="flex space-x-4">
                    {['read', 'write', 'check', 'admin'].map((perm) => (
                      <label key={perm} className="flex items-center">
                        <input
                          type="checkbox"
                          checked={apiKeyForm.permissions.includes(perm)}
                          onChange={(e) => {
                            if (e.target.checked) {
                              setApiKeyForm({
                                ...apiKeyForm,
                                permissions: [...apiKeyForm.permissions, perm],
                              });
                            } else {
                              setApiKeyForm({
                                ...apiKeyForm,
                                permissions: apiKeyForm.permissions.filter(p => p !== perm),
                              });
                            }
                          }}
                          className="mr-2"
                        />
                        <span className="text-sm">{perm}</span>
                      </label>
                    ))}
                  </div>
                </div>
                <button type="submit" className="btn-primary">
                  Create API Key
                </button>
              </form>

              {/* Existing Keys */}
              <div>
                <h4 className="font-medium text-slate-700 mb-3">Existing Keys</h4>
                {apiKeys.length === 0 ? (
                  <p className="text-sm text-slate-500">No API keys created yet</p>
                ) : (
                  <div className="space-y-2">
                    {apiKeys.map((key) => (
                      <div
                        key={key.id}
                        className="flex items-center justify-between p-3 bg-slate-50 rounded-lg"
                      >
                        <div>
                          <div className="font-medium">{key.name}</div>
                          <div className="text-sm text-slate-500">
                            {key.key_prefix}... | {key.permissions.join(', ')}
                          </div>
                        </div>
                        <button
                          onClick={() => handleRevokeApiKey(key.id)}
                          className={`text-sm ${key.is_active ? 'text-red-600 hover:text-red-800' : 'text-slate-400'}`}
                          disabled={!key.is_active}
                        >
                          {key.is_active ? 'Revoke' : 'Revoked'}
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
            <div className="p-6 border-t border-slate-200 flex justify-end">
              <button
                onClick={() => {
                  setShowApiKeyModal(false);
                  setNewApiKey(null);
                }}
                className="btn-secondary"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Tuples Modal */}
      {showTuplesModal && selectedStore && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col">
            <div className="p-6 border-b border-slate-200">
              <h2 className="text-xl font-semibold">Relation Tuples - {selectedStore.name}</h2>
              <p className="text-sm text-slate-500 mt-1">{tuples.length} tuples</p>
            </div>
            <div className="flex-1 overflow-auto">
              {tuples.length === 0 ? (
                <div className="p-12 text-center">
                  <p className="text-slate-500">No tuples in this store</p>
                </div>
              ) : (
                <table className="w-full">
                  <thead className="bg-slate-50 sticky top-0">
                    <tr>
                      <th className="px-4 py-3 text-left text-xs font-medium text-slate-500 uppercase">Subject</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-slate-500 uppercase">Relation</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-slate-500 uppercase">Object</th>
                      <th className="px-4 py-3 text-left text-xs font-medium text-slate-500 uppercase">Created</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-200">
                    {tuples.map((tuple) => (
                      <tr key={tuple.id} className="hover:bg-slate-50">
                        <td className="px-4 py-3 text-sm">
                          <span className="font-mono text-blue-600">
                            {tuple.subject_type}:{tuple.subject_id}
                            {tuple.subject_relation && `#${tuple.subject_relation}`}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-sm font-medium">{tuple.relation}</td>
                        <td className="px-4 py-3 text-sm">
                          <span className="font-mono text-purple-600">
                            {tuple.namespace}:{tuple.object_id}
                          </span>
                        </td>
                        <td className="px-4 py-3 text-sm text-slate-500">
                          {new Date(tuple.created_at).toLocaleDateString()}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
            <div className="p-6 border-t border-slate-200 flex justify-end">
              <button
                onClick={() => setShowTuplesModal(false)}
                className="btn-secondary"
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
