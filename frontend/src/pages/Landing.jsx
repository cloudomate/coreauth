import { Link } from 'react-router-dom';

export default function Landing() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100">
      {/* Navigation */}
      <nav className="border-b border-slate-200 bg-white/80 backdrop-blur-sm sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <Link to="/" className="text-3xl">
              <span className="font-bold text-slate-900">core.</span>
              <span className="font-normal text-slate-500">auth</span>
            </Link>
            <div className="flex items-center space-x-4">
              <Link
                to="/login"
                className="text-slate-600 hover:text-slate-900 font-medium transition-colors"
              >
                Sign In
              </Link>
              <Link
                to="/signup"
                className="btn-primary"
              >
                Get Started
              </Link>
            </div>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <div className="max-w-7xl mx-auto px-6 pt-20 pb-32">
        <div className="text-center max-w-4xl mx-auto">
          <div className="inline-flex items-center space-x-2 bg-primary-50 text-primary-700 px-4 py-2 rounded-full text-sm font-medium mb-6">
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M6.267 3.455a3.066 3.066 0 001.745-.723 3.066 3.066 0 013.976 0 3.066 3.066 0 001.745.723 3.066 3.066 0 012.812 2.812c.051.643.304 1.254.723 1.745a3.066 3.066 0 010 3.976 3.066 3.066 0 00-.723 1.745 3.066 3.066 0 01-2.812 2.812 3.066 3.066 0 00-1.745.723 3.066 3.066 0 01-3.976 0 3.066 3.066 0 00-1.745-.723 3.066 3.066 0 01-2.812-2.812 3.066 3.066 0 00-.723-1.745 3.066 3.066 0 010-3.976 3.066 3.066 0 00.723-1.745 3.066 3.066 0 012.812-2.812zm7.44 5.252a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
            </svg>
            <span>Enterprise-ready identity platform for developers</span>
          </div>

          <h1 className="text-6xl font-bold mb-6 leading-tight">
            Authentication that
            <span className="block bg-gradient-to-r from-primary-600 via-blue-600 to-purple-600 bg-clip-text text-transparent">
              developers love
            </span>
          </h1>

          <p className="text-xl text-slate-600 mb-12 leading-relaxed">
            Drop-in authentication and authorization for your applications.
            Multi-tenant, hierarchical, and fully customizable. Built by developers, for developers.
          </p>

          <div className="flex items-center justify-center space-x-4">
            <Link to="/signup" className="btn-primary text-lg px-8 py-3.5">
              Start Building Free
            </Link>
            <a
              href="#features"
              className="btn-secondary text-lg px-8 py-3.5"
            >
              See Features
            </a>
          </div>

          {/* Code Example */}
          <div className="mt-16 bg-slate-900 rounded-2xl p-8 text-left shadow-2xl border border-slate-800">
            <div className="flex items-center space-x-2 mb-4">
              <div className="flex space-x-2">
                <div className="w-3 h-3 rounded-full bg-red-500"></div>
                <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
                <div className="w-3 h-3 rounded-full bg-green-500"></div>
              </div>
              <span className="text-slate-400 text-sm font-mono ml-4">Quick Start Example</span>
            </div>
            <pre className="text-sm font-mono overflow-x-auto">
              <code className="text-emerald-400">
{`// Initialize CoreAuth
const coreauth = new CoreAuth({
  domain: 'your-org.coreauth.dev',
  clientId: 'YOUR_CLIENT_ID'
});

// Login user
await coreauth.loginWithOrganization({
  email: 'user@example.com',
  password: 'password',
  organizationSlug: 'acme'
});

// Get user info
const user = await coreauth.getUser();
console.log(user.organization);`}
              </code>
            </pre>
          </div>
        </div>
      </div>

      {/* Features Section */}
      <div id="features" className="max-w-7xl mx-auto px-6 py-24">
        <div className="text-center mb-16">
          <h2 className="text-4xl font-bold mb-4">Built for modern applications</h2>
          <p className="text-xl text-slate-600">Everything you need to add authentication to your app</p>
        </div>

        <div className="grid md:grid-cols-3 gap-8">
          {features.map((feature, index) => (
            <div key={index} className="card hover:shadow-lg transition-shadow">
              <div className="w-12 h-12 bg-primary-100 rounded-lg flex items-center justify-center mb-4">
                <svg className="w-6 h-6 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={feature.icon} />
                </svg>
              </div>
              <h3 className="text-xl font-semibold mb-2">{feature.title}</h3>
              <p className="text-slate-600">{feature.description}</p>
            </div>
          ))}
        </div>
      </div>

      {/* CTA Section */}
      <div className="max-w-4xl mx-auto px-6 py-24 text-center">
        <div className="bg-gradient-to-r from-primary-600 to-primary-800 rounded-2xl p-12 text-white shadow-2xl">
          <h2 className="text-4xl font-bold mb-4">Ready to get started?</h2>
          <p className="text-xl mb-8 text-primary-100">
            Create your organization in seconds. No credit card required.
          </p>
          <Link
            to="/signup"
            className="inline-block px-8 py-3.5 bg-white text-primary-700 rounded-lg font-semibold hover:bg-primary-50 transition-colors"
          >
            Create Your Organization →
          </Link>
        </div>
      </div>

      {/* Footer */}
      <footer className="border-t border-slate-200 bg-white">
        <div className="max-w-7xl mx-auto px-6 py-12">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-2">
              <div className="w-8 h-8 bg-gradient-to-br from-primary-600 to-primary-800 rounded-lg flex items-center justify-center">
                <svg className="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
              <span className="text-xl">
                <span className="font-bold text-slate-900">core.</span>
                <span className="font-normal text-slate-500">auth</span>
              </span>
            </div>
            <p className="text-slate-500">© 2026 core.auth. Built for developers.</p>
          </div>
        </div>
      </footer>
    </div>
  );
}

const features = [
  {
    title: 'Multi-Tenant Architecture',
    description: 'Hierarchical organization support with isolated data and customizable authentication flows per organization.',
    icon: 'M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4'
  },
  {
    title: 'Enterprise SSO',
    description: 'SAML, OIDC, and social login providers. Let your users sign in with their existing accounts.',
    icon: 'M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z'
  },
  {
    title: 'Developer Experience',
    description: 'Clean APIs, comprehensive SDKs, and detailed documentation. Get started in minutes, not days.',
    icon: 'M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4'
  },
  {
    title: 'Fine-Grained Authorization',
    description: 'Relationship-based access control (ReBAC) for complex permission models. Similar to Google Zanzibar.',
    icon: 'M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z'
  },
  {
    title: 'Audit Logs',
    description: 'Complete audit trail of all authentication and authorization events. SOC2 and GDPR ready.',
    icon: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z'
  },
  {
    title: 'Self-Hosted',
    description: 'Deploy on your infrastructure. Keep full control over your data and compliance requirements.',
    icon: 'M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01'
  }
];
