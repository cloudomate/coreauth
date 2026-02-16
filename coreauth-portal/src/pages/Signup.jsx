import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import api from '../lib/api';

// Step indicator component
function StepIndicator({ currentStep, totalSteps, labels }) {
  return (
    <div className="mb-8">
      <div className="flex items-center justify-between">
        {labels.map((label, index) => {
          const stepNum = index + 1;
          const isActive = stepNum === currentStep;
          const isCompleted = stepNum < currentStep;

          return (
            <div key={stepNum} className="flex items-center flex-1">
              <div className="flex flex-col items-center">
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-all ${
                    isCompleted
                      ? 'bg-primary-600 text-white'
                      : isActive
                      ? 'bg-primary-600 text-white ring-4 ring-primary-100'
                      : 'bg-slate-200 text-slate-500'
                  }`}
                >
                  {isCompleted ? (
                    <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                    </svg>
                  ) : (
                    stepNum
                  )}
                </div>
                <span className={`text-xs mt-1 ${isActive ? 'text-primary-600 font-medium' : 'text-slate-500'}`}>
                  {label}
                </span>
              </div>
              {stepNum < totalSteps && (
                <div
                  className={`flex-1 h-0.5 mx-2 ${
                    isCompleted ? 'bg-primary-600' : 'bg-slate-200'
                  }`}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export default function Signup() {
  const navigate = useNavigate();
  const [step, setStep] = useState(1);
  const [accountType, setAccountType] = useState('');
  const [formData, setFormData] = useState({
    organizationName: '',
    organizationSlug: '',
    adminEmail: '',
    adminPassword: '',
    adminPasswordConfirm: '',
    adminFullName: '',
    isolationMode: 'shared',
  });
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [verificationRequired, setVerificationRequired] = useState(false);

  const totalSteps = accountType === 'business' ? 4 : 3;
  const stepLabels = accountType === 'business'
    ? ['Type', 'Details', 'Account', 'Options']
    : ['Type', 'Details', 'Account'];

  const generateSlug = (value) => {
    return value
      .toLowerCase()
      .replace(/[^a-z0-9-]/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-|-$/g, '');
  };

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }));

    if (accountType === 'business' && name === 'organizationName') {
      setFormData((prev) => ({
        ...prev,
        organizationSlug: generateSlug(value),
      }));
    } else if (accountType === 'personal' && name === 'adminFullName') {
      setFormData((prev) => ({
        ...prev,
        organizationSlug: generateSlug(value),
        organizationName: value,
      }));
    }
  };

  const handleAccountTypeSelect = (type) => {
    setAccountType(type);
    setStep(2);
    setError('');
    setFormData({
      organizationName: '',
      organizationSlug: '',
      adminEmail: '',
      adminPassword: '',
      adminPasswordConfirm: '',
      adminFullName: '',
      isolationMode: 'shared',
    });
  };

  const handleBack = () => {
    setError('');
    if (step === 2) {
      setStep(1);
      setAccountType('');
    } else {
      setStep(step - 1);
    }
  };

  const handleNext = () => {
    setError('');

    // Validation for step 2 (Details)
    if (step === 2) {
      if (accountType === 'business' && !formData.organizationName.trim()) {
        setError('Company name is required');
        return;
      }
      if (!formData.adminFullName.trim()) {
        setError('Full name is required');
        return;
      }
      if (!formData.adminEmail.trim()) {
        setError('Email is required');
        return;
      }
      if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.adminEmail)) {
        setError('Please enter a valid email address');
        return;
      }
    }

    // Validation for step 3 (Account)
    if (step === 3) {
      if (!formData.organizationSlug.trim()) {
        setError('Account name is required');
        return;
      }
      if (!/^[a-z0-9-]+$/.test(formData.organizationSlug)) {
        setError('Account name can only contain lowercase letters, numbers, and hyphens');
        return;
      }
      if (!formData.adminPassword) {
        setError('Password is required');
        return;
      }
      if (formData.adminPassword.length < 8) {
        setError('Password must be at least 8 characters');
        return;
      }
      if (formData.adminPassword !== formData.adminPasswordConfirm) {
        setError('Passwords do not match');
        return;
      }

      // For personal accounts, submit here
      if (accountType === 'personal') {
        handleSubmit();
        return;
      }
    }

    setStep(step + 1);
  };

  const handleSubmit = async (e) => {
    if (e) e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const response = await api.post('/tenants', {
        name: formData.organizationName,
        slug: formData.organizationSlug,
        admin_email: formData.adminEmail,
        admin_password: formData.adminPassword,
        admin_full_name: formData.adminFullName,
        account_type: accountType,
        isolation_mode: accountType === 'business' ? formData.isolationMode : 'shared',
      });

      if (response.data.email_verification_required) {
        setVerificationRequired(true);
        setLoading(false);
        return;
      }

      const loginResponse = await api.post('/auth/login-hierarchical', {
        email: formData.adminEmail,
        password: formData.adminPassword,
        organization_slug: formData.organizationSlug,
      });

      localStorage.setItem('access_token', loginResponse.data.access_token);
      localStorage.setItem('refresh_token', loginResponse.data.refresh_token);
      window.dispatchEvent(new Event('auth-change'));
      navigate('/dashboard');
    } catch (err) {
      console.error('Signup error:', err);
      setError(
        err.response?.data?.message || 'Failed to create account. Please try again.'
      );
    } finally {
      setLoading(false);
    }
  };

  // Email verification screen
  if (verificationRequired) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center p-6">
        <div className="w-full max-w-md">
          <Link to="/" className="flex items-center justify-center mb-8">
            <span className="text-3xl">
              <span className="font-bold text-slate-900">core.</span>
              <span className="font-normal text-slate-500">auth</span>
            </span>
          </Link>
          <div className="bg-white rounded-2xl shadow-xl border border-slate-200 p-8 text-center">
            <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <h1 className="text-2xl font-bold text-slate-900 mb-2">Check Your Email</h1>
            <p className="text-slate-600 mb-6">
              We've sent a verification link to <strong>{formData.adminEmail}</strong>.
              Please click the link to verify your email and activate your account.
            </p>
            <p className="text-sm text-slate-500 mb-6">
              Didn't receive the email? Check your spam folder or{' '}
              <button
                onClick={() => setVerificationRequired(false)}
                className="text-primary-600 hover:text-primary-700 font-medium"
              >
                try again
              </button>
            </p>
            <Link
              to="/login"
              className="inline-block px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
            >
              Go to Login
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center p-6">
      <div className="w-full max-w-md">
        {/* Logo */}
        <Link to="/" className="flex items-center justify-center mb-8">
          <img
            src="/core-auth-logo.svg"
            alt="CoreAuth"
            className="w-48 h-auto" style={{ minWidth: '140px' }}
          />
        </Link>

        {/* Card */}
        <div className="bg-white rounded-2xl shadow-xl border border-slate-200 p-8">

          {/* Step 1: Account Type Selection */}
          {step === 1 && (
            <>
              <div className="text-center mb-8">
                <h1 className="text-2xl font-bold mb-2">Create Your Account</h1>
                <p className="text-slate-600">Choose your account type to get started</p>
              </div>

              <div className="space-y-4">
                <button
                  type="button"
                  onClick={() => handleAccountTypeSelect('personal')}
                  className="w-full p-4 border-2 border-slate-200 rounded-xl hover:border-primary-500 hover:bg-primary-50 transition-all text-left group"
                >
                  <div className="flex items-start space-x-4">
                    <div className="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center flex-shrink-0 group-hover:bg-blue-200 transition-colors">
                      <svg className="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                      </svg>
                    </div>
                    <div>
                      <h3 className="font-semibold text-slate-900">Personal</h3>
                      <p className="text-sm text-slate-500 mt-1">For individual developers and personal projects</p>
                    </div>
                  </div>
                </button>

                <button
                  type="button"
                  onClick={() => handleAccountTypeSelect('business')}
                  className="w-full p-4 border-2 border-slate-200 rounded-xl hover:border-primary-500 hover:bg-primary-50 transition-all text-left group"
                >
                  <div className="flex items-start space-x-4">
                    <div className="w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center flex-shrink-0 group-hover:bg-purple-200 transition-colors">
                      <svg className="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4" />
                      </svg>
                    </div>
                    <div>
                      <h3 className="font-semibold text-slate-900">Business</h3>
                      <p className="text-sm text-slate-500 mt-1">For companies managing customer authentication</p>
                    </div>
                  </div>
                </button>
              </div>

              <div className="mt-6 text-center text-sm text-slate-600">
                Already have an account?{' '}
                <Link to="/login" className="text-primary-600 hover:text-primary-700 font-medium">
                  Sign in
                </Link>
              </div>
            </>
          )}

          {/* Steps 2-4: Multi-step form */}
          {step > 1 && (
            <>
              {/* Progress indicator */}
              <StepIndicator currentStep={step} totalSteps={totalSteps} labels={stepLabels} />

              {/* Back button */}
              <button
                type="button"
                onClick={handleBack}
                className="flex items-center text-sm text-slate-600 hover:text-slate-900 transition-colors mb-4"
              >
                <svg className="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                </svg>
                Back
              </button>

              {/* Error message */}
              {error && (
                <div className="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded-lg flex items-start space-x-2">
                  <svg className="w-5 h-5 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                    <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                  </svg>
                  <span className="text-sm">{error}</span>
                </div>
              )}

              {/* Step 2: Details */}
              {step === 2 && (
                <div>
                  <div className="text-center mb-6">
                    <h2 className="text-xl font-bold mb-1">Your Details</h2>
                    <p className="text-slate-600 text-sm">Tell us about yourself</p>
                  </div>

                  <div className="space-y-4">
                    {accountType === 'business' && (
                      <div>
                        <label className="block text-sm font-medium text-slate-700 mb-1.5">
                          Company Name
                        </label>
                        <input
                          type="text"
                          name="organizationName"
                          className="input-field"
                          placeholder="Acme Corporation"
                          value={formData.organizationName}
                          onChange={handleChange}
                          autoFocus
                        />
                      </div>
                    )}

                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1.5">
                        Full Name
                      </label>
                      <input
                        type="text"
                        name="adminFullName"
                        className="input-field"
                        placeholder="John Doe"
                        value={formData.adminFullName}
                        onChange={handleChange}
                        autoFocus={accountType === 'personal'}
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1.5">
                        Email Address
                      </label>
                      <input
                        type="email"
                        name="adminEmail"
                        className="input-field"
                        placeholder={accountType === 'personal' ? 'john@example.com' : 'john@acme.com'}
                        value={formData.adminEmail}
                        onChange={handleChange}
                      />
                    </div>
                  </div>

                  <button
                    type="button"
                    onClick={handleNext}
                    className="w-full btn-primary py-3 mt-6"
                  >
                    Continue
                  </button>
                </div>
              )}

              {/* Step 3: Account Setup */}
              {step === 3 && (
                <div>
                  <div className="text-center mb-6">
                    <h2 className="text-xl font-bold mb-1">Account Setup</h2>
                    <p className="text-slate-600 text-sm">Set up your account credentials</p>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1.5">
                        Account Name
                        <span className="text-slate-400 font-normal ml-1">(unique identifier)</span>
                      </label>
                      <input
                        type="text"
                        name="organizationSlug"
                        pattern="[a-z0-9-]+"
                        className="input-field"
                        placeholder={accountType === 'personal' ? 'john-doe' : 'acme'}
                        value={formData.organizationSlug}
                        onChange={handleChange}
                        autoFocus
                      />
                      <p className="text-xs text-slate-500 mt-1">
                        Lowercase letters, numbers, and hyphens only
                      </p>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1.5">
                        Password
                      </label>
                      <input
                        type="password"
                        name="adminPassword"
                        minLength={8}
                        className="input-field"
                        placeholder="••••••••"
                        value={formData.adminPassword}
                        onChange={handleChange}
                      />
                      <p className="text-xs text-slate-500 mt-1">At least 8 characters</p>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-slate-700 mb-1.5">
                        Confirm Password
                      </label>
                      <input
                        type="password"
                        name="adminPasswordConfirm"
                        minLength={8}
                        className="input-field"
                        placeholder="••••••••"
                        value={formData.adminPasswordConfirm}
                        onChange={handleChange}
                      />
                    </div>
                  </div>

                  <button
                    type="button"
                    onClick={handleNext}
                    disabled={loading}
                    className="w-full btn-primary py-3 mt-6 disabled:opacity-50"
                  >
                    {accountType === 'personal' ? (
                      loading ? (
                        <span className="flex items-center justify-center space-x-2">
                          <svg className="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                          </svg>
                          <span>Creating Account...</span>
                        </span>
                      ) : 'Create Account'
                    ) : 'Continue'}
                  </button>
                </div>
              )}

              {/* Step 4: Options (Business only) */}
              {step === 4 && accountType === 'business' && (
                <div>
                  <div className="text-center mb-6">
                    <h2 className="text-xl font-bold mb-1">Database Options</h2>
                    <p className="text-slate-600 text-sm">Choose how to store your customers' data</p>
                  </div>

                  <div className="space-y-3">
                    <label
                      className={`flex items-start p-4 border-2 rounded-xl cursor-pointer transition-all ${
                        formData.isolationMode === 'shared'
                          ? 'border-primary-500 bg-primary-50'
                          : 'border-slate-200 hover:border-slate-300'
                      }`}
                    >
                      <input
                        type="radio"
                        name="isolationMode"
                        value="shared"
                        checked={formData.isolationMode === 'shared'}
                        onChange={handleChange}
                        className="mt-0.5 h-4 w-4 text-primary-600 border-slate-300 focus:ring-primary-500"
                      />
                      <div className="ml-3 flex-1">
                        <div className="flex items-center justify-between">
                          <span className="font-medium text-slate-900">Shared Database</span>
                          <span className="text-xs font-medium text-green-600 bg-green-100 px-2 py-0.5 rounded">
                            Recommended
                          </span>
                        </div>
                        <p className="text-sm text-slate-500 mt-1">
                          Cost-effective multi-tenant database with logical isolation. Best for most use cases.
                        </p>
                      </div>
                    </label>

                    <label
                      className={`flex items-start p-4 border-2 rounded-xl cursor-pointer transition-all ${
                        formData.isolationMode === 'dedicated'
                          ? 'border-primary-500 bg-primary-50'
                          : 'border-slate-200 hover:border-slate-300'
                      }`}
                    >
                      <input
                        type="radio"
                        name="isolationMode"
                        value="dedicated"
                        checked={formData.isolationMode === 'dedicated'}
                        onChange={handleChange}
                        className="mt-0.5 h-4 w-4 text-primary-600 border-slate-300 focus:ring-primary-500"
                      />
                      <div className="ml-3 flex-1">
                        <div className="flex items-center justify-between">
                          <span className="font-medium text-slate-900">Dedicated Database</span>
                          <span className="text-xs font-medium text-purple-600 bg-purple-100 px-2 py-0.5 rounded">
                            Enterprise
                          </span>
                        </div>
                        <p className="text-sm text-slate-500 mt-1">
                          Your own isolated database instance. Ideal for compliance requirements.
                        </p>
                      </div>
                    </label>
                  </div>

                  {formData.isolationMode === 'dedicated' && (
                    <div className="mt-4 p-3 bg-amber-50 border border-amber-200 rounded-lg">
                      <p className="text-xs text-amber-700">
                        You'll configure your dedicated database connection in Settings after account creation.
                      </p>
                    </div>
                  )}

                  <button
                    type="button"
                    onClick={handleSubmit}
                    disabled={loading}
                    className="w-full btn-primary py-3 mt-6 disabled:opacity-50"
                  >
                    {loading ? (
                      <span className="flex items-center justify-center space-x-2">
                        <svg className="animate-spin h-5 w-5" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        <span>Creating Account...</span>
                      </span>
                    ) : 'Create Account'}
                  </button>
                </div>
              )}

              {step > 1 && (
                <div className="mt-6 text-center text-sm text-slate-600">
                  Already have an account?{' '}
                  <Link to="/login" className="text-primary-600 hover:text-primary-700 font-medium">
                    Sign in
                  </Link>
                </div>
              )}
            </>
          )}
        </div>

        <p className="text-center text-sm text-slate-500 mt-6">
          By signing up, you agree to our Terms of Service and Privacy Policy
        </p>
      </div>
    </div>
  );
}
