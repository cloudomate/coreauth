import { useState, useEffect } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { QRCodeSVG } from 'qrcode.react';
import api from '../lib/api';

export default function MfaSetup() {
  const navigate = useNavigate();
  const [enrollmentData, setEnrollmentData] = useState(null);
  const [mfaData, setMfaData] = useState(null);
  const [verificationCode, setVerificationCode] = useState('');
  const [backupCodes, setBackupCodes] = useState([]);
  const [step, setStep] = useState('info'); // info, scan, verify, backup, complete
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const data = localStorage.getItem('mfa_enrollment_data');
    if (!data) {
      navigate('/login');
      return;
    }
    setEnrollmentData(JSON.parse(data));
  }, [navigate]);

  const handleStartEnrollment = async () => {
    setLoading(true);
    setError('');

    try {
      // Use enrollment token for unauthenticated MFA enrollment
      const response = await api.post('/mfa/enroll-with-token/totp', {
        enrollment_token: enrollmentData.enrollment_token,
      });

      setMfaData(response.data);
      setStep('scan');
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to start MFA enrollment');
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async (e) => {
    e.preventDefault();
    setLoading(true);
    setError('');

    try {
      // Verify with enrollment token - returns full auth response
      const response = await api.post(`/mfa/verify-with-token/totp/${mfaData.method_id}`, {
        enrollment_token: enrollmentData.enrollment_token,
        code: verificationCode,
      });

      // Check if we got auth tokens (successful verification)
      if (response.data.access_token) {
        // Store auth tokens
        localStorage.setItem('access_token', response.data.access_token);
        localStorage.setItem('refresh_token', response.data.refresh_token);
        localStorage.setItem('user', JSON.stringify(response.data.user));

        // Show backup codes
        setBackupCodes(mfaData.backup_codes || []);
        setStep('backup');
      } else {
        setError('Unexpected response from server');
      }
    } catch (err) {
      setError(err.response?.data?.message || 'Invalid verification code');
    } finally {
      setLoading(false);
    }
  };

  const handleComplete = () => {
    localStorage.removeItem('mfa_enrollment_data');
    navigate('/dashboard');
  };

  const handleSkip = () => {
    if (enrollmentData?.can_skip) {
      localStorage.removeItem('mfa_enrollment_data');
      navigate('/login');
    }
  };

  if (!enrollmentData) {
    return null;
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-100 flex items-center justify-center p-6">
      <div className="w-full max-w-2xl">
        {/* Logo */}
        <Link to="/" className="flex items-center justify-center mb-8">
          <span className="text-3xl">
            <span className="font-bold text-slate-900">core.</span>
            <span className="font-normal text-slate-500">auth</span>
          </span>
        </Link>

        {/* Card */}
        <div className="bg-white rounded-2xl shadow-xl border border-slate-200 p-8">
          {/* Step: Info */}
          {step === 'info' && (
            <>
              <div className="text-center mb-8">
                <div className="w-16 h-16 bg-primary-100 rounded-full flex items-center justify-center mx-auto mb-4">
                  <svg className="w-8 h-8 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                  </svg>
                </div>
                <h1 className="text-3xl font-bold mb-2">Set Up Multi-Factor Authentication</h1>
                <p className="text-slate-600">{enrollmentData.message}</p>
                {enrollmentData.grace_period_expires && (
                  <p className="text-sm text-yellow-600 mt-2">
                    Grace period expires: {new Date(enrollmentData.grace_period_expires).toLocaleDateString()}
                  </p>
                )}
              </div>

              {error && (
                <div className="mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
                  {error}
                </div>
              )}

              <div className="space-y-4 mb-8">
                <div className="flex items-start space-x-3">
                  <div className="w-6 h-6 bg-primary-600 text-white rounded-full flex items-center justify-center text-sm font-bold flex-shrink-0">1</div>
                  <div>
                    <h3 className="font-semibold text-slate-900">Download an authenticator app</h3>
                    <p className="text-sm text-slate-600">Get Google Authenticator, Microsoft Authenticator, or any compatible TOTP app</p>
                  </div>
                </div>
                <div className="flex items-start space-x-3">
                  <div className="w-6 h-6 bg-primary-600 text-white rounded-full flex items-center justify-center text-sm font-bold flex-shrink-0">2</div>
                  <div>
                    <h3 className="font-semibold text-slate-900">Scan the QR code</h3>
                    <p className="text-sm text-slate-600">Use your authenticator app to scan the QR code we'll show you</p>
                  </div>
                </div>
                <div className="flex items-start space-x-3">
                  <div className="w-6 h-6 bg-primary-600 text-white rounded-full flex items-center justify-center text-sm font-bold flex-shrink-0">3</div>
                  <div>
                    <h3 className="font-semibold text-slate-900">Enter the verification code</h3>
                    <p className="text-sm text-slate-600">Enter the 6-digit code from your app to verify setup</p>
                  </div>
                </div>
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={handleStartEnrollment}
                  disabled={loading}
                  className="flex-1 btn-primary py-3 disabled:opacity-50"
                >
                  {loading ? 'Setting up...' : 'Get Started'}
                </button>
                {enrollmentData.can_skip && (
                  <button
                    onClick={handleSkip}
                    className="px-6 py-3 border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50 transition-colors"
                  >
                    Skip for now
                  </button>
                )}
              </div>
            </>
          )}

          {/* Step: Scan QR Code */}
          {step === 'scan' && mfaData && (
            <>
              <div className="text-center mb-8">
                <h1 className="text-2xl font-bold mb-2">Scan QR Code</h1>
                <p className="text-slate-600">Use your authenticator app to scan this QR code</p>
              </div>

              <div className="flex justify-center mb-6">
                <div className="p-4 bg-white border-4 border-slate-200 rounded-xl">
                  <QRCodeSVG
                    value={mfaData.qr_code_uri}
                    size={256}
                    level="H"
                    includeMargin={false}
                  />
                </div>
              </div>

              <div className="bg-slate-50 border border-slate-200 rounded-lg p-4 mb-6">
                <p className="text-sm text-slate-600 mb-2">Can't scan? Enter this code manually:</p>
                <div className="flex items-center space-x-2">
                  <code className="flex-1 px-3 py-2 bg-white border border-slate-300 rounded font-mono text-sm">
                    {mfaData.secret}
                  </code>
                  <button
                    onClick={() => navigator.clipboard.writeText(mfaData.secret)}
                    className="px-3 py-2 bg-primary-600 text-white rounded hover:bg-primary-700 text-sm"
                  >
                    Copy
                  </button>
                </div>
              </div>

              <button
                onClick={() => setStep('verify')}
                className="w-full btn-primary py-3"
              >
                Continue to Verification
              </button>
            </>
          )}

          {/* Step: Verify Code */}
          {step === 'verify' && (
            <>
              <div className="text-center mb-8">
                <h1 className="text-2xl font-bold mb-2">Enter Verification Code</h1>
                <p className="text-slate-600">Enter the 6-digit code from your authenticator app</p>
              </div>

              {error && (
                <div className="mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
                  {error}
                </div>
              )}

              <form onSubmit={handleVerify} className="space-y-6">
                <div>
                  <input
                    type="text"
                    value={verificationCode}
                    onChange={(e) => setVerificationCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                    className="input-field text-center text-2xl font-mono tracking-widest"
                    placeholder="000000"
                    maxLength="6"
                    required
                    autoFocus
                  />
                </div>

                <div className="flex space-x-3">
                  <button
                    type="button"
                    onClick={() => setStep('scan')}
                    className="px-6 py-3 border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50 transition-colors"
                  >
                    Back
                  </button>
                  <button
                    type="submit"
                    disabled={loading || verificationCode.length !== 6}
                    className="flex-1 btn-primary py-3 disabled:opacity-50"
                  >
                    {loading ? 'Verifying...' : 'Verify'}
                  </button>
                </div>
              </form>
            </>
          )}

          {/* Step: Backup Codes */}
          {step === 'backup' && (
            <>
              <div className="text-center mb-8">
                <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
                  <svg className="w-8 h-8 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <h1 className="text-2xl font-bold mb-2">Save Your Backup Codes</h1>
                <p className="text-slate-600">Store these codes in a safe place. Each can be used once if you lose access to your authenticator.</p>
              </div>

              <div className="bg-slate-50 border border-slate-200 rounded-lg p-6 mb-6">
                <div className="grid grid-cols-2 gap-3">
                  {backupCodes.map((code, index) => (
                    <div key={index} className="bg-white border border-slate-300 rounded px-4 py-2 font-mono text-center">
                      {code}
                    </div>
                  ))}
                </div>
              </div>

              <div className="flex space-x-3">
                <button
                  onClick={() => {
                    const codes = backupCodes.join('\n');
                    navigator.clipboard.writeText(codes);
                  }}
                  className="flex-1 px-6 py-3 border border-slate-300 text-slate-700 rounded-lg hover:bg-slate-50 transition-colors"
                >
                  Copy Codes
                </button>
                <button
                  onClick={handleComplete}
                  className="flex-1 btn-primary py-3"
                >
                  Continue to Login
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
