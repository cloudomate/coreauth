import { Link } from 'react-router-dom';

export default function Dashboard() {
  return (
    <div className="space-y-6">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-slate-900 mb-2">
          Welcome to <span className="font-bold">core.</span><span className="font-normal text-slate-600">auth</span>
        </h1>
        <p className="text-slate-600">
          Your dashboard is ready. Start building your authentication flow.
        </p>
      </div>

      {/* Quick Stats */}
      <div className="grid md:grid-cols-4 gap-6 mb-12">
        {stats.map((stat, index) => (
          <div key={index} className="card">
            <div className="flex items-center justify-between mb-2">
              <span className="text-slate-600 text-sm font-medium">{stat.label}</span>
              <div className={`w-10 h-10 ${stat.color} rounded-lg flex items-center justify-center`}>
                <svg className="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={stat.icon} />
                </svg>
              </div>
            </div>
            <div className="text-3xl font-bold">{stat.value}</div>
            <div className="text-sm text-slate-500 mt-1">{stat.change}</div>
          </div>
        ))}
      </div>

      {/* Quick Actions */}
      <div className="grid md:grid-cols-2 gap-6 mb-12">
        <div className="card">
          <h2 className="text-xl font-semibold mb-4">Quick Start</h2>
          <div className="space-y-3">
            {quickActions.map((action, index) => (
              <Link
                key={index}
                to={action.link}
                className="block w-full text-left p-4 rounded-lg border border-slate-200 hover:border-primary-300 hover:bg-primary-50/50 transition-all group"
              >
                <div className="flex items-start space-x-3">
                  <div className="w-10 h-10 bg-primary-100 rounded-lg flex items-center justify-center group-hover:bg-primary-200 transition-colors">
                    <svg className="w-5 h-5 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={action.icon} />
                    </svg>
                  </div>
                  <div className="flex-1">
                    <div className="font-medium text-slate-900 mb-1">{action.title}</div>
                    <div className="text-sm text-slate-600">{action.description}</div>
                  </div>
                  <svg className="w-5 h-5 text-slate-400 group-hover:text-primary-600 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                  </svg>
                </div>
              </Link>
            ))}
          </div>
        </div>

        <div className="card">
          <h2 className="text-xl font-semibold mb-4">Resources</h2>
          <div className="space-y-3">
            {resources.map((resource, index) => (
              <a
                key={index}
                href="#"
                className="block p-4 rounded-lg border border-slate-200 hover:border-primary-300 hover:bg-primary-50/50 transition-all"
              >
                <div className="flex items-center justify-between">
                  <div>
                    <div className="font-medium text-slate-900 mb-1">{resource.title}</div>
                    <div className="text-sm text-slate-600">{resource.description}</div>
                  </div>
                  <svg className="w-5 h-5 text-slate-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                  </svg>
                </div>
              </a>
            ))}
          </div>
        </div>
      </div>

      {/* Recent Activity */}
      <div className="card">
        <h2 className="text-xl font-semibold mb-4">Recent Activity</h2>
        <div className="space-y-4">
          {recentActivity.map((activity, index) => (
            <div key={index} className="flex items-start space-x-4 p-4 rounded-lg hover:bg-slate-50 transition-colors">
              <div className={`w-10 h-10 ${activity.color} rounded-lg flex items-center justify-center flex-shrink-0`}>
                <svg className="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={activity.icon} />
                </svg>
              </div>
              <div className="flex-1">
                <div className="text-slate-900 font-medium">{activity.title}</div>
                <div className="text-sm text-slate-600 mt-1">{activity.description}</div>
              </div>
              <div className="text-sm text-slate-500">{activity.time}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

const stats = [
  {
    label: 'Total Users',
    value: '1',
    change: '+1 this month',
    color: 'bg-primary-700',
    icon: 'M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z'
  },
  {
    label: 'Active Sessions',
    value: '1',
    change: 'Live now',
    color: 'bg-primary-600',
    icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z'
  },
  {
    label: 'API Calls',
    value: '0',
    change: 'Last 24 hours',
    color: 'bg-primary-800',
    icon: 'M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z'
  },
  {
    label: 'Uptime',
    value: '100%',
    change: '30 days',
    color: 'bg-primary-900',
    icon: 'M13 10V3L4 14h7v7l9-11h-7z'
  }
];

const quickActions = [
  {
    title: 'Configure SSO',
    description: 'Add SAML or OIDC providers for enterprise authentication',
    icon: 'M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z',
    link: '/connections'
  },
  {
    title: 'Add Application',
    description: 'Register your app to start authenticating users',
    icon: 'M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4',
    link: '/applications'
  },
  {
    title: 'Create Actions',
    description: 'Extend auth flows with custom JavaScript hooks',
    icon: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4',
    link: '/actions'
  },
  {
    title: 'Manage Organizations',
    description: 'Configure your multi-tenant hierarchy',
    icon: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4',
    link: '/organizations'
  }
];

const resources = [
  {
    title: 'API Documentation',
    description: 'Integrate core.auth into your application'
  },
  {
    title: 'SDK Libraries',
    description: 'Official SDKs for all major languages'
  },
  {
    title: 'Example Projects',
    description: 'Sample apps and integration guides'
  },
  {
    title: 'Support',
    description: 'Get help from our team'
  }
];

const recentActivity = [
  {
    title: 'Tenant Created',
    description: 'Your tenant account was successfully set up',
    time: 'Just now',
    color: 'bg-primary-700',
    icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z'
  },
  {
    title: 'Admin User Added',
    description: 'Admin account was created and verified',
    time: 'Just now',
    color: 'bg-primary-800',
    icon: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z'
  }
];
