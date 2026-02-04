import { Routes, Route, Navigate } from 'react-router-dom';
import { useState, useEffect } from 'react';
import Layout from './components/Layout';
import Landing from './pages/Landing';
import Signup from './pages/Signup';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import Users from './pages/Users';
import Organizations from './pages/Organizations';
import Applications from './pages/Applications';
import Connections from './pages/Connections';
import Actions from './pages/Actions';
import Security from './pages/Security';
import Billing from './pages/Billing';
import Webhooks from './pages/Webhooks';
import SCIM from './pages/SCIM';
import MfaSetup from './pages/MfaSetup';
import AcceptInvitation from './pages/AcceptInvitation';
import OrgLogin from './pages/OrgLogin';
import SSOCallback from './pages/SSOCallback';

function App() {
  // Initialize from localStorage directly to avoid flash redirect
  const [isAuthenticated, setIsAuthenticated] = useState(() => {
    return !!localStorage.getItem('access_token');
  });

  // Listen for storage changes (login/logout in other tabs)
  useEffect(() => {
    const handleStorageChange = () => {
      setIsAuthenticated(!!localStorage.getItem('access_token'));
    };
    window.addEventListener('storage', handleStorageChange);
    return () => window.removeEventListener('storage', handleStorageChange);
  }, []);

  const ProtectedRoute = ({ children }) => {
    return isAuthenticated ? <Layout>{children}</Layout> : <Navigate to="/login" replace />;
  };

  return (
    <Routes>
      <Route path="/" element={<Landing />} />
      <Route path="/signup" element={<Signup />} />
      <Route path="/login" element={<Login />} />
      <Route path="/login/:orgSlug" element={<OrgLogin />} />
      <Route path="/sso/callback" element={<SSOCallback />} />
      <Route path="/mfa-setup" element={<MfaSetup />} />
      <Route path="/accept-invitation" element={<AcceptInvitation />} />
      <Route
        path="/dashboard"
        element={
          <ProtectedRoute>
            <Dashboard />
          </ProtectedRoute>
        }
      />
      <Route
        path="/users"
        element={
          <ProtectedRoute>
            <Users />
          </ProtectedRoute>
        }
      />
      <Route
        path="/organizations"
        element={
          <ProtectedRoute>
            <Organizations />
          </ProtectedRoute>
        }
      />
      <Route
        path="/applications"
        element={
          <ProtectedRoute>
            <Applications />
          </ProtectedRoute>
        }
      />
      <Route
        path="/connections"
        element={
          <ProtectedRoute>
            <Connections />
          </ProtectedRoute>
        }
      />
      <Route
        path="/actions"
        element={
          <ProtectedRoute>
            <Actions />
          </ProtectedRoute>
        }
      />
      <Route
        path="/security"
        element={
          <ProtectedRoute>
            <Security />
          </ProtectedRoute>
        }
      />
      <Route
        path="/billing"
        element={
          <ProtectedRoute>
            <Billing />
          </ProtectedRoute>
        }
      />
      <Route
        path="/webhooks"
        element={
          <ProtectedRoute>
            <Webhooks />
          </ProtectedRoute>
        }
      />
      <Route
        path="/scim"
        element={
          <ProtectedRoute>
            <SCIM />
          </ProtectedRoute>
        }
      />
    </Routes>
  );
}

export default App;
