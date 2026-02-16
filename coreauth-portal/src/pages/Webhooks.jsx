import { useState, useEffect } from 'react';
import api from '../lib/api';

const WEBHOOK_EVENTS = [
  { id: 'user.created', label: 'User Created', description: 'When a new user registers' },
  { id: 'user.updated', label: 'User Updated', description: 'When user profile changes' },
  { id: 'user.deleted', label: 'User Deleted', description: 'When a user is removed' },
  { id: 'user.login', label: 'User Login', description: 'On successful authentication' },
  { id: 'user.login_failed', label: 'Login Failed', description: 'On failed login attempt' },
  { id: 'user.logout', label: 'User Logout', description: 'When user logs out' },
  { id: 'user.mfa_enrolled', label: 'MFA Enrolled', description: 'When MFA is enabled' },
  { id: 'user.password_changed', label: 'Password Changed', description: 'On password update' },
  { id: 'organization.created', label: 'Organization Created', description: 'New organization added' },
  { id: 'organization.updated', label: 'Organization Updated', description: 'Organization settings changed' },
];

export default function Webhooks() {
  const [webhooks, setWebhooks] = useState([]);
  const [deliveries, setDeliveries] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');
  const [showModal, setShowModal] = useState(false);
  const [showDeliveriesModal, setShowDeliveriesModal] = useState(false);
  const [selectedWebhook, setSelectedWebhook] = useState(null);
  const [editingWebhook, setEditingWebhook] = useState(null);
  const [testingWebhook, setTestingWebhook] = useState(null);
  const [formData, setFormData] = useState({
    name: '',
    url: '',
    events: [],
  });

  const user = JSON.parse(localStorage.getItem('user') || '{}');
  const tenantId = user.default_tenant_id;

  useEffect(() => {
    if (tenantId) {
      fetchWebhooks();
    } else {
      setLoading(false);
    }
  }, [tenantId]);

  const fetchWebhooks = async () => {
    try {
      const response = await api.get(`/organizations/${tenantId}/webhooks`);
      setWebhooks(response.data || []);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to load webhooks');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setSuccess('');

    if (formData.events.length === 0) {
      setError('Please select at least one event');
      return;
    }

    try {
      if (editingWebhook) {
        await api.put(`/organizations/${tenantId}/webhooks/${editingWebhook.id}`, {
          name: formData.name,
          url: formData.url,
          events: formData.events,
        });
        setSuccess('Webhook updated successfully');
      } else {
        await api.post(`/organizations/${tenantId}/webhooks`, {
          name: formData.name,
          url: formData.url,
          events: formData.events,
          is_enabled: true,
        });
        setSuccess('Webhook created successfully');
      }
      setShowModal(false);
      setEditingWebhook(null);
      setFormData({ name: '', url: '', events: [] });
      fetchWebhooks();
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to save webhook');
    }
  };

  const handleEdit = (webhook) => {
    setEditingWebhook(webhook);
    setFormData({
      name: webhook.name,
      url: webhook.url,
      events: webhook.events || [],
    });
    setShowModal(true);
  };

  const handleDelete = async (webhookId) => {
    if (!confirm('Are you sure you want to delete this webhook?')) return;

    try {
      await api.delete(`/organizations/${tenantId}/webhooks/${webhookId}`);
      setSuccess('Webhook deleted successfully');
      fetchWebhooks();
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to delete webhook');
    }
  };

  const handleToggle = async (webhook) => {
    try {
      await api.put(`/organizations/${tenantId}/webhooks/${webhook.id}`, {
        is_enabled: !webhook.is_enabled,
      });
      fetchWebhooks();
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to toggle webhook');
    }
  };

  const handleTest = async (webhook) => {
    setTestingWebhook(webhook.id);
    setError('');
    setSuccess('');

    try {
      await api.post(`/organizations/${tenantId}/webhooks/${webhook.id}/test`, {});
      setSuccess(`Test event sent to ${webhook.url}`);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to send test event');
    } finally {
      setTestingWebhook(null);
    }
  };

  const handleViewDeliveries = async (webhook) => {
    setSelectedWebhook(webhook);
    setShowDeliveriesModal(true);

    try {
      const response = await api.get(`/organizations/${tenantId}/webhooks/${webhook.id}/deliveries`);
      setDeliveries(response.data || []);
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to load deliveries');
    }
  };

  const handleEventToggle = (eventId) => {
    setFormData((prev) => ({
      ...prev,
      events: prev.events.includes(eventId)
        ? prev.events.filter((e) => e !== eventId)
        : [...prev.events, eventId],
    }));
  };

  const getStatusBadge = (status) => {
    switch (status) {
      case 'delivered':
        return 'bg-green-100 text-green-700';
      case 'failed':
        return 'bg-red-100 text-red-700';
      case 'pending':
        return 'bg-yellow-100 text-yellow-700';
      default:
        return 'bg-slate-100 text-slate-700';
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-slate-600">Loading webhooks...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-slate-900">Webhooks</h1>
          <p className="text-slate-600 mt-1">
            Send real-time notifications to your applications when events occur
          </p>
        </div>
        <button
          onClick={() => {
            setEditingWebhook(null);
            setFormData({ name: '', url: '', events: [] });
            setShowModal(true);
          }}
          className="btn-primary"
        >
          <svg className="w-5 h-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          Add Webhook
        </button>
      </div>

      {/* Alerts */}
      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded flex items-center justify-between">
          <span>{error}</span>
          <button onClick={() => setError('')} className="text-red-500 hover:text-red-700">
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}

      {success && (
        <div className="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded flex items-center justify-between">
          <span>{success}</span>
          <button onClick={() => setSuccess('')} className="text-green-500 hover:text-green-700">
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      )}

      {/* Webhooks List */}
      {webhooks.length === 0 ? (
        <div className="bg-white rounded-lg shadow border border-slate-200 p-12 text-center">
          <div className="w-16 h-16 bg-slate-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg className="w-8 h-8 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
          </div>
          <h3 className="text-lg font-semibold text-slate-900 mb-2">No webhooks configured</h3>
          <p className="text-slate-600 mb-6">Create your first webhook to receive real-time event notifications.</p>
          <button
            onClick={() => setShowModal(true)}
            className="btn-primary"
          >
            Create Webhook
          </button>
        </div>
      ) : (
        <div className="bg-white rounded-lg shadow border border-slate-200">
          <div className="divide-y divide-slate-200">
            {webhooks.map((webhook) => (
              <div key={webhook.id} className="p-6">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-3">
                      <h3 className="text-lg font-semibold text-slate-900">{webhook.name}</h3>
                      <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                        webhook.is_enabled ? 'bg-green-100 text-green-700' : 'bg-slate-100 text-slate-600'
                      }`}>
                        {webhook.is_enabled ? 'Active' : 'Disabled'}
                      </span>
                    </div>
                    <p className="text-sm text-slate-600 mt-1 font-mono">{webhook.url}</p>
                    <div className="flex flex-wrap gap-2 mt-3">
                      {(webhook.events || []).map((event) => (
                        <span key={event} className="px-2 py-1 bg-blue-50 text-blue-700 text-xs rounded">
                          {event}
                        </span>
                      ))}
                    </div>
                    {webhook.last_delivery_at && (
                      <p className="text-xs text-slate-500 mt-3">
                        Last delivery: {new Date(webhook.last_delivery_at).toLocaleString()}
                        {webhook.last_delivery_status && (
                          <span className={`ml-2 px-2 py-0.5 rounded ${getStatusBadge(webhook.last_delivery_status)}`}>
                            {webhook.last_delivery_status}
                          </span>
                        )}
                      </p>
                    )}
                  </div>
                  <div className="flex items-center space-x-2 ml-4">
                    <button
                      onClick={() => handleTest(webhook)}
                      disabled={testingWebhook === webhook.id}
                      className="p-2 text-slate-600 hover:text-blue-600 hover:bg-blue-50 rounded"
                      title="Send Test Event"
                    >
                      {testingWebhook === webhook.id ? (
                        <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                      ) : (
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                      )}
                    </button>
                    <button
                      onClick={() => handleViewDeliveries(webhook)}
                      className="p-2 text-slate-600 hover:text-blue-600 hover:bg-blue-50 rounded"
                      title="View Deliveries"
                    >
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleToggle(webhook)}
                      className={`p-2 rounded ${webhook.is_enabled ? 'text-yellow-600 hover:bg-yellow-50' : 'text-green-600 hover:bg-green-50'}`}
                      title={webhook.is_enabled ? 'Disable' : 'Enable'}
                    >
                      {webhook.is_enabled ? (
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                      ) : (
                        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                      )}
                    </button>
                    <button
                      onClick={() => handleEdit(webhook)}
                      className="p-2 text-slate-600 hover:text-blue-600 hover:bg-blue-50 rounded"
                      title="Edit"
                    >
                      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleDelete(webhook.id)}
                      className="p-2 text-slate-600 hover:text-red-600 hover:bg-red-50 rounded"
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
        </div>
      )}

      {/* Create/Edit Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto">
            <div className="p-6 border-b border-slate-200">
              <div className="flex items-center justify-between">
                <h2 className="text-2xl font-bold text-slate-900">
                  {editingWebhook ? 'Edit Webhook' : 'Create Webhook'}
                </h2>
                <button
                  onClick={() => {
                    setShowModal(false);
                    setEditingWebhook(null);
                  }}
                  className="text-slate-400 hover:text-slate-600"
                >
                  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>

            <form onSubmit={handleSubmit} className="p-6 space-y-6">
              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Webhook Name
                </label>
                <input
                  type="text"
                  required
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="input-field"
                  placeholder="e.g., Production Webhook"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-2">
                  Endpoint URL
                </label>
                <input
                  type="url"
                  required
                  value={formData.url}
                  onChange={(e) => setFormData({ ...formData, url: e.target.value })}
                  className="input-field"
                  placeholder="https://your-app.com/webhooks/coreauth"
                />
                <p className="text-xs text-slate-500 mt-1">
                  Must be a publicly accessible HTTPS URL
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-slate-700 mb-3">
                  Events to Subscribe
                </label>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  {WEBHOOK_EVENTS.map((event) => (
                    <label
                      key={event.id}
                      className={`flex items-start p-3 border rounded-lg cursor-pointer transition-colors ${
                        formData.events.includes(event.id)
                          ? 'border-primary-500 bg-primary-50'
                          : 'border-slate-200 hover:border-slate-300'
                      }`}
                    >
                      <input
                        type="checkbox"
                        checked={formData.events.includes(event.id)}
                        onChange={() => handleEventToggle(event.id)}
                        className="mt-0.5 mr-3"
                      />
                      <div>
                        <span className="font-medium text-slate-900">{event.label}</span>
                        <p className="text-xs text-slate-500">{event.description}</p>
                      </div>
                    </label>
                  ))}
                </div>
              </div>

              <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                <h4 className="font-semibold text-blue-900 mb-2">Webhook Security</h4>
                <p className="text-sm text-blue-700">
                  Each webhook request includes an <code className="bg-white px-1 rounded">X-CoreAuth-Signature</code> header
                  for verification. The signature is computed using HMAC-SHA256 with your webhook secret.
                </p>
                {editingWebhook && (
                  <div className="mt-3">
                    <span className="text-sm text-blue-700">Secret: </span>
                    <code className="bg-white px-2 py-1 rounded text-sm">{editingWebhook.secret || '••••••••'}</code>
                  </div>
                )}
              </div>

              <div className="flex space-x-3">
                <button
                  type="button"
                  onClick={() => {
                    setShowModal(false);
                    setEditingWebhook(null);
                  }}
                  className="flex-1 px-6 py-3 border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50"
                >
                  Cancel
                </button>
                <button type="submit" className="flex-1 btn-primary py-3">
                  {editingWebhook ? 'Save Changes' : 'Create Webhook'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Deliveries Modal */}
      {showDeliveriesModal && selectedWebhook && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white rounded-lg max-w-4xl w-full max-h-[90vh] overflow-y-auto">
            <div className="p-6 border-b border-slate-200">
              <div className="flex items-center justify-between">
                <div>
                  <h2 className="text-2xl font-bold text-slate-900">Delivery History</h2>
                  <p className="text-slate-600">{selectedWebhook.name}</p>
                </div>
                <button
                  onClick={() => {
                    setShowDeliveriesModal(false);
                    setSelectedWebhook(null);
                    setDeliveries([]);
                  }}
                  className="text-slate-400 hover:text-slate-600"
                >
                  <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>

            <div>
              {deliveries.length === 0 ? (
                <div className="text-center py-12">
                  <svg className="w-12 h-12 text-slate-300 mx-auto mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                  </svg>
                  <p className="text-slate-500">No deliveries yet</p>
                </div>
              ) : (
                <div className="space-y-4">
                  {deliveries.map((delivery) => (
                    <div key={delivery.id} className="border border-slate-200 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-3">
                        <div className="flex items-center space-x-3">
                          <span className={`px-2 py-1 rounded text-xs font-medium ${getStatusBadge(delivery.status)}`}>
                            {delivery.status}
                          </span>
                          <span className="text-sm font-medium text-slate-900">{delivery.event_type}</span>
                        </div>
                        <span className="text-xs text-slate-500">
                          {new Date(delivery.created_at).toLocaleString()}
                        </span>
                      </div>
                      {delivery.response_status && (
                        <div className="text-sm text-slate-600">
                          Response: HTTP {delivery.response_status}
                          {delivery.attempt_count > 1 && (
                            <span className="ml-2 text-slate-500">({delivery.attempt_count} attempts)</span>
                          )}
                        </div>
                      )}
                      {delivery.error_message && (
                        <div className="mt-2 text-sm text-red-600 bg-red-50 rounded p-2">
                          {delivery.error_message}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
