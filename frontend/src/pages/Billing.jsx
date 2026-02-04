import { useState, useEffect } from 'react';
import { billingApi } from '../api/client';

export default function Billing() {
  const [subscription, setSubscription] = useState(null);
  const [usage, setUsage] = useState(null);
  const [invoices, setInvoices] = useState([]);
  const [plans, setPlans] = useState([]);
  const [loading, setLoading] = useState(true);
  const [upgradeLoading, setUpgradeLoading] = useState(false);
  const [portalLoading, setPortalLoading] = useState(false);

  useEffect(() => {
    loadBillingData();
  }, []);

  const loadBillingData = async () => {
    try {
      const [subRes, usageRes, invoicesRes, plansRes] = await Promise.all([
        billingApi.getSubscription(),
        billingApi.getUsage(),
        billingApi.getInvoices(),
        billingApi.getPlans(),
      ]);
      setSubscription(subRes.data);
      setUsage(usageRes.data);
      setInvoices(invoicesRes.data.invoices || []);
      setPlans(plansRes.data.plans || []);
    } catch (error) {
      console.error('Failed to load billing data:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleUpgrade = async (planId) => {
    setUpgradeLoading(true);
    try {
      const { data } = await billingApi.createCheckout(planId);
      if (data.checkout_url) {
        window.location.href = data.checkout_url;
      }
    } catch (error) {
      console.error('Failed to create checkout session:', error);
      alert('Failed to start upgrade process. Please try again.');
    } finally {
      setUpgradeLoading(false);
    }
  };

  const handleManageBilling = async () => {
    setPortalLoading(true);
    try {
      const { data } = await billingApi.createPortalSession();
      if (data.portal_url) {
        window.location.href = data.portal_url;
      }
    } catch (error) {
      console.error('Failed to create portal session:', error);
      alert('Failed to open billing portal. Please try again.');
    } finally {
      setPortalLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-96">
        <div className="text-slate-600">Loading billing information...</div>
      </div>
    );
  }

  const currentPlan = subscription?.plan || { name: 'Free', mau_limit: 1000 };
  const mauUsed = usage?.mau_count || 0;
  const mauLimit = currentPlan.mau_limit || 1000;
  const mauPercentage = Math.min(100, Math.round((mauUsed / mauLimit) * 100));
  const isNearLimit = mauPercentage >= 80;
  const isAtLimit = mauPercentage >= 100;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-slate-900">Billing & Usage</h1>
        <p className="text-slate-600 mt-1">Manage your subscription, view usage, and download invoices</p>
      </div>

      {/* Current Plan & Usage */}
      <div className="grid md:grid-cols-2 gap-6">
        {/* Current Plan */}
        <div className="card">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-slate-900">Current Plan</h2>
            <span className={`px-3 py-1 rounded-full text-sm font-medium ${
              subscription?.status === 'active' ? 'bg-green-100 text-green-700' :
              subscription?.status === 'trialing' ? 'bg-blue-100 text-blue-700' :
              subscription?.status === 'past_due' ? 'bg-red-100 text-red-700' :
              'bg-slate-100 text-slate-700'
            }`}>
              {subscription?.status === 'active' ? 'Active' :
               subscription?.status === 'trialing' ? 'Trial' :
               subscription?.status === 'past_due' ? 'Past Due' :
               subscription?.status || 'Free'}
            </span>
          </div>

          <div className="flex items-baseline space-x-2 mb-4">
            <span className="text-4xl font-bold text-slate-900">{currentPlan.name}</span>
            {currentPlan.price_monthly_cents > 0 && (
              <span className="text-slate-600">
                ${(currentPlan.price_monthly_cents / 100).toFixed(0)}/month
              </span>
            )}
          </div>

          <ul className="space-y-2 mb-6">
            <li className="flex items-center text-sm text-slate-600">
              <svg className="w-4 h-4 text-green-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              {mauLimit.toLocaleString()} Monthly Active Users
            </li>
            <li className="flex items-center text-sm text-slate-600">
              <svg className="w-4 h-4 text-green-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              {currentPlan.app_limit ? `${currentPlan.app_limit} Applications` : 'Unlimited Applications'}
            </li>
            <li className="flex items-center text-sm text-slate-600">
              <svg className="w-4 h-4 text-green-500 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              {currentPlan.connection_limit ? `${currentPlan.connection_limit} SSO Connections` : 'Unlimited Connections'}
            </li>
          </ul>

          {subscription?.current_period_end && (
            <p className="text-sm text-slate-500 mb-4">
              {subscription.cancel_at_period_end
                ? `Cancels on ${new Date(subscription.current_period_end).toLocaleDateString()}`
                : `Renews on ${new Date(subscription.current_period_end).toLocaleDateString()}`
              }
            </p>
          )}

          <button
            onClick={handleManageBilling}
            disabled={portalLoading}
            className="w-full px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg transition-colors font-medium disabled:opacity-50"
          >
            {portalLoading ? 'Opening...' : 'Manage Billing'}
          </button>
        </div>

        {/* Usage */}
        <div className="card">
          <h2 className="text-xl font-semibold text-slate-900 mb-4">Monthly Active Users</h2>

          <div className="mb-6">
            <div className="flex items-baseline justify-between mb-2">
              <span className="text-4xl font-bold text-slate-900">{mauUsed.toLocaleString()}</span>
              <span className="text-slate-600">/ {mauLimit.toLocaleString()}</span>
            </div>

            <div className="w-full bg-slate-200 rounded-full h-4 overflow-hidden">
              <div
                className={`h-4 rounded-full transition-all duration-500 ${
                  isAtLimit ? 'bg-red-500' :
                  isNearLimit ? 'bg-yellow-500' :
                  'bg-primary-600'
                }`}
                style={{ width: `${mauPercentage}%` }}
              />
            </div>

            <div className="flex items-center justify-between mt-2">
              <span className={`text-sm font-medium ${
                isAtLimit ? 'text-red-600' :
                isNearLimit ? 'text-yellow-600' :
                'text-slate-600'
              }`}>
                {mauPercentage}% used
              </span>
              {usage?.period_end && (
                <span className="text-sm text-slate-500">
                  Resets {new Date(usage.period_end).toLocaleDateString()}
                </span>
              )}
            </div>
          </div>

          {isAtLimit && (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4 mb-4">
              <div className="flex items-start">
                <svg className="w-5 h-5 text-red-500 mt-0.5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <div>
                  <h4 className="text-sm font-medium text-red-800">Usage Limit Reached</h4>
                  <p className="text-sm text-red-600 mt-1">
                    New users cannot authenticate. Upgrade your plan to continue.
                  </p>
                </div>
              </div>
            </div>
          )}

          {isNearLimit && !isAtLimit && (
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
              <div className="flex items-start">
                <svg className="w-5 h-5 text-yellow-500 mt-0.5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <div>
                  <h4 className="text-sm font-medium text-yellow-800">Approaching Limit</h4>
                  <p className="text-sm text-yellow-600 mt-1">
                    Consider upgrading to avoid service interruption.
                  </p>
                </div>
              </div>
            </div>
          )}

          <div className="grid grid-cols-2 gap-4 pt-4 border-t border-slate-200">
            <div>
              <div className="text-2xl font-bold text-slate-900">{usage?.login_count?.toLocaleString() || 0}</div>
              <div className="text-sm text-slate-600">Logins this month</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-slate-900">{usage?.api_calls?.toLocaleString() || 0}</div>
              <div className="text-sm text-slate-600">API calls</div>
            </div>
          </div>
        </div>
      </div>

      {/* Available Plans */}
      <div className="card">
        <h2 className="text-xl font-semibold text-slate-900 mb-6">Available Plans</h2>

        <div className="grid md:grid-cols-4 gap-4">
          {plans.map((plan) => {
            const isCurrent = subscription?.plan_id === plan.id ||
                             (!subscription && plan.id === 'free');
            const isUpgrade = !isCurrent && plan.price_monthly_cents > (currentPlan.price_monthly_cents || 0);

            return (
              <div
                key={plan.id}
                className={`relative p-6 rounded-lg border-2 transition-all ${
                  isCurrent
                    ? 'border-primary-500 bg-primary-50'
                    : 'border-slate-200 hover:border-slate-300'
                }`}
              >
                {plan.id === 'pro' && (
                  <span className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-1 bg-primary-600 text-white text-xs font-medium rounded-full">
                    Popular
                  </span>
                )}

                <h3 className="text-lg font-semibold text-slate-900 mb-1">{plan.name}</h3>
                <div className="flex items-baseline mb-4">
                  <span className="text-3xl font-bold text-slate-900">
                    ${(plan.price_monthly_cents / 100).toFixed(0)}
                  </span>
                  <span className="text-slate-600 ml-1">/mo</span>
                </div>

                <ul className="space-y-2 mb-6 text-sm">
                  <li className="text-slate-600">
                    {plan.mau_limit?.toLocaleString() || 'Custom'} MAU
                  </li>
                  <li className="text-slate-600">
                    {plan.app_limit ? `${plan.app_limit} apps` : 'Unlimited apps'}
                  </li>
                  <li className="text-slate-600">
                    {plan.connection_limit ? `${plan.connection_limit} connections` : 'Unlimited connections'}
                  </li>
                </ul>

                {isCurrent ? (
                  <button
                    disabled
                    className="w-full px-4 py-2 bg-slate-100 text-slate-500 rounded-lg font-medium cursor-not-allowed"
                  >
                    Current Plan
                  </button>
                ) : isUpgrade ? (
                  <button
                    onClick={() => handleUpgrade(plan.id)}
                    disabled={upgradeLoading}
                    className="w-full px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
                  >
                    {upgradeLoading ? 'Processing...' : 'Upgrade'}
                  </button>
                ) : (
                  <button
                    onClick={handleManageBilling}
                    disabled={portalLoading}
                    className="w-full px-4 py-2 bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg font-medium transition-colors disabled:opacity-50"
                  >
                    Downgrade
                  </button>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Invoice History */}
      <div className="card">
        <h2 className="text-xl font-semibold text-slate-900 mb-4">Invoice History</h2>

        {invoices.length === 0 ? (
          <div className="text-center py-8">
            <svg className="w-12 h-12 text-slate-300 mx-auto mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p className="text-slate-600">No invoices yet</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-slate-200">
                  <th className="text-left py-3 px-4 text-sm font-medium text-slate-600">Date</th>
                  <th className="text-left py-3 px-4 text-sm font-medium text-slate-600">Amount</th>
                  <th className="text-left py-3 px-4 text-sm font-medium text-slate-600">Status</th>
                  <th className="text-right py-3 px-4 text-sm font-medium text-slate-600">Invoice</th>
                </tr>
              </thead>
              <tbody>
                {invoices.map((invoice) => (
                  <tr key={invoice.id} className="border-b border-slate-100 hover:bg-slate-50">
                    <td className="py-3 px-4 text-sm text-slate-900">
                      {new Date(invoice.created_at).toLocaleDateString('en-US', {
                        year: 'numeric',
                        month: 'short',
                        day: 'numeric'
                      })}
                    </td>
                    <td className="py-3 px-4 text-sm text-slate-900 font-medium">
                      ${(invoice.amount_cents / 100).toFixed(2)} {invoice.currency?.toUpperCase()}
                    </td>
                    <td className="py-3 px-4">
                      <span className={`inline-flex px-2 py-1 text-xs font-medium rounded-full ${
                        invoice.status === 'paid' ? 'bg-green-100 text-green-700' :
                        invoice.status === 'open' ? 'bg-blue-100 text-blue-700' :
                        invoice.status === 'draft' ? 'bg-slate-100 text-slate-700' :
                        'bg-red-100 text-red-700'
                      }`}>
                        {invoice.status.charAt(0).toUpperCase() + invoice.status.slice(1)}
                      </span>
                    </td>
                    <td className="py-3 px-4 text-right">
                      {invoice.invoice_pdf_url && (
                        <a
                          href={invoice.invoice_pdf_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="text-primary-600 hover:text-primary-700 text-sm font-medium"
                        >
                          Download PDF
                        </a>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Payment Method */}
      <div className="card">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold text-slate-900">Payment Method</h2>
          <button
            onClick={handleManageBilling}
            disabled={portalLoading}
            className="text-primary-600 hover:text-primary-700 text-sm font-medium disabled:opacity-50"
          >
            {portalLoading ? 'Opening...' : 'Update'}
          </button>
        </div>

        {subscription?.payment_method ? (
          <div className="flex items-center space-x-4 p-4 bg-slate-50 rounded-lg">
            <div className="w-12 h-8 bg-slate-200 rounded flex items-center justify-center">
              <svg className="w-8 h-5 text-slate-600" fill="currentColor" viewBox="0 0 24 24">
                <path d="M20 4H4c-1.11 0-1.99.89-1.99 2L2 18c0 1.11.89 2 2 2h16c1.11 0 2-.89 2-2V6c0-1.11-.89-2-2-2zm0 14H4v-6h16v6zm0-10H4V6h16v2z" />
              </svg>
            </div>
            <div>
              <div className="text-sm font-medium text-slate-900">
                {subscription.payment_method.brand?.charAt(0).toUpperCase() + subscription.payment_method.brand?.slice(1)} ending in {subscription.payment_method.last4}
              </div>
              <div className="text-sm text-slate-600">
                Expires {subscription.payment_method.exp_month}/{subscription.payment_method.exp_year}
              </div>
            </div>
          </div>
        ) : (
          <div className="text-center py-6">
            <svg className="w-12 h-12 text-slate-300 mx-auto mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z" />
            </svg>
            <p className="text-slate-600 mb-3">No payment method on file</p>
            <button
              onClick={handleManageBilling}
              disabled={portalLoading}
              className="px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
            >
              Add Payment Method
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
