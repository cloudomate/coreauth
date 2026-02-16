# CoreAuth CIAM - Comprehensive Test Cases

> **Tenants:** `corerun` (EntraID: 44c462b5-fc20-4de0-8c23-fcec7516435c) and `imys` (EntraID: b6e6a0ae-c665-470c-808d-161c2fa37323)
> **Base URL:** `http://localhost:8000` (Backend API), `http://localhost:4000` (Proxy)

---

## Table of Contents

1. [Authentication - Basic](#1-authentication---basic)
2. [Authentication - Hierarchical Login](#2-authentication---hierarchical-login)
3. [Email Verification](#3-email-verification)
4. [Password Reset](#4-password-reset)
5. [Multi-Factor Authentication (MFA)](#5-multi-factor-authentication-mfa)
6. [Passwordless Authentication](#6-passwordless-authentication)
7. [OAuth2/OIDC Server](#7-oauth2oidc-server)
8. [Universal Login](#8-universal-login)
9. [Self-Service Flows (Headless API)](#9-self-service-flows-headless-api)
10. [EntraID/Azure AD Integration](#10-entraidazure-ad-integration)
11. [Social Login & OIDC Providers](#11-social-login--oidc-providers)
12. [Connection Management](#12-connection-management)
13. [Tenant & Organization Management](#13-tenant--organization-management)
14. [Invitation System](#14-invitation-system)
15. [Groups Management](#15-groups-management)
16. [Session Management](#16-session-management)
17. [OAuth Application Management](#17-oauth-application-management)
18. [Fine-Grained Authorization (FGA)](#18-fine-grained-authorization-fga)
19. [FGA Stores & Models](#19-fga-stores--models)
20. [Actions/Hooks](#20-actionshooks)
21. [Webhooks](#21-webhooks)
22. [SCIM 2.0 Provisioning](#22-scim-20-provisioning)
23. [Audit Logging](#23-audit-logging)
24. [Email Templates](#24-email-templates)
25. [Branding & Security Settings](#25-branding--security-settings)
26. [Rate Limiting & Token Customization](#26-rate-limiting--token-customization)
27. [Tenant Registry (Platform Admin)](#27-tenant-registry-platform-admin)
28. [Forward Auth & Proxy Integration](#28-forward-auth--proxy-integration)
29. [Sample App (CoreRun) - Workspaces](#29-sample-app-corerun---workspaces)
30. [Sample App (CoreRun) - Resources & FGA](#30-sample-app-corerun---resources--fga)
31. [Sample App (CoreRun) - Admin Features](#31-sample-app-corerun---admin-features)

---

## 1. Authentication - Basic

### 1.1 User Registration

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| AUTH-REG-001 | Register with valid email, password, first/last name | POST | `/api/auth/register` | 201 Created, user object + tokens returned | Positive |
| AUTH-REG-002 | Register with minimum valid password (8 chars) | POST | `/api/auth/register` | 201 Created | Positive |
| AUTH-REG-003 | Register with all optional fields (phone, avatar, language, timezone) | POST | `/api/auth/register` | 201 Created with all fields | Positive |
| AUTH-REG-004 | Register with duplicate email | POST | `/api/auth/register` | 409 Conflict, "email already exists" | Negative |
| AUTH-REG-005 | Register with invalid email format | POST | `/api/auth/register` | 400 Bad Request, validation error | Negative |
| AUTH-REG-006 | Register with empty email | POST | `/api/auth/register` | 400 Bad Request | Negative |
| AUTH-REG-007 | Register with password too short (<8 chars) | POST | `/api/auth/register` | 400 Bad Request, "password too short" | Negative |
| AUTH-REG-008 | Register with empty password | POST | `/api/auth/register` | 400 Bad Request | Negative |
| AUTH-REG-009 | Register with empty body | POST | `/api/auth/register` | 400 Bad Request | Negative |
| AUTH-REG-010 | Register with missing tenant_id | POST | `/api/auth/register` | 400 Bad Request | Negative |
| AUTH-REG-011 | Register with non-existent tenant_id | POST | `/api/auth/register` | 404 Not Found | Negative |
| AUTH-REG-012 | Rate limit: rapid registration attempts (>5 in 60s) | POST | `/api/auth/register` | 429 Too Many Requests | Negative |
| AUTH-REG-013 | Register with SQL injection in email field | POST | `/api/auth/register` | 400 Bad Request (sanitized) | Security |
| AUTH-REG-014 | Register with XSS payload in name fields | POST | `/api/auth/register` | 201 but sanitized output | Security |
| AUTH-REG-015 | Register with extremely long email (>255 chars) | POST | `/api/auth/register` | 400 Bad Request | Negative |
| AUTH-REG-016 | Register with extremely long password (>1000 chars) | POST | `/api/auth/register` | 400 Bad Request or 201 (bounded) | Boundary |

### 1.2 User Login

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| AUTH-LOGIN-001 | Login with valid credentials | POST | `/api/auth/login` | 200 OK, access_token + refresh_token | Positive |
| AUTH-LOGIN-002 | Login with wrong password | POST | `/api/auth/login` | 401 Unauthorized | Negative |
| AUTH-LOGIN-003 | Login with non-existent email | POST | `/api/auth/login` | 401 Unauthorized (no user enumeration) | Negative |
| AUTH-LOGIN-004 | Login with empty email | POST | `/api/auth/login` | 400 Bad Request | Negative |
| AUTH-LOGIN-005 | Login with empty password | POST | `/api/auth/login` | 400 Bad Request | Negative |
| AUTH-LOGIN-006 | Login with empty body | POST | `/api/auth/login` | 400 Bad Request | Negative |
| AUTH-LOGIN-007 | Rate limit: rapid login attempts (>10 in 60s) | POST | `/api/auth/login` | 429 Too Many Requests | Negative |
| AUTH-LOGIN-008 | Login with account lockout (after N failed attempts) | POST | `/api/auth/login` | 423 Locked / 429 | Negative |
| AUTH-LOGIN-009 | Login with unverified email (when verification required) | POST | `/api/auth/login` | 403 or 200 with verification_required flag | Negative |
| AUTH-LOGIN-010 | Login returns correct user profile data | POST | `/api/auth/login` | Verify user object fields match | Positive |
| AUTH-LOGIN-011 | Login with case-insensitive email | POST | `/api/auth/login` | 200 OK (email normalized) | Positive |
| AUTH-LOGIN-012 | Login with SQL injection in email | POST | `/api/auth/login` | 401 (not SQL error) | Security |
| AUTH-LOGIN-013 | Login response does not leak password hash | POST | `/api/auth/login` | No password field in response | Security |

### 1.3 Token Refresh

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| AUTH-REFRESH-001 | Refresh with valid refresh_token | POST | `/api/auth/refresh` | 200 OK, new access_token | Positive |
| AUTH-REFRESH-002 | Refresh with expired refresh_token | POST | `/api/auth/refresh` | 401 Unauthorized | Negative |
| AUTH-REFRESH-003 | Refresh with invalid refresh_token | POST | `/api/auth/refresh` | 401 Unauthorized | Negative |
| AUTH-REFRESH-004 | Refresh with empty token | POST | `/api/auth/refresh` | 400 Bad Request | Negative |
| AUTH-REFRESH-005 | Refresh after logout (revoked token) | POST | `/api/auth/refresh` | 401 Unauthorized | Negative |
| AUTH-REFRESH-006 | Refresh returns new refresh_token (rotation) | POST | `/api/auth/refresh` | New refresh_token returned | Positive |

### 1.4 User Profile (me)

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| AUTH-ME-001 | Get current user profile with valid token | GET | `/api/auth/me` | 200 OK, full user object | Positive |
| AUTH-ME-002 | Get profile without Authorization header | GET | `/api/auth/me` | 401 Unauthorized | Negative |
| AUTH-ME-003 | Get profile with expired token | GET | `/api/auth/me` | 401 Unauthorized | Negative |
| AUTH-ME-004 | Get profile with malformed JWT | GET | `/api/auth/me` | 401 Unauthorized | Negative |
| AUTH-ME-005 | Update profile (first_name, last_name) | PATCH | `/api/auth/me` | 200 OK, updated fields | Positive |
| AUTH-ME-006 | Update profile with empty body | PATCH | `/api/auth/me` | 200 OK (no changes) or 400 | Boundary |
| AUTH-ME-007 | Update profile with invalid field names | PATCH | `/api/auth/me` | 200 OK (ignored) or 400 | Negative |
| AUTH-ME-008 | Profile does not expose password_hash | GET | `/api/auth/me` | No password-related fields | Security |

### 1.5 Change Password

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| AUTH-CHPWD-001 | Change password with valid current password | POST | `/api/auth/change-password` | 200 OK | Positive |
| AUTH-CHPWD-002 | Change password with wrong current password | POST | `/api/auth/change-password` | 401 Unauthorized | Negative |
| AUTH-CHPWD-003 | Change password with new password too short | POST | `/api/auth/change-password` | 400 Bad Request | Negative |
| AUTH-CHPWD-004 | Change password same as old password | POST | `/api/auth/change-password` | 400 Bad Request (should differ) | Negative |
| AUTH-CHPWD-005 | Change password without auth | POST | `/api/auth/change-password` | 401 Unauthorized | Negative |
| AUTH-CHPWD-006 | Verify old tokens invalidated after password change | - | - | Old tokens should fail on next request | Security |

### 1.6 Logout

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| AUTH-LOGOUT-001 | Logout with valid token | POST | `/api/auth/logout` | 200 OK | Positive |
| AUTH-LOGOUT-002 | Verify access_token invalid after logout | GET | `/api/auth/me` | 401 Unauthorized | Security |
| AUTH-LOGOUT-003 | Verify refresh_token invalid after logout | POST | `/api/auth/refresh` | 401 Unauthorized | Security |
| AUTH-LOGOUT-004 | Logout without token | POST | `/api/auth/logout` | 401 Unauthorized | Negative |
| AUTH-LOGOUT-005 | Double logout (already logged out) | POST | `/api/auth/logout` | 200 OK (idempotent) or 401 | Boundary |

---

## 2. Authentication - Hierarchical Login

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| AUTH-HIER-001 | Login with email + password + organization slug | POST | `/api/auth/login-hierarchical` | 200 OK, token with org context | Positive |
| AUTH-HIER-002 | Login with valid creds but wrong organization | POST | `/api/auth/login-hierarchical` | 401/403 not a member | Negative |
| AUTH-HIER-003 | Login with non-existent organization slug | POST | `/api/auth/login-hierarchical` | 404 Not Found | Negative |
| AUTH-HIER-004 | Login without organization (fallback to basic) | POST | `/api/auth/login-hierarchical` | 200 OK, basic token | Positive |
| AUTH-HIER-005 | Rate limited same as standard login | POST | `/api/auth/login-hierarchical` | 429 after threshold | Negative |
| AUTH-HIER-006 | Token contains tenant_id claim when org provided | POST | `/api/auth/login-hierarchical` | Decode JWT, verify tenant_id | Positive |

---

## 3. Email Verification

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| VERIFY-001 | Verify email with valid token | GET | `/api/verify-email?token=xxx` | 200 OK, email verified | Positive |
| VERIFY-002 | Verify email with expired token | GET | `/api/verify-email?token=expired` | 400/410 Token expired | Negative |
| VERIFY-003 | Verify email with invalid token | GET | `/api/verify-email?token=invalid` | 400 Invalid token | Negative |
| VERIFY-004 | Verify email with already-verified account | GET | `/api/verify-email?token=xxx` | 200 (idempotent) or 400 | Boundary |
| VERIFY-005 | Verify email with missing token param | GET | `/api/verify-email` | 400 Missing token | Negative |
| VERIFY-006 | Resend verification email (authenticated) | POST | `/api/auth/resend-verification` | 200 OK | Positive |
| VERIFY-007 | Resend verification without auth | POST | `/api/auth/resend-verification` | 401 Unauthorized | Negative |
| VERIFY-008 | Resend verification for already-verified email | POST | `/api/auth/resend-verification` | 400 Already verified | Negative |
| VERIFY-009 | Verify token is single-use | GET | `/api/verify-email?token=xxx` | Second use returns error | Security |

---

## 4. Password Reset

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| PWRESET-001 | Request password reset for existing email | POST | `/api/auth/forgot-password` | 200 OK (email sent) | Positive |
| PWRESET-002 | Request reset for non-existent email | POST | `/api/auth/forgot-password` | 200 OK (no enumeration) | Security |
| PWRESET-003 | Rate limit: rapid reset requests | POST | `/api/auth/forgot-password` | 429 Too Many Requests | Negative |
| PWRESET-004 | Verify reset token is valid | GET | `/api/auth/verify-reset-token?token=xxx` | 200 OK, token valid | Positive |
| PWRESET-005 | Verify expired reset token | GET | `/api/auth/verify-reset-token?token=expired` | 400 Token expired | Negative |
| PWRESET-006 | Verify invalid reset token | GET | `/api/auth/verify-reset-token?token=random` | 400 Invalid token | Negative |
| PWRESET-007 | Reset password with valid token + new password | POST | `/api/auth/reset-password` | 200 OK, password changed | Positive |
| PWRESET-008 | Reset password with expired token | POST | `/api/auth/reset-password` | 400 Token expired | Negative |
| PWRESET-009 | Reset password with weak new password | POST | `/api/auth/reset-password` | 400 Password too weak | Negative |
| PWRESET-010 | Reset token is single-use | POST | `/api/auth/reset-password` | Second use returns error | Security |
| PWRESET-011 | Old password no longer works after reset | POST | `/api/auth/login` | 401 Unauthorized | Security |
| PWRESET-012 | New password works after reset | POST | `/api/auth/login` | 200 OK | Positive |

---

## 5. Multi-Factor Authentication (MFA)

### 5.1 TOTP Enrollment

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| MFA-TOTP-001 | Enroll in TOTP (authenticated) | POST | `/api/mfa/enroll/totp` | 200 OK, secret + QR URI + backup codes | Positive |
| MFA-TOTP-002 | Verify TOTP enrollment with valid code | POST | `/api/mfa/totp/:method_id/verify` | 200 OK, method activated | Positive |
| MFA-TOTP-003 | Verify TOTP enrollment with wrong code | POST | `/api/mfa/totp/:method_id/verify` | 400 Invalid code | Negative |
| MFA-TOTP-004 | Verify TOTP with expired code (>30s window) | POST | `/api/mfa/totp/:method_id/verify` | 400 Invalid code | Negative |
| MFA-TOTP-005 | Enroll TOTP without auth | POST | `/api/mfa/enroll/totp` | 401 Unauthorized | Negative |
| MFA-TOTP-006 | Enroll TOTP with enrollment token (unauthenticated flow) | POST | `/api/mfa/enroll-with-token/totp` | 200 OK | Positive |
| MFA-TOTP-007 | Verify TOTP with enrollment token | POST | `/api/mfa/verify-with-token/totp/:method_id` | 200 OK | Positive |
| MFA-TOTP-008 | Enroll TOTP with invalid enrollment token | POST | `/api/mfa/enroll-with-token/totp` | 401 Invalid token | Negative |
| MFA-TOTP-009 | Duplicate TOTP enrollment (already enrolled) | POST | `/api/mfa/enroll/totp` | 409 Conflict or 200 (replace) | Boundary |

### 5.2 SMS MFA

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| MFA-SMS-001 | Enroll in SMS MFA (authenticated) | POST | `/api/mfa/enroll/sms` | 200 OK, OTP sent | Positive |
| MFA-SMS-002 | Verify SMS OTP with correct code | POST | `/api/mfa/sms/:method_id/verify` | 200 OK, method activated | Positive |
| MFA-SMS-003 | Verify SMS OTP with wrong code | POST | `/api/mfa/sms/:method_id/verify` | 400 Invalid code | Negative |
| MFA-SMS-004 | Resend SMS OTP | POST | `/api/mfa/sms/:method_id/resend` | 200 OK, new OTP sent | Positive |
| MFA-SMS-005 | Enroll SMS without phone number | POST | `/api/mfa/enroll/sms` | 400 Phone required | Negative |
| MFA-SMS-006 | Enroll SMS without auth | POST | `/api/mfa/enroll/sms` | 401 Unauthorized | Negative |
| MFA-SMS-007 | Resend OTP rate limiting | POST | `/api/mfa/sms/:method_id/resend` | 429 after threshold | Negative |

### 5.3 MFA Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| MFA-MGT-001 | List all MFA methods | GET | `/api/mfa/methods` | 200 OK, array of methods | Positive |
| MFA-MGT-002 | Delete MFA method | DELETE | `/api/mfa/methods/:method_id` | 200 OK, method removed | Positive |
| MFA-MGT-003 | Delete non-existent method | DELETE | `/api/mfa/methods/:fake_id` | 404 Not Found | Negative |
| MFA-MGT-004 | Delete another user's method | DELETE | `/api/mfa/methods/:other_id` | 403 Forbidden | Security |
| MFA-MGT-005 | Regenerate backup codes | POST | `/api/mfa/backup-codes/regenerate` | 200 OK, new codes | Positive |
| MFA-MGT-006 | Regenerate backup codes without auth | POST | `/api/mfa/backup-codes/regenerate` | 401 Unauthorized | Negative |

### 5.4 MFA Login Flow

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| MFA-FLOW-001 | Login with MFA-enabled account returns mfa_required | POST | `/api/auth/login` | 200 with mfa_required + mfa_token | Positive |
| MFA-FLOW-002 | Complete MFA with valid TOTP code | POST | `/api/mfa/verify-with-token/totp/:id` | 200 OK, access_token | Positive |
| MFA-FLOW-003 | Complete MFA with wrong TOTP code | POST | `/api/mfa/verify-with-token/totp/:id` | 400 Invalid code | Negative |
| MFA-FLOW-004 | Complete MFA with backup code | POST | `/api/mfa/verify-with-token/totp/:id` | 200 OK (backup code accepted) | Positive |
| MFA-FLOW-005 | Use same backup code twice | POST | `/api/mfa/verify-with-token/totp/:id` | 400 Code already used | Security |
| MFA-FLOW-006 | MFA token expires after timeout | POST | `/api/mfa/verify-with-token/totp/:id` | 401 Token expired | Negative |

---

## 6. Passwordless Authentication

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| PWLESS-001 | Start magic link flow with valid email | POST | `/api/tenants/:tid/passwordless/start` | 200 OK, email sent | Positive |
| PWLESS-002 | Start OTP flow with valid email | POST | `/api/tenants/:tid/passwordless/start` | 200 OK, OTP sent | Positive |
| PWLESS-003 | Verify magic link token | POST | `/api/tenants/:tid/passwordless/verify` | 200 OK, access_token | Positive |
| PWLESS-004 | Verify OTP code | POST | `/api/tenants/:tid/passwordless/verify` | 200 OK, access_token | Positive |
| PWLESS-005 | Verify with wrong OTP | POST | `/api/tenants/:tid/passwordless/verify` | 400 Invalid code | Negative |
| PWLESS-006 | Verify with expired token | POST | `/api/tenants/:tid/passwordless/verify` | 400 Expired | Negative |
| PWLESS-007 | Resend passwordless token | POST | `/api/tenants/:tid/passwordless/resend` | 200 OK | Positive |
| PWLESS-008 | Start with non-existent tenant | POST | `/api/tenants/:fake/passwordless/start` | 404 Not Found | Negative |
| PWLESS-009 | Start with invalid email format | POST | `/api/tenants/:tid/passwordless/start` | 400 Bad Request | Negative |
| PWLESS-010 | Start passwordless for unregistered email (auto-create?) | POST | `/api/tenants/:tid/passwordless/start` | Depends on config | Boundary |

---

## 7. OAuth2/OIDC Server

### 7.1 Discovery

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| OAUTH-DISC-001 | Get OpenID Configuration | GET | `/.well-known/openid-configuration` | 200 OK, valid OIDC discovery doc | Positive |
| OAUTH-DISC-002 | Verify discovery doc contains required fields | GET | `/.well-known/openid-configuration` | issuer, authorization_endpoint, token_endpoint, jwks_uri present | Positive |
| OAUTH-DISC-003 | Get JWKS | GET | `/.well-known/jwks.json` | 200 OK, valid JWK Set | Positive |
| OAUTH-DISC-004 | JWKS contains at least one key | GET | `/.well-known/jwks.json` | keys array non-empty | Positive |

### 7.2 Authorization Code Flow

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| OAUTH-AUTH-001 | Authorize with valid client_id, redirect_uri, response_type=code | GET | `/authorize` | 302 Redirect to login or consent | Positive |
| OAUTH-AUTH-002 | Authorize with PKCE (code_challenge, code_challenge_method=S256) | GET | `/authorize` | 302 Redirect with code | Positive |
| OAUTH-AUTH-003 | Authorize with invalid client_id | GET | `/authorize` | 400 Invalid client | Negative |
| OAUTH-AUTH-004 | Authorize with invalid redirect_uri | GET | `/authorize` | 400 Invalid redirect_uri | Negative |
| OAUTH-AUTH-005 | Authorize with unsupported response_type | GET | `/authorize` | 400 Unsupported response type | Negative |
| OAUTH-AUTH-006 | Authorize with organization scope | GET | `/authorize` | Token includes org context | Positive |
| OAUTH-AUTH-007 | Authorize with invalid scope | GET | `/authorize` | 400 Invalid scope | Negative |

### 7.3 Token Exchange

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| OAUTH-TOKEN-001 | Exchange valid authorization code for tokens | POST | `/oauth/token` | 200 OK, access_token + id_token | Positive |
| OAUTH-TOKEN-002 | Exchange code with PKCE code_verifier | POST | `/oauth/token` | 200 OK | Positive |
| OAUTH-TOKEN-003 | Exchange code with wrong code_verifier | POST | `/oauth/token` | 400 Invalid code verifier | Negative |
| OAUTH-TOKEN-004 | Exchange expired authorization code | POST | `/oauth/token` | 400 Code expired | Negative |
| OAUTH-TOKEN-005 | Exchange code twice (replay) | POST | `/oauth/token` | 400 Code already used | Security |
| OAUTH-TOKEN-006 | Client credentials grant | POST | `/oauth/token` | 200 OK, access_token | Positive |
| OAUTH-TOKEN-007 | Client credentials with wrong secret | POST | `/oauth/token` | 401 Unauthorized | Negative |
| OAUTH-TOKEN-008 | Refresh token grant | POST | `/oauth/token` | 200 OK, new tokens | Positive |
| OAUTH-TOKEN-009 | Token with unsupported grant_type | POST | `/oauth/token` | 400 Unsupported grant type | Negative |

### 7.4 Token Operations

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| OAUTH-OPS-001 | Get userinfo with valid access_token | GET | `/userinfo` | 200 OK, user claims | Positive |
| OAUTH-OPS-002 | Get userinfo without token | GET | `/userinfo` | 401 Unauthorized | Negative |
| OAUTH-OPS-003 | Revoke access_token | POST | `/oauth/revoke` | 200 OK | Positive |
| OAUTH-OPS-004 | Introspect valid token | POST | `/oauth/introspect` | 200 OK, active=true | Positive |
| OAUTH-OPS-005 | Introspect revoked token | POST | `/oauth/introspect` | 200 OK, active=false | Positive |
| OAUTH-OPS-006 | Introspect expired token | POST | `/oauth/introspect` | 200 OK, active=false | Positive |
| OAUTH-OPS-007 | Logout endpoint clears session | GET | `/logout` | 302 Redirect to logged-out | Positive |

---

## 8. Universal Login

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| UL-001 | Render login page | GET | `/login` | 200 OK, HTML with login form | Positive |
| UL-002 | Submit login form with valid credentials | POST | `/login` | 302 Redirect to consent/callback | Positive |
| UL-003 | Submit login form with invalid credentials | POST | `/login` | 200 with error message | Negative |
| UL-004 | Render signup page | GET | `/signup` | 200 OK, HTML with signup form | Positive |
| UL-005 | Submit signup form | POST | `/signup` | 302 Redirect or verification needed | Positive |
| UL-006 | Render MFA page | GET | `/mfa` | 200 OK, HTML with MFA input | Positive |
| UL-007 | Submit MFA verification | POST | `/mfa/verify` | 302 Redirect on success | Positive |
| UL-008 | Render consent page | GET | `/consent` | 200 OK, HTML with scopes | Positive |
| UL-009 | Submit consent (approve) | POST | `/consent` | 302 Redirect with code | Positive |
| UL-010 | Submit consent (deny) | POST | `/consent` | 302 Redirect with error=access_denied | Positive |
| UL-011 | Render logged-out page | GET | `/logged-out` | 200 OK, confirmation HTML | Positive |
| UL-012 | Render email verification page | GET | `/verify-email` | 200 OK, HTML | Positive |

---

## 9. Self-Service Flows (Headless API)

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SS-001 | Create login flow (browser) | GET | `/self-service/login/browser` | 200 OK, flow object with flow_id | Positive |
| SS-002 | Create login flow (API/SPA) | GET | `/self-service/login/api` | 200 OK, flow object | Positive |
| SS-003 | Get login flow state | GET | `/self-service/login?flow=xxx` | 200 OK, current state | Positive |
| SS-004 | Submit login flow with credentials | POST | `/self-service/login?flow=xxx` | 200 OK, session | Positive |
| SS-005 | Submit login flow with wrong credentials | POST | `/self-service/login?flow=xxx` | 400, error in flow | Negative |
| SS-006 | Create registration flow (browser) | GET | `/self-service/registration/browser` | 200 OK, flow object | Positive |
| SS-007 | Create registration flow (API) | GET | `/self-service/registration/api` | 200 OK | Positive |
| SS-008 | Get registration flow state | GET | `/self-service/registration?flow=xxx` | 200 OK | Positive |
| SS-009 | Submit registration flow | POST | `/self-service/registration?flow=xxx` | 200 OK, user created | Positive |
| SS-010 | Whoami with valid session | GET | `/sessions/whoami` | 200 OK, session + identity | Positive |
| SS-011 | Whoami without session | GET | `/sessions/whoami` | 401 Unauthorized | Negative |
| SS-012 | Get expired flow | GET | `/self-service/login?flow=expired` | 410 Flow expired | Negative |
| SS-013 | Get non-existent flow | GET | `/self-service/login?flow=fake` | 404 Not Found | Negative |

---

## 10. EntraID/Azure AD Integration

### 10.1 Corerun Tenant EntraID

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| ENTRA-CR-001 | Create EntraID OIDC provider for corerun tenant | POST | `/api/oidc/providers` | 201 Created, provider with azuread type | Positive |
| ENTRA-CR-002 | Verify EntraID provider listed in public providers | GET | `/api/oidc/providers/public?tenant_id=CORERUN_TID` | Provider visible with correct name | Positive |
| ENTRA-CR-003 | Initiate EntraID login for corerun | GET | `/api/oidc/login?tenant_id=CORERUN_TID&provider_id=xxx` | 302 Redirect to login.microsoftonline.com | Positive |
| ENTRA-CR-004 | Verify redirect URL contains correct Azure tenant ID (4ef69c55...) | GET | `/api/oidc/login` | URL contains /4ef69c55-ad51-453a-8538-449356a6c6c6/ | Positive |
| ENTRA-CR-005 | Verify redirect URL contains correct client_id | GET | `/api/oidc/login` | URL contains client_id=44c462b5-fc20-4de0-8c23-fcec7516435c | Positive |
| ENTRA-CR-006 | Verify PKCE code_challenge is included | GET | `/api/oidc/login` | URL contains code_challenge param | Positive |
| ENTRA-CR-007 | Verify scopes include openid profile email | GET | `/api/oidc/login` | scope=openid+profile+email | Positive |
| ENTRA-CR-008 | Handle EntraID callback with valid code | GET | `/api/oidc/callback` | 302 Redirect, user created/logged in | Positive |
| ENTRA-CR-009 | Handle EntraID callback with invalid code | GET | `/api/oidc/callback` | Error, invalid_grant | Negative |
| ENTRA-CR-010 | Handle EntraID callback with invalid state | GET | `/api/oidc/callback` | 400 Invalid state (CSRF) | Security |
| ENTRA-CR-011 | Verify user created from EntraID has correct email | - | - | User email matches Azure AD email | Positive |
| ENTRA-CR-012 | Verify Azure AD group claim extraction | - | - | Groups extracted from ID token | Positive |
| ENTRA-CR-013 | Verify admin group mapping (f80af0fc...) grants admin role | - | - | User in admin group gets admin role | Positive |
| ENTRA-CR-014 | User not in allowed group denied access | GET | `/api/oidc/callback` | 403 Forbidden (group filter) | Negative |
| ENTRA-CR-015 | Azure AD email from preferred_username claim | - | - | Email extracted correctly | Positive |
| ENTRA-CR-016 | Azure AD email from upn claim (fallback) | - | - | Email extracted from UPN | Positive |
| ENTRA-CR-017 | Azure AD user profile sync on login | - | - | Name, email updated from claims | Positive |
| ENTRA-CR-018 | EntraID login creates tenant membership | - | - | User added to corerun tenant | Positive |

### 10.2 IMYS Tenant EntraID

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| ENTRA-IM-001 | Create EntraID OIDC provider for imys tenant | POST | `/api/oidc/providers` | 201 Created | Positive |
| ENTRA-IM-002 | Verify imys uses correct Azure tenant (c9af7ec0...) | GET | `/api/oidc/login` | URL contains correct tenant ID | Positive |
| ENTRA-IM-003 | Verify imys uses correct client_id (b6e6a0ae...) | GET | `/api/oidc/login` | Correct client_id in URL | Positive |
| ENTRA-IM-004 | Verify issuer URL matches config | - | - | Token issuer = https://login.microsoftonline.com/c9af7ec0.../v2.0 | Positive |
| ENTRA-IM-005 | Admin group mapping (0d35f722...) | - | - | Correct role assigned | Positive |
| ENTRA-IM-006 | User group mapping (28b11038...) | - | - | User role assigned | Positive |
| ENTRA-IM-007 | User in admin group gets admin, not just user role | - | - | Highest role wins | Positive |
| ENTRA-IM-008 | User in neither group denied or gets default role | - | - | Depends on config | Boundary |

### 10.3 Cross-Tenant EntraID Isolation

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| ENTRA-ISO-001 | Corerun EntraID user cannot access imys tenant | - | - | 403 Forbidden | Security |
| ENTRA-ISO-002 | IMYS EntraID user cannot access corerun tenant | - | - | 403 Forbidden | Security |
| ENTRA-ISO-003 | EntraID provider from tenant A not visible to tenant B | GET | `/api/oidc/providers/public` | Only own tenant's providers | Security |
| ENTRA-ISO-004 | Cannot delete EntraID provider from wrong tenant | DELETE | `/api/oidc/providers/:id` | 404 or 403 | Security |
| ENTRA-ISO-005 | SSO discovery returns correct provider per email domain | GET | `/api/oidc/sso-check?email=user@domain` | Returns matching provider | Positive |

### 10.4 EntraID Edge Cases

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| ENTRA-EDGE-001 | EntraID token with no email claim | - | - | Fallback to preferred_username/upn | Boundary |
| ENTRA-EDGE-002 | EntraID token with no groups claim | - | - | User gets default role | Boundary |
| ENTRA-EDGE-003 | EntraID user already exists (re-login) | - | - | Profile synced, no duplicate | Positive |
| ENTRA-EDGE-004 | EntraID provider disabled | GET | `/api/oidc/login` | 400/404 Provider disabled | Negative |
| ENTRA-EDGE-005 | EntraID with expired/revoked client secret | - | - | Token exchange fails, user-friendly error | Negative |
| ENTRA-EDGE-006 | EntraID with wrong JWKS URI | - | - | Token validation fails | Negative |
| ENTRA-EDGE-007 | EntraID with mismatched issuer | - | - | Token rejected (issuer mismatch) | Security |
| ENTRA-EDGE-008 | Create provider with invalid Azure tenant ID | POST | `/api/oidc/providers` | Validation error or discovery fails | Negative |
| ENTRA-EDGE-009 | Create provider with empty client_secret | POST | `/api/oidc/providers` | 400 Bad Request | Negative |
| ENTRA-EDGE-010 | EntraID callback with tampered ID token | GET | `/api/oidc/callback` | 401 Signature verification failed | Security |

---

## 11. Social Login & OIDC Providers

### 11.1 OIDC Provider Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| OIDC-MGT-001 | Create OIDC provider (Google) | POST | `/api/oidc/providers` | 201 Created | Positive |
| OIDC-MGT-002 | Create OIDC provider (Okta) | POST | `/api/oidc/providers` | 201 Created | Positive |
| OIDC-MGT-003 | Create OIDC provider (custom) | POST | `/api/oidc/providers` | 201 Created | Positive |
| OIDC-MGT-004 | Create provider with missing required fields | POST | `/api/oidc/providers` | 400 Bad Request | Negative |
| OIDC-MGT-005 | Create provider without tenant admin role | POST | `/api/oidc/providers` | 403 Forbidden | Security |
| OIDC-MGT-006 | List providers (authenticated) | GET | `/api/oidc/providers` | 200 OK, array | Positive |
| OIDC-MGT-007 | List public providers | GET | `/api/oidc/providers/public?tenant_id=xxx` | 200 OK, only enabled | Positive |
| OIDC-MGT-008 | Update provider (enable/disable) | PATCH | `/api/oidc/providers/:id` | 200 OK | Positive |
| OIDC-MGT-009 | Delete provider | DELETE | `/api/oidc/providers/:id` | 200 OK | Positive |
| OIDC-MGT-010 | Delete non-existent provider | DELETE | `/api/oidc/providers/:fake` | 404 Not Found | Negative |
| OIDC-MGT-011 | Delete provider from wrong tenant | DELETE | `/api/oidc/providers/:id` | 403/404 | Security |

### 11.2 OIDC Templates

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| OIDC-TPL-001 | List all provider templates | GET | `/api/oidc/templates` | 200 OK, includes azuread, google, okta, auth0 | Positive |
| OIDC-TPL-002 | Get Azure AD template | GET | `/api/oidc/templates/azuread` | 200 OK, template with Azure endpoints | Positive |
| OIDC-TPL-003 | Get non-existent template | GET | `/api/oidc/templates/nonexistent` | 404 Not Found | Negative |

### 11.3 SSO Discovery

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SSO-DISC-001 | Check SSO for email with configured SSO | GET | `/api/oidc/sso-check?email=user@company.com` | 200 OK, sso_enabled=true, provider info | Positive |
| SSO-DISC-002 | Check SSO for email without SSO | GET | `/api/oidc/sso-check?email=user@gmail.com` | 200 OK, sso_enabled=false | Positive |
| SSO-DISC-003 | Check SSO with invalid email | GET | `/api/oidc/sso-check?email=invalid` | 400 Bad Request | Negative |
| SSO-DISC-004 | Check SSO with missing email param | GET | `/api/oidc/sso-check` | 400 Bad Request | Negative |

### 11.4 Social Login Flow

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SOCIAL-001 | Initiate social login (Google) | GET | `/login/social/:connection_id` | 302 Redirect to Google OAuth | Positive |
| SOCIAL-002 | Social login callback with valid code | GET | `/login/social/callback` | 302, user authenticated | Positive |
| SOCIAL-003 | Social login callback with error param | GET | `/login/social/callback?error=access_denied` | Error page rendered | Negative |
| SOCIAL-004 | Social login with invalid connection_id | GET | `/login/social/:fake_id` | 404 Not Found | Negative |
| SOCIAL-005 | Social login links to existing user (same email) | GET | `/login/social/callback` | Linked, no duplicate | Positive |
| SOCIAL-006 | Social login callback with CSRF state mismatch | GET | `/login/social/callback` | 400 Invalid state | Security |

---

## 12. Connection Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CONN-001 | List organization connections | GET | `/api/organizations/:org_id/connections` | 200 OK, array | Positive |
| CONN-002 | Create OIDC connection | POST | `/api/organizations/:org_id/connections` | 201 Created | Positive |
| CONN-003 | Create database connection | POST | `/api/organizations/:org_id/connections` | 201 Created | Positive |
| CONN-004 | Create social connection | POST | `/api/organizations/:org_id/connections` | 201 Created | Positive |
| CONN-005 | Create connection with invalid type | POST | `/api/organizations/:org_id/connections` | 400 Bad Request | Negative |
| CONN-006 | Create connection without admin role | POST | `/api/organizations/:org_id/connections` | 403 Forbidden | Security |
| CONN-007 | Get connection details | GET | `/api/organizations/:org_id/connections/:conn_id` | 200 OK | Positive |
| CONN-008 | Update connection config | PUT | `/api/organizations/:org_id/connections/:conn_id` | 200 OK | Positive |
| CONN-009 | Delete connection | DELETE | `/api/organizations/:org_id/connections/:conn_id` | 200 OK | Positive |
| CONN-010 | Get connection from wrong org | GET | `/api/organizations/:wrong_org/connections/:conn_id` | 404 | Security |
| CONN-011 | Delete connection from wrong org | DELETE | `/api/organizations/:wrong_org/connections/:conn_id` | 404 | Security |
| CONN-012 | Get available auth methods | GET | `/api/organizations/:org_id/connections/auth-methods` | 200 OK, method list | Positive |
| CONN-013 | List all platform connections (admin) | GET | `/api/admin/connections` | 200 OK | Positive |
| CONN-014 | Create platform connection (admin) | POST | `/api/admin/connections` | 201 Created | Positive |
| CONN-015 | List connections without auth | GET | `/api/organizations/:org_id/connections` | 401 Unauthorized | Negative |
| CONN-016 | Create connection with missing required config | POST | `/api/organizations/:org_id/connections` | 400 validation error | Negative |
| CONN-017 | Create duplicate connection name | POST | `/api/organizations/:org_id/connections` | 409 Conflict | Negative |

---

## 13. Tenant & Organization Management

### 13.1 Tenant Creation

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| TENANT-001 | Create new tenant with admin user | POST | `/api/tenants` | 201 Created, tenant + admin user | Positive |
| TENANT-002 | Create tenant with duplicate name/slug | POST | `/api/tenants` | 409 Conflict | Negative |
| TENANT-003 | Create tenant with missing required fields | POST | `/api/tenants` | 400 Bad Request | Negative |
| TENANT-004 | Create tenant with invalid slug (uppercase, special chars) | POST | `/api/tenants` | 400 Invalid slug | Negative |
| TENANT-005 | Get organization by slug | GET | `/api/organizations/by-slug/:slug` | 200 OK, org info | Positive |
| TENANT-006 | Get organization by non-existent slug | GET | `/api/organizations/by-slug/:fake` | 404 Not Found | Negative |

### 13.2 Tenant User Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| TUSER-001 | List tenant users (as admin) | GET | `/api/tenants/:tid/users` | 200 OK, user array | Positive |
| TUSER-002 | List tenant users (as regular user) | GET | `/api/tenants/:tid/users` | 403 Forbidden | Security |
| TUSER-003 | Update user role to admin | PUT | `/api/tenants/:tid/users/:uid/role` | 200 OK | Positive |
| TUSER-004 | Update user role to member | PUT | `/api/tenants/:tid/users/:uid/role` | 200 OK | Positive |
| TUSER-005 | Update own role (demote self from admin) | PUT | `/api/tenants/:tid/users/:own_uid/role` | 400 or 200 (risky) | Boundary |
| TUSER-006 | Update role for user in different tenant | PUT | `/api/tenants/:tid/users/:other_uid/role` | 404 | Security |
| TUSER-007 | Update role with invalid role value | PUT | `/api/tenants/:tid/users/:uid/role` | 400 Bad Request | Negative |
| TUSER-008 | List users from wrong tenant (cross-tenant) | GET | `/api/tenants/:other_tid/users` | 403 Forbidden | Security |

---

## 14. Invitation System

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| INV-001 | Create invitation with valid email | POST | `/api/tenants/:tid/invitations` | 201 Created, invitation sent | Positive |
| INV-002 | Create invitation with already-member email | POST | `/api/tenants/:tid/invitations` | 409 Conflict or 400 | Negative |
| INV-003 | Create invitation without admin role | POST | `/api/tenants/:tid/invitations` | 403 Forbidden | Security |
| INV-004 | Create invitation with invalid email | POST | `/api/tenants/:tid/invitations` | 400 Bad Request | Negative |
| INV-005 | List invitations (as admin) | GET | `/api/tenants/:tid/invitations` | 200 OK, array | Positive |
| INV-006 | Verify invitation token (public) | GET | `/api/invitations/verify?token=xxx` | 200 OK, invitation details | Positive |
| INV-007 | Verify expired invitation | GET | `/api/invitations/verify?token=expired` | 400 Expired | Negative |
| INV-008 | Verify invalid invitation token | GET | `/api/invitations/verify?token=fake` | 400 Invalid | Negative |
| INV-009 | Accept invitation (new user signup) | POST | `/api/invitations/accept` | 201, user created + joined tenant | Positive |
| INV-010 | Accept invitation (existing user) | POST | `/api/invitations/accept` | 200 OK, joined tenant | Positive |
| INV-011 | Accept already-accepted invitation | POST | `/api/invitations/accept` | 400 Already accepted | Negative |
| INV-012 | Revoke invitation | DELETE | `/api/tenants/:tid/invitations/:inv_id` | 200 OK | Positive |
| INV-013 | Revoke already-accepted invitation | DELETE | `/api/tenants/:tid/invitations/:inv_id` | 400 Already accepted | Negative |
| INV-014 | Resend invitation | POST | `/api/tenants/:tid/invitations/:inv_id/resend` | 200 OK, email resent | Positive |
| INV-015 | Accept revoked invitation | POST | `/api/invitations/accept` | 400 Invitation revoked | Negative |
| INV-016 | Create duplicate invitation for same email | POST | `/api/tenants/:tid/invitations` | 409 or replace existing | Boundary |

---

## 15. Groups Management

### 15.1 Group CRUD

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| GRP-001 | Create group | POST | `/api/tenants/:tid/groups` | 201 Created | Positive |
| GRP-002 | Create group with duplicate name | POST | `/api/tenants/:tid/groups` | 409 Conflict | Negative |
| GRP-003 | Create group without admin | POST | `/api/tenants/:tid/groups` | 403 Forbidden | Security |
| GRP-004 | List groups | GET | `/api/tenants/:tid/groups` | 200 OK, array | Positive |
| GRP-005 | Get group by ID | GET | `/api/tenants/:tid/groups/:gid` | 200 OK, group details | Positive |
| GRP-006 | Get non-existent group | GET | `/api/tenants/:tid/groups/:fake` | 404 Not Found | Negative |
| GRP-007 | Update group name | PUT | `/api/tenants/:tid/groups/:gid` | 200 OK | Positive |
| GRP-008 | Delete group | DELETE | `/api/tenants/:tid/groups/:gid` | 200 OK | Positive |
| GRP-009 | Delete group with members | DELETE | `/api/tenants/:tid/groups/:gid` | 200 OK (cascade) or 400 | Boundary |
| GRP-010 | Access group from wrong tenant | GET | `/api/tenants/:wrong/groups/:gid` | 404 | Security |

### 15.2 Group Members

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| GRPMEM-001 | Add member to group | POST | `/api/tenants/:tid/groups/:gid/members` | 200 OK | Positive |
| GRPMEM-002 | Add non-existent user to group | POST | `/api/tenants/:tid/groups/:gid/members` | 404 User not found | Negative |
| GRPMEM-003 | Add duplicate member | POST | `/api/tenants/:tid/groups/:gid/members` | 409 Conflict | Negative |
| GRPMEM-004 | List group members | GET | `/api/tenants/:tid/groups/:gid/members` | 200 OK, member array | Positive |
| GRPMEM-005 | Update member role in group | PATCH | `/api/tenants/:tid/groups/:gid/members/:uid` | 200 OK | Positive |
| GRPMEM-006 | Remove member from group | DELETE | `/api/tenants/:tid/groups/:gid/members/:uid` | 200 OK | Positive |
| GRPMEM-007 | Remove non-member from group | DELETE | `/api/tenants/:tid/groups/:gid/members/:uid` | 404 | Negative |
| GRPMEM-008 | Get user's groups | GET | `/api/tenants/:tid/users/:uid/groups` | 200 OK, group array | Positive |

### 15.3 Group Roles

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| GRPROLE-001 | Assign role to group | POST | `/api/tenants/:tid/groups/:gid/roles` | 200 OK | Positive |
| GRPROLE-002 | List group roles | GET | `/api/tenants/:tid/groups/:gid/roles` | 200 OK, role array | Positive |
| GRPROLE-003 | Remove role from group | DELETE | `/api/tenants/:tid/groups/:gid/roles/:rid` | 200 OK | Positive |
| GRPROLE-004 | Assign invalid role | POST | `/api/tenants/:tid/groups/:gid/roles` | 400 Bad Request | Negative |
| GRPROLE-005 | Assign duplicate role | POST | `/api/tenants/:tid/groups/:gid/roles` | 409 Conflict | Negative |

---

## 16. Session Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SESS-001 | List active sessions | GET | `/api/sessions` | 200 OK, session array with device/IP info | Positive |
| SESS-002 | Revoke specific session | DELETE | `/api/sessions/:session_id` | 200 OK, session invalidated | Positive |
| SESS-003 | Revoke all sessions | POST | `/api/sessions/revoke-all` | 200 OK, all sessions cleared | Positive |
| SESS-004 | Revoke session from another user | DELETE | `/api/sessions/:other_session` | 403 Forbidden | Security |
| SESS-005 | Revoke non-existent session | DELETE | `/api/sessions/:fake` | 404 Not Found | Negative |
| SESS-006 | List sessions without auth | GET | `/api/sessions` | 401 Unauthorized | Negative |
| SESS-007 | Get login history | GET | `/api/login-history` | 200 OK, login events with timestamps | Positive |
| SESS-008 | Get security audit logs | GET | `/api/security/audit-logs` | 200 OK, security events | Positive |
| SESS-009 | Verify revoked session cannot access APIs | GET | `/api/auth/me` | 401 after session revoke | Security |
| SESS-010 | Verify revoke-all invalidates current session too | GET | `/api/auth/me` | 401 after revoke-all | Security |

---

## 17. OAuth Application Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| OAPP-001 | Create OAuth application | POST | `/api/organizations/:org_id/applications` | 201 Created, client_id + client_secret | Positive |
| OAPP-002 | Create app with missing name | POST | `/api/organizations/:org_id/applications` | 400 Bad Request | Negative |
| OAPP-003 | Create app without admin role | POST | `/api/organizations/:org_id/applications` | 403 Forbidden | Security |
| OAPP-004 | List applications | GET | `/api/organizations/:org_id/applications` | 200 OK, array | Positive |
| OAPP-005 | Get application details | GET | `/api/organizations/:org_id/applications/:app_id` | 200 OK | Positive |
| OAPP-006 | Update application (name, redirect_uris) | PUT | `/api/organizations/:org_id/applications/:app_id` | 200 OK | Positive |
| OAPP-007 | Rotate client secret | POST | `/api/organizations/:org_id/applications/:app_id/rotate-secret` | 200 OK, new secret | Positive |
| OAPP-008 | Verify old secret invalid after rotation | POST | `/oauth/token` | 401 with old secret | Security |
| OAPP-009 | Delete application | DELETE | `/api/organizations/:org_id/applications/:app_id` | 200 OK | Positive |
| OAPP-010 | Delete non-existent app | DELETE | `/api/organizations/:org_id/applications/:fake` | 404 | Negative |
| OAPP-011 | Access app from wrong org | GET | `/api/organizations/:wrong/applications/:app_id` | 404 | Security |
| OAPP-012 | Authenticate application (client credentials) | POST | `/api/applications/authenticate` | 200 OK, token | Positive |
| OAPP-013 | Authenticate app with wrong secret | POST | `/api/applications/authenticate` | 401 Unauthorized | Negative |

---

## 18. Fine-Grained Authorization (FGA)

### 18.1 Relation Tuples

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| FGA-TUP-001 | Create relation tuple (user:alice is viewer of doc:1) | POST | `/api/authz/tuples` | 200 OK | Positive |
| FGA-TUP-002 | Create tuple with missing fields | POST | `/api/authz/tuples` | 400 Bad Request | Negative |
| FGA-TUP-003 | Create duplicate tuple | POST | `/api/authz/tuples` | 409 Conflict | Negative |
| FGA-TUP-004 | Delete tuple | DELETE | `/api/authz/tuples` | 200 OK | Positive |
| FGA-TUP-005 | Delete non-existent tuple | DELETE | `/api/authz/tuples` | 404 | Negative |
| FGA-TUP-006 | Query tuples by object | POST | `/api/authz/tuples/query` | 200 OK, matching tuples | Positive |
| FGA-TUP-007 | Query tuples by subject | POST | `/api/authz/tuples/query` | 200 OK | Positive |
| FGA-TUP-008 | Get object tuples | GET | `/api/authz/tuples/by-object/:tid/:ns/:oid` | 200 OK | Positive |
| FGA-TUP-009 | Get subject tuples | GET | `/api/authz/tuples/by-subject/:tid/:st/:sid` | 200 OK | Positive |
| FGA-TUP-010 | Create tuple without auth | POST | `/api/authz/tuples` | 401 Unauthorized | Negative |
| FGA-TUP-011 | Cross-tenant tuple access | GET | `/api/authz/tuples/by-object/:wrong_tid/:ns/:oid` | Empty or 403 | Security |

### 18.2 Permission Checks

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| FGA-CHK-001 | Check direct permission (allowed) | POST | `/api/authz/check` | 200 OK, allowed=true | Positive |
| FGA-CHK-002 | Check direct permission (denied) | POST | `/api/authz/check` | 200 OK, allowed=false | Positive |
| FGA-CHK-003 | Check inherited permission (role hierarchy: owner has viewer) | POST | `/api/authz/check` | 200 OK, allowed=true | Positive |
| FGA-CHK-004 | Check cross-type inheritance (workspace admin → resource owner) | POST | `/api/authz/check` | 200 OK, allowed=true | Positive |
| FGA-CHK-005 | Check group-based permission | POST | `/api/authz/check` | 200 OK, allowed=true | Positive |
| FGA-CHK-006 | Check with missing fields | POST | `/api/authz/check` | 400 Bad Request | Negative |
| FGA-CHK-007 | Check without auth | POST | `/api/authz/check` | 401 Unauthorized | Negative |
| FGA-CHK-008 | Expand relation tree | GET | `/api/authz/expand/:tid/:ns/:oid/:rel` | 200 OK, expansion tree | Positive |
| FGA-CHK-009 | Check permission after tuple deleted | POST | `/api/authz/check` | allowed=false | Security |
| FGA-CHK-010 | Check permission with non-existent namespace | POST | `/api/authz/check` | 400 or allowed=false | Negative |

### 18.3 Forward Auth

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| FGA-FWD-001 | Forward auth check (allowed) | POST | `/authz/forward-auth` | 200 OK | Positive |
| FGA-FWD-002 | Forward auth check (denied) | POST | `/authz/forward-auth` | 403 Forbidden | Positive |
| FGA-FWD-003 | Forward auth GET variant | GET | `/authz/forward-auth` | 200 or 403 | Positive |
| FGA-FWD-004 | Forward auth with missing headers | POST | `/authz/forward-auth` | 400 Bad Request | Negative |

---

## 19. FGA Stores & Models

### 19.1 Store Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| STORE-001 | Create FGA store | POST | `/api/fga/stores` | 201 Created, store_id | Positive |
| STORE-002 | Create store with duplicate name | POST | `/api/fga/stores` | 409 Conflict | Negative |
| STORE-003 | Create store without auth | POST | `/api/fga/stores` | 401 Unauthorized | Negative |
| STORE-004 | List FGA stores | GET | `/api/fga/stores` | 200 OK, store array | Positive |
| STORE-005 | Get store by ID | GET | `/api/fga/stores/:store_id` | 200 OK | Positive |
| STORE-006 | Get non-existent store | GET | `/api/fga/stores/:fake` | 404 | Negative |
| STORE-007 | Update store name | PATCH | `/api/fga/stores/:store_id` | 200 OK | Positive |
| STORE-008 | Delete store | DELETE | `/api/fga/stores/:store_id` | 200 OK | Positive |
| STORE-009 | Delete store with existing tuples | DELETE | `/api/fga/stores/:store_id` | 200 (cascade) or 400 | Boundary |

### 19.2 Authorization Models

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| MODEL-001 | Write authorization model | POST | `/api/fga/stores/:sid/models` | 201 Created, version | Positive |
| MODEL-002 | Write model with invalid schema | POST | `/api/fga/stores/:sid/models` | 400 Bad Request | Negative |
| MODEL-003 | Get current model | GET | `/api/fga/stores/:sid/models/current` | 200 OK, model schema | Positive |
| MODEL-004 | Get model by version | GET | `/api/fga/stores/:sid/models/:version` | 200 OK | Positive |
| MODEL-005 | Get non-existent model version | GET | `/api/fga/stores/:sid/models/:fake` | 404 | Negative |
| MODEL-006 | List model versions | GET | `/api/fga/stores/:sid/models` | 200 OK, version array | Positive |
| MODEL-007 | Write model with computedUserset relations | POST | `/api/fga/stores/:sid/models` | 201, role hierarchy works | Positive |
| MODEL-008 | Write model with tupleToUserset | POST | `/api/fga/stores/:sid/models` | 201, cross-type inheritance | Positive |
| MODEL-009 | Write model with union/intersection/exclusion | POST | `/api/fga/stores/:sid/models` | 201, complex relations | Positive |

### 19.3 FGA API Keys

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| FGAKEY-001 | Create API key for store | POST | `/api/fga/stores/:sid/api-keys` | 201 Created, key value | Positive |
| FGAKEY-002 | List API keys | GET | `/api/fga/stores/:sid/api-keys` | 200 OK, key array (masked) | Positive |
| FGAKEY-003 | Revoke API key | DELETE | `/api/fga/stores/:sid/api-keys/:kid` | 200 OK | Positive |
| FGAKEY-004 | Use revoked API key | POST | `/api/fga/stores/:sid/check` | 401 Unauthorized | Security |

### 19.4 Store Tuple Operations

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| STUP-001 | Write tuples to store | POST | `/api/fga/stores/:sid/tuples` | 200 OK | Positive |
| STUP-002 | Read tuples from store | GET | `/api/fga/stores/:sid/tuples` | 200 OK, tuple array | Positive |
| STUP-003 | Store-scoped permission check | POST | `/api/fga/stores/:sid/check` | 200 OK, result | Positive |
| STUP-004 | Write tuples to wrong store | POST | `/api/fga/stores/:wrong/tuples` | 404 | Negative |

---

## 20. Actions/Hooks

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| ACT-001 | Create action (post-login hook) | POST | `/api/organizations/:org_id/actions` | 201 Created | Positive |
| ACT-002 | Create action with missing fields | POST | `/api/organizations/:org_id/actions` | 400 Bad Request | Negative |
| ACT-003 | Create action without admin | POST | `/api/organizations/:org_id/actions` | 403 Forbidden | Security |
| ACT-004 | List actions | GET | `/api/organizations/:org_id/actions` | 200 OK, array | Positive |
| ACT-005 | Get action details | GET | `/api/organizations/:org_id/actions/:aid` | 200 OK | Positive |
| ACT-006 | Update action | PUT | `/api/organizations/:org_id/actions/:aid` | 200 OK | Positive |
| ACT-007 | Delete action | DELETE | `/api/organizations/:org_id/actions/:aid` | 200 OK | Positive |
| ACT-008 | Test action execution | POST | `/api/organizations/:org_id/actions/:aid/test` | 200 OK, execution result | Positive |
| ACT-009 | Test action with invalid webhook URL | POST | `/api/organizations/:org_id/actions/:aid/test` | Error (connection refused) | Negative |
| ACT-010 | Get action executions | GET | `/api/organizations/:org_id/actions/:aid/executions` | 200 OK, execution history | Positive |
| ACT-011 | Get org-wide executions | GET | `/api/organizations/:org_id/actions/executions` | 200 OK | Positive |
| ACT-012 | Get actions from wrong org | GET | `/api/organizations/:wrong/actions` | 403 | Security |

---

## 21. Webhooks

### 21.1 Webhook Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| WH-001 | Create webhook | POST | `/api/organizations/:org_id/webhooks` | 201 Created, includes secret | Positive |
| WH-002 | Create webhook with invalid URL | POST | `/api/organizations/:org_id/webhooks` | 400 Bad Request | Negative |
| WH-003 | Create webhook without admin | POST | `/api/organizations/:org_id/webhooks` | 403 Forbidden | Security |
| WH-004 | Create webhook with specific event types | POST | `/api/organizations/:org_id/webhooks` | 201, only subscribed events | Positive |
| WH-005 | List webhooks | GET | `/api/organizations/:org_id/webhooks` | 200 OK | Positive |
| WH-006 | Get webhook details | GET | `/api/organizations/:org_id/webhooks/:wid` | 200 OK | Positive |
| WH-007 | Update webhook URL | PUT | `/api/organizations/:org_id/webhooks/:wid` | 200 OK | Positive |
| WH-008 | Update webhook event subscriptions | PUT | `/api/organizations/:org_id/webhooks/:wid` | 200 OK | Positive |
| WH-009 | Delete webhook | DELETE | `/api/organizations/:org_id/webhooks/:wid` | 200 OK | Positive |
| WH-010 | Rotate webhook secret | POST | `/api/organizations/:org_id/webhooks/:wid/rotate-secret` | 200 OK, new secret | Positive |
| WH-011 | Test webhook delivery | POST | `/api/organizations/:org_id/webhooks/:wid/test` | 200 OK, test event sent | Positive |
| WH-012 | List webhook event types | GET | `/api/webhooks/event-types` | 200 OK, event list | Positive |
| WH-013 | Access webhook from wrong org | GET | `/api/organizations/:wrong/webhooks/:wid` | 404 | Security |

### 21.2 Webhook Deliveries

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| WHD-001 | List deliveries for webhook | GET | `/api/organizations/:org_id/webhooks/:wid/deliveries` | 200 OK | Positive |
| WHD-002 | Get specific delivery details | GET | `/api/organizations/:org_id/webhooks/:wid/deliveries/:did` | 200 OK, request/response | Positive |
| WHD-003 | Retry failed delivery | POST | `/api/organizations/:org_id/webhooks/:wid/deliveries/:did/retry` | 200 OK | Positive |
| WHD-004 | Retry successful delivery | POST | `/api/organizations/:org_id/webhooks/:wid/deliveries/:did/retry` | 200 OK (re-sent) | Boundary |
| WHD-005 | Verify webhook signature (HMAC) | - | - | Signature matches secret | Security |
| WHD-006 | Verify webhook payload includes correct event data | - | - | Event type + data present | Positive |

---

## 22. SCIM 2.0 Provisioning

### 22.1 SCIM Discovery

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SCIM-DISC-001 | Get service provider config | GET | `/scim/v2/ServiceProviderConfig` | 200 OK, SCIM config | Positive |
| SCIM-DISC-002 | Get resource types | GET | `/scim/v2/ResourceTypes` | 200 OK, User + Group | Positive |
| SCIM-DISC-003 | Get schemas | GET | `/scim/v2/Schemas` | 200 OK, SCIM schemas | Positive |

### 22.2 SCIM Users

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SCIM-USR-001 | Create SCIM user | POST | `/scim/v2/Users` | 201 Created, SCIM user | Positive |
| SCIM-USR-002 | Create user with duplicate userName | POST | `/scim/v2/Users` | 409 Conflict | Negative |
| SCIM-USR-003 | Create user with missing required fields | POST | `/scim/v2/Users` | 400 Bad Request | Negative |
| SCIM-USR-004 | List SCIM users | GET | `/scim/v2/Users` | 200 OK, ListResponse | Positive |
| SCIM-USR-005 | List users with email filter | GET | `/scim/v2/Users?filter=emails.value eq "x"` | 200 OK, filtered | Positive |
| SCIM-USR-006 | List users with externalId filter | GET | `/scim/v2/Users?filter=externalId eq "x"` | 200 OK, filtered | Positive |
| SCIM-USR-007 | List users with pagination (startIndex, count) | GET | `/scim/v2/Users?startIndex=1&count=10` | 200 OK, paginated | Positive |
| SCIM-USR-008 | Get SCIM user by ID | GET | `/scim/v2/Users/:uid` | 200 OK, SCIM user | Positive |
| SCIM-USR-009 | Get non-existent SCIM user | GET | `/scim/v2/Users/:fake` | 404 Not Found | Negative |
| SCIM-USR-010 | Replace SCIM user (PUT) | PUT | `/scim/v2/Users/:uid` | 200 OK, updated user | Positive |
| SCIM-USR-011 | Patch SCIM user (activate/deactivate) | PATCH | `/scim/v2/Users/:uid` | 200 OK | Positive |
| SCIM-USR-012 | Patch user with Replace operation | PATCH | `/scim/v2/Users/:uid` | 200 OK | Positive |
| SCIM-USR-013 | Patch user with Add operation | PATCH | `/scim/v2/Users/:uid` | 200 OK | Positive |
| SCIM-USR-014 | Delete SCIM user | DELETE | `/scim/v2/Users/:uid` | 204 No Content | Positive |
| SCIM-USR-015 | SCIM without bearer token | GET | `/scim/v2/Users` | 401 Unauthorized | Security |
| SCIM-USR-016 | SCIM with invalid bearer token | GET | `/scim/v2/Users` | 401 Unauthorized | Security |
| SCIM-USR-017 | SCIM with revoked token | GET | `/scim/v2/Users` | 401 Unauthorized | Security |

### 22.3 SCIM Groups

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SCIM-GRP-001 | Create SCIM group | POST | `/scim/v2/Groups` | 201 Created | Positive |
| SCIM-GRP-002 | Create group with members | POST | `/scim/v2/Groups` | 201, members linked | Positive |
| SCIM-GRP-003 | List SCIM groups | GET | `/scim/v2/Groups` | 200 OK, ListResponse | Positive |
| SCIM-GRP-004 | Get SCIM group | GET | `/scim/v2/Groups/:gid` | 200 OK, group with members | Positive |
| SCIM-GRP-005 | Patch group (add member) | PATCH | `/scim/v2/Groups/:gid` | 200 OK, member added | Positive |
| SCIM-GRP-006 | Patch group (remove member) | PATCH | `/scim/v2/Groups/:gid` | 200 OK, member removed | Positive |
| SCIM-GRP-007 | Delete SCIM group | DELETE | `/scim/v2/Groups/:gid` | 204 No Content | Positive |

### 22.4 SCIM Token Management

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SCIM-TOK-001 | Create SCIM token | POST | `/api/organizations/:org_id/scim/tokens` | 201 Created, token value | Positive |
| SCIM-TOK-002 | List SCIM tokens | GET | `/api/organizations/:org_id/scim/tokens` | 200 OK, masked tokens | Positive |
| SCIM-TOK-003 | Revoke SCIM token | DELETE | `/api/organizations/:org_id/scim/tokens/:tid` | 200 OK | Positive |
| SCIM-TOK-004 | Create token without admin | POST | `/api/organizations/:org_id/scim/tokens` | 403 Forbidden | Security |
| SCIM-TOK-005 | Token only shown once on creation | POST | `/api/organizations/:org_id/scim/tokens` | Token in response, not in list | Security |

---

## 23. Audit Logging

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| AUDIT-001 | Query audit logs (no filter) | GET | `/api/audit/logs` | 200 OK, recent logs | Positive |
| AUDIT-002 | Filter by event_types | GET | `/api/audit/logs?event_types=user.login_success` | 200 OK, filtered | Positive |
| AUDIT-003 | Filter by date range | GET | `/api/audit/logs?from=...&to=...` | 200 OK, date-filtered | Positive |
| AUDIT-004 | Filter by actor_id | GET | `/api/audit/logs?actor_id=xxx` | 200 OK, actor-filtered | Positive |
| AUDIT-005 | Get specific audit log | GET | `/api/audit/logs/:id` | 200 OK, full log entry | Positive |
| AUDIT-006 | Get non-existent audit log | GET | `/api/audit/logs/:fake` | 404 Not Found | Negative |
| AUDIT-007 | Get security events | GET | `/api/audit/security-events` | 200 OK, security events | Positive |
| AUDIT-008 | Get failed logins for user | GET | `/api/audit/failed-logins/:uid` | 200 OK, failed attempts | Positive |
| AUDIT-009 | Export audit logs | GET | `/api/audit/export` | 200 OK, export data (CSV/JSON) | Positive |
| AUDIT-010 | Get audit stats | GET | `/api/audit/stats` | 200 OK, statistics | Positive |
| AUDIT-011 | Audit logs without auth | GET | `/api/audit/logs` | 401 Unauthorized | Negative |
| AUDIT-012 | Verify login creates audit entry | POST + GET | `/api/auth/login` + `/api/audit/logs` | Login event recorded | Positive |
| AUDIT-013 | Verify registration creates audit entry | POST + GET | `/api/auth/register` + `/api/audit/logs` | Registration recorded | Positive |
| AUDIT-014 | Verify password change creates audit entry | - | - | Password change recorded | Positive |
| AUDIT-015 | Verify failed login creates audit entry | - | - | Failed login recorded with IP | Positive |
| AUDIT-016 | Cross-tenant audit isolation | GET | `/api/audit/logs` | Only own tenant's logs | Security |

---

## 24. Email Templates

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| ETPL-001 | List email templates | GET | `/api/organizations/:org_id/email-templates` | 200 OK, template list | Positive |
| ETPL-002 | Get specific template (verification) | GET | `/api/organizations/:org_id/email-templates/verification` | 200 OK, template | Positive |
| ETPL-003 | Get specific template (password_reset) | GET | `/api/organizations/:org_id/email-templates/password_reset` | 200 OK | Positive |
| ETPL-004 | Update email template | PUT | `/api/organizations/:org_id/email-templates/verification` | 200 OK | Positive |
| ETPL-005 | Update template with HTML content | PUT | `/api/organizations/:org_id/email-templates/verification` | 200 OK, HTML saved | Positive |
| ETPL-006 | Delete custom template (revert to default) | DELETE | `/api/organizations/:org_id/email-templates/verification` | 200 OK | Positive |
| ETPL-007 | Preview email template | POST | `/api/organizations/:org_id/email-templates/verification/preview` | 200 OK, rendered HTML | Positive |
| ETPL-008 | Get non-existent template type | GET | `/api/organizations/:org_id/email-templates/fake` | 404 | Negative |
| ETPL-009 | Update template without admin | PUT | `/api/organizations/:org_id/email-templates/verification` | 403 | Security |
| ETPL-010 | Update template with XSS in content | PUT | `/api/organizations/:org_id/email-templates/verification` | Should sanitize or store as-is | Security |
| ETPL-011 | Preview template with variable substitution | POST | `/api/organizations/:org_id/email-templates/verification/preview` | Variables replaced correctly | Positive |

---

## 25. Branding & Security Settings

### 25.1 Branding

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| BRAND-001 | Get branding settings | GET | `/api/organizations/:org_id/branding` | 200 OK, current branding | Positive |
| BRAND-002 | Update branding (logo, colors, CSS) | PUT | `/api/organizations/:org_id/branding` | 200 OK | Positive |
| BRAND-003 | Update branding without admin | PUT | `/api/organizations/:org_id/branding` | 403 Forbidden | Security |
| BRAND-004 | Update branding with XSS in custom CSS | PUT | `/api/organizations/:org_id/branding` | Sanitized or stored safely | Security |
| BRAND-005 | Update branding for wrong org | PUT | `/api/organizations/:wrong/branding` | 403 | Security |

### 25.2 Security Settings

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| SEC-001 | Get security settings | GET | `/api/organizations/:org_id/security` | 200 OK, settings object | Positive |
| SEC-002 | Update MFA requirement | PUT | `/api/organizations/:org_id/security` | 200 OK, mfa_required updated | Positive |
| SEC-003 | Update password minimum length | PUT | `/api/organizations/:org_id/security` | 200 OK | Positive |
| SEC-004 | Update max login attempts | PUT | `/api/organizations/:org_id/security` | 200 OK | Positive |
| SEC-005 | Update lockout duration | PUT | `/api/organizations/:org_id/security` | 200 OK | Positive |
| SEC-006 | Update security without admin | PUT | `/api/organizations/:org_id/security` | 403 Forbidden | Security |
| SEC-007 | Set password min length to 0 | PUT | `/api/organizations/:org_id/security` | 400 Bad Request | Negative |
| SEC-008 | Set max login attempts to negative | PUT | `/api/organizations/:org_id/security` | 400 Bad Request | Negative |
| SEC-009 | Verify MFA enforcement after enabling | POST | `/api/auth/login` | Login requires MFA | Positive |
| SEC-010 | Update security for wrong org | PUT | `/api/organizations/:wrong/security` | 403 | Security |

---

## 26. Rate Limiting & Token Customization

### 26.1 Rate Limits

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| RATE-001 | Get rate limit config | GET | `/api/tenants/:tid/rate-limits` | 200 OK, current limits | Positive |
| RATE-002 | Update rate limit | PUT | `/api/tenants/:tid/rate-limits` | 200 OK | Positive |
| RATE-003 | Update rate limit without admin | PUT | `/api/tenants/:tid/rate-limits` | 403 Forbidden | Security |
| RATE-004 | Set rate limit to 0 | PUT | `/api/tenants/:tid/rate-limits` | 400 or 200 (disables) | Boundary |
| RATE-005 | Set rate limit to negative | PUT | `/api/tenants/:tid/rate-limits` | 400 Bad Request | Negative |

### 26.2 Token Claims Customization

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| TCLAIM-001 | Get token claims config | GET | `/api/tenants/:tid/applications/:app_id/token-claims` | 200 OK | Positive |
| TCLAIM-002 | Update token claims | PUT | `/api/tenants/:tid/applications/:app_id/token-claims` | 200 OK | Positive |
| TCLAIM-003 | Verify custom claims in issued token | POST | `/oauth/token` | Decode JWT, custom claims present | Positive |
| TCLAIM-004 | Update claims without admin | PUT | `/api/tenants/:tid/applications/:app_id/token-claims` | 403 | Security |

---

## 27. Tenant Registry (Platform Admin)

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| TREG-001 | List all tenants (admin) | GET | `/api/admin/tenants` | 200 OK, all tenants | Positive |
| TREG-002 | Create tenant via admin API | POST | `/api/admin/tenants` | 201 Created | Positive |
| TREG-003 | Create tenant with dedicated database | POST | `/api/admin/tenants` | 201, isolation_mode=dedicated | Positive |
| TREG-004 | Create tenant with invalid slug | POST | `/api/admin/tenants` | 400 Bad Request | Negative |
| TREG-005 | Get tenant details | GET | `/api/admin/tenants/:tid` | 200 OK, full details | Positive |
| TREG-006 | Get router stats | GET | `/api/admin/tenants/stats` | 200 OK, connection stats | Positive |
| TREG-007 | Configure dedicated database | POST | `/api/admin/tenants/:tid/database` | 200 OK | Positive |
| TREG-008 | Configure DB with invalid connection string | POST | `/api/admin/tenants/:tid/database` | 400 | Negative |
| TREG-009 | Test tenant DB connection | POST | `/api/admin/tenants/:tid/test-connection` | 200 OK, connection test result | Positive |
| TREG-010 | Activate tenant | POST | `/api/admin/tenants/:tid/activate` | 200 OK, status=active | Positive |
| TREG-011 | Suspend tenant | POST | `/api/admin/tenants/:tid/suspend` | 200 OK, status=suspended | Positive |
| TREG-012 | Verify suspended tenant cannot login | POST | `/api/auth/login` | 403 Tenant suspended | Security |
| TREG-013 | Re-activate suspended tenant | POST | `/api/admin/tenants/:tid/activate` | 200 OK | Positive |
| TREG-014 | Registry API without auth | GET | `/api/admin/tenants` | 401 Unauthorized | Negative |
| TREG-015 | Registry API with non-admin user | GET | `/api/admin/tenants` | 403 Forbidden | Security |
| TREG-016 | Create tenant with duplicate slug | POST | `/api/admin/tenants` | 409 Conflict | Negative |

---

## 28. Forward Auth & Proxy Integration

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| PROXY-001 | Access protected route through proxy (authenticated) | GET | `http://localhost:4000/dashboard` | 200 OK, X-CoreAuth-* headers injected | Positive |
| PROXY-002 | Access protected route without session | GET | `http://localhost:4000/dashboard` | 302 Redirect to login | Positive |
| PROXY-003 | Verify X-CoreAuth-User-Id header injected | GET | Proxy protected route | Header present with correct UUID | Positive |
| PROXY-004 | Verify X-CoreAuth-User-Email header injected | GET | Proxy protected route | Header present | Positive |
| PROXY-005 | Verify X-CoreAuth-Tenant-Id header injected | GET | Proxy protected route | Header present | Positive |
| PROXY-006 | Verify X-CoreAuth-Token header injected | GET | Proxy protected route | Bearer token present | Positive |
| PROXY-007 | Verify X-CoreAuth-Role header injected | GET | Proxy protected route | Role present (admin/member/viewer) | Positive |
| PROXY-008 | Access route with auth mode "none" | GET | Proxy public route | No redirect, no headers | Positive |
| PROXY-009 | Access route with auth mode "optional" | GET | Proxy optional route | No redirect, headers if available | Positive |
| PROXY-010 | Proxy session expires | GET | Proxy protected route | 302 Redirect to re-auth | Negative |
| PROXY-011 | Proxy cannot spoof X-CoreAuth-* headers | GET | Send fake headers | Proxy overwrites with real values | Security |
| PROXY-012 | CORS headers properly set | OPTIONS | Proxy route | Correct CORS headers | Positive |
| PROXY-013 | Proxy handles backend downtime gracefully | GET | Proxy route (backend down) | 502/503, not crash | Negative |

---

## 29. Sample App (CoreRun) - Workspaces

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRWS-001 | View dashboard (authenticated via proxy) | GET | `/dashboard` | 200 OK, workspace list | Positive |
| CRWS-002 | Create workspace | POST | `/workspaces` | 302 Redirect, workspace created | Positive |
| CRWS-003 | Create workspace sets FGA admin tuple | POST | `/workspaces` | Creator is admin via FGA | Positive |
| CRWS-004 | View workspace detail (admin) | GET | `/workspaces/:id` | 200 OK, full details | Positive |
| CRWS-005 | View workspace detail (viewer) | GET | `/workspaces/:id` | 200 OK, limited actions | Positive |
| CRWS-006 | View workspace detail (no permission) | GET | `/workspaces/:id` | 403 Forbidden | Security |
| CRWS-007 | Share workspace with user (admin role) | POST | `/workspaces/:id/share` | 200 OK, FGA tuple created | Positive |
| CRWS-008 | Share workspace as viewer (no permission) | POST | `/workspaces/:id/share` | 403 Forbidden | Security |
| CRWS-009 | Delete workspace (admin) | POST | `/workspaces/:id/delete` | 200 OK, cascade delete | Positive |
| CRWS-010 | Delete workspace (viewer) | POST | `/workspaces/:id/delete` | 403 Forbidden | Security |
| CRWS-011 | Create workspace with empty name | POST | `/workspaces` | 400 / validation error | Negative |
| CRWS-012 | List workspaces shows only FGA-permitted | GET | `/workspaces` | Only accessible workspaces shown | Security |

---

## 30. Sample App (CoreRun) - Resources & FGA

### 30.1 Resource CRUD

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRRES-001 | Create compute_instance | POST | `/workspaces/:wid/resources/compute_instance` | 201, FGA owner tuple | Positive |
| CRRES-002 | Create compute_function | POST | `/workspaces/:wid/resources/compute_function` | 201 | Positive |
| CRRES-003 | Create storage_bucket | POST | `/workspaces/:wid/resources/storage_bucket` | 201 | Positive |
| CRRES-004 | Create storage_volume | POST | `/workspaces/:wid/resources/storage_volume` | 201 | Positive |
| CRRES-005 | Create storage_database | POST | `/workspaces/:wid/resources/storage_database` | 201 | Positive |
| CRRES-006 | Create network_vpc | POST | `/workspaces/:wid/resources/network_vpc` | 201 | Positive |
| CRRES-007 | Create network_subnet | POST | `/workspaces/:wid/resources/network_subnet` | 201 | Positive |
| CRRES-008 | Create network_firewall | POST | `/workspaces/:wid/resources/network_firewall` | 201 | Positive |
| CRRES-009 | Create network_lb | POST | `/workspaces/:wid/resources/network_lb` | 201 | Positive |
| CRRES-010 | Create resource with invalid type | POST | `/workspaces/:wid/resources/invalid_type` | 400/404 | Negative |
| CRRES-011 | Create resource in non-existent workspace | POST | `/workspaces/:fake/resources/compute_instance` | 404 | Negative |
| CRRES-012 | Create resource without workspace permission | POST | `/workspaces/:wid/resources/compute_instance` | 403 | Security |
| CRRES-013 | View resource detail (owner) | GET | `/resources/compute_instance/:id` | 200 OK, full details | Positive |
| CRRES-014 | View resource detail (viewer via workspace) | GET | `/resources/compute_instance/:id` | 200 OK, limited actions | Positive |
| CRRES-015 | View resource detail (no permission) | GET | `/resources/compute_instance/:id` | 403 Forbidden | Security |
| CRRES-016 | Delete resource (owner) | POST | `/resources/compute_instance/:id/delete` | 200 OK | Positive |
| CRRES-017 | Delete resource (viewer) | POST | `/resources/compute_instance/:id/delete` | 403 Forbidden | Security |

### 30.2 Resource Actions

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRACT-001 | Start compute_instance (operator role) | POST | `/resources/compute_instance/:id/action` | 200 OK, status=running | Positive |
| CRACT-002 | Stop compute_instance (operator role) | POST | `/resources/compute_instance/:id/action` | 200 OK, status=stopped | Positive |
| CRACT-003 | Restart compute_instance (operator role) | POST | `/resources/compute_instance/:id/action` | 200 OK | Positive |
| CRACT-004 | Deploy compute_function (deployer role) | POST | `/resources/compute_function/:id/action` | 200 OK | Positive |
| CRACT-005 | Invoke compute_function (invoker role) | POST | `/resources/compute_function/:id/action` | 200 OK | Positive |
| CRACT-006 | Attach storage_volume (attacher role) | POST | `/resources/storage_volume/:id/action` | 200 OK | Positive |
| CRACT-007 | Start storage_database (admin role) | POST | `/resources/storage_database/:id/action` | 200 OK | Positive |
| CRACT-008 | Perform action without required role | POST | `/resources/compute_instance/:id/action` | 403 Forbidden | Security |
| CRACT-009 | Perform action as viewer (insufficient) | POST | `/resources/compute_instance/:id/action` | 403 Forbidden | Security |

### 30.3 FGA Permission Inheritance

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRFGA-001 | Workspace admin can view all resources | GET | `/resources/:type/:id` | 200 OK (via workspace admin→owner) | Positive |
| CRFGA-002 | Workspace member can view resources | GET | `/resources/:type/:id` | 200 OK (member→viewer) | Positive |
| CRFGA-003 | Workspace viewer can view resources | GET | `/resources/:type/:id` | 200 OK | Positive |
| CRFGA-004 | Workspace viewer cannot perform actions | POST | `/resources/:type/:id/action` | 403 | Security |
| CRFGA-005 | Resource owner has all permissions | POST | `/resources/:type/:id/action` | 200 OK (any action) | Positive |
| CRFGA-006 | After removing workspace access, resources inaccessible | DELETE tuple + GET | Resource endpoint | 403 | Security |
| CRFGA-007 | Share resource with specific user (non-workspace member) | POST | `/resources/:type/:id/share` | 200, user gains access | Positive |
| CRFGA-008 | Group-based workspace access | POST | FGA tuple (group→workspace) | Group members access workspace | Positive |

### 30.4 Resource Sharing

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRSHARE-001 | Share resource with user (owner) | POST | `/resources/:type/:id/share` | 200 OK, FGA tuple | Positive |
| CRSHARE-002 | Share resource (not owner) | POST | `/resources/:type/:id/share` | 403 Forbidden | Security |
| CRSHARE-003 | Share with invalid user email | POST | `/resources/:type/:id/share` | 404 User not found | Negative |
| CRSHARE-004 | Share with all role types for resource | POST | `/resources/:type/:id/share` | Correct role assigned | Positive |
| CRSHARE-005 | Search users for sharing (autocomplete) | GET | `/api/users/search?q=alice` | 200 OK, matching users | Positive |
| CRSHARE-006 | Search users with empty query | GET | `/api/users/search?q=` | 200 OK, empty or all | Boundary |

---

## 31. Sample App (CoreRun) - Admin Features

### 31.1 User Management (via CoreRun)

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRADM-001 | View user management page | GET | `/admin/users` | 200 OK, user list + MFA stats | Positive |
| CRADM-002 | Update user role | POST | `/admin/users/:uid/role` | 200 OK, role updated | Positive |
| CRADM-003 | Send invitation | POST | `/admin/invitations` | 200 OK, email sent | Positive |
| CRADM-004 | Revoke invitation | POST | `/admin/invitations/:id/revoke` | 200 OK | Positive |
| CRADM-005 | Resend invitation | POST | `/admin/invitations/:id/resend` | 200 OK | Positive |

### 31.2 Group Management (via CoreRun)

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRGRP-001 | View groups page | GET | `/admin/groups` | 200 OK, group list | Positive |
| CRGRP-002 | Create group | POST | `/admin/groups` | 302 Redirect, created | Positive |
| CRGRP-003 | View group detail | GET | `/admin/groups/:gid` | 200 OK, members list | Positive |
| CRGRP-004 | Add member to group | POST | `/admin/groups/:gid/members` | 200 OK | Positive |
| CRGRP-005 | Remove member from group | POST | `/admin/groups/:gid/members/:uid/remove` | 200 OK | Positive |
| CRGRP-006 | Delete group | POST | `/admin/groups/:gid/delete` | 200 OK | Positive |

### 31.3 Session Management (via CoreRun)

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRSESS-001 | View active sessions | GET | `/admin/sessions` | 200 OK, session list | Positive |
| CRSESS-002 | Revoke session | POST | `/admin/sessions/:sid/revoke` | 200 OK | Positive |

### 31.4 Settings (via CoreRun)

| ID | Test Case | Method | Endpoint | Expected | Type |
|----|-----------|--------|----------|----------|------|
| CRSET-001 | View settings page | GET | `/admin/settings` | 200 OK, branding + SSO + MFA | Positive |
| CRSET-002 | Update branding (app_name, logo, color) | POST | `/admin/settings/branding` | 200 OK | Positive |
| CRSET-003 | Enable/disable MFA | POST | `/admin/settings/mfa` | 200 OK | Positive |
| CRSET-004 | Create SSO connection (Azure AD) | POST | `/admin/settings/sso` | 200 OK, provider created | Positive |
| CRSET-005 | Create SSO connection (Okta) | POST | `/admin/settings/sso` | 200 OK | Positive |
| CRSET-006 | Create SSO connection (Google Workspace) | POST | `/admin/settings/sso` | 200 OK | Positive |
| CRSET-007 | Create SSO connection (custom OIDC) | POST | `/admin/settings/sso` | 200 OK | Positive |
| CRSET-008 | Toggle SSO provider (enable/disable) | POST | `/admin/settings/sso/:pid/toggle` | 200 OK | Positive |
| CRSET-009 | Delete SSO provider | POST | `/admin/settings/sso/:pid/delete` | 200 OK | Positive |
| CRSET-010 | Create SSO with invalid credentials | POST | `/admin/settings/sso` | Error message displayed | Negative |

---

## Cross-Cutting Concerns

### Security Test Cases

| ID | Test Case | Expected | Type |
|----|-----------|----------|------|
| SEC-XC-001 | All protected endpoints return 401 without auth token | 401 Unauthorized | Security |
| SEC-XC-002 | All tenant-admin endpoints return 403 for non-admin users | 403 Forbidden | Security |
| SEC-XC-003 | JWT tokens cannot be tampered with (signature verification) | 401 on tampered token | Security |
| SEC-XC-004 | Cross-tenant data isolation (tenant A cannot see tenant B data) | Empty results or 403 | Security |
| SEC-XC-005 | SQL injection in all string parameters | No SQL errors exposed | Security |
| SEC-XC-006 | XSS payloads in user-controlled fields | Sanitized or escaped | Security |
| SEC-XC-007 | CORS headers only allow configured origins | No wildcard CORS | Security |
| SEC-XC-008 | Sensitive data (passwords, secrets) never in response bodies | No plaintext secrets | Security |
| SEC-XC-009 | Rate limiting applies to all auth endpoints | 429 after threshold | Security |
| SEC-XC-010 | Account lockout enforced after max failed attempts | 423 Locked | Security |
| SEC-XC-011 | Expired tokens cannot access any endpoint | 401 Unauthorized | Security |
| SEC-XC-012 | HTTPS enforcement (no sensitive data over HTTP) | Redirect or refuse | Security |
| SEC-XC-013 | No user enumeration via login/reset/invite endpoints | Generic error messages | Security |
| SEC-XC-014 | Verify PKCE prevents authorization code interception | Code exchange fails without verifier | Security |
| SEC-XC-015 | Client secret rotation invalidates old secret immediately | 401 with old secret | Security |

### Performance & Boundary Cases

| ID | Test Case | Expected | Type |
|----|-----------|----------|------|
| PERF-001 | Create 100+ users in a tenant | All created successfully | Performance |
| PERF-002 | Create 50+ groups with members | All created | Performance |
| PERF-003 | FGA check with deep relation hierarchy (5+ levels) | Resolves within timeout | Performance |
| PERF-004 | List users with large result set (pagination) | Paginated correctly | Performance |
| PERF-005 | Concurrent login requests (50 simultaneous) | All succeed or rate-limited | Performance |
| PERF-006 | SCIM sync with 500+ users | Completes without timeout | Performance |
| PERF-007 | Audit log query over large dataset | Paginated, fast response | Performance |
| BOUND-001 | Maximum field lengths (email 255, name 255, etc.) | Accepted at boundary | Boundary |
| BOUND-002 | Unicode characters in all text fields | Stored and returned correctly | Boundary |
| BOUND-003 | Empty string vs null in optional fields | Handled consistently | Boundary |
| BOUND-004 | UUID format validation on all ID parameters | 400 for invalid UUIDs | Boundary |
| BOUND-005 | Pagination: page=0, page=-1, count=0 | Handled gracefully | Boundary |

---

## Test Execution Notes

### Prerequisites
1. Docker compose stack running (`docker compose up --build`)
2. Both tenants (corerun, imys) bootstrapped with EntraID connections
3. Admin accounts created for both tenants
4. MailHog running for email verification (port 1025/8025)
5. SMPP gateway available for SMS testing

### Environment Variables Required
```bash
# Corerun EntraID
CORERUN_ENTRA_CLIENT_ID="44c462b5-fc20-4de0-8c23-fcec7516435c"
CORERUN_ENTRA_TENANT_ID="4ef69c55-ad51-453a-8538-449356a6c6c6"
CORERUN_ENTRA_ADMIN_GROUP_ID="f80af0fc-8353-4ed6-b129-61af0d722abc"

# IMYS EntraID
IMYS_ENTRA_CLIENT_ID="b6e6a0ae-c665-470c-808d-161c2fa37323"
IMYS_ENTRA_TENANT_ID="c9af7ec0-d5ea-4b5f-9aa8-f2d09e515f46"
IMYS_ENTRA_ADMIN_GROUP_ID="0d35f722-797c-46ec-8360-8dc018c81e09"
IMYS_ENTRA_USER_GROUP_ID="28b11038-ea9c-452b-a140-51444a6af489"
```

### Test Data
- Tenant A: corerun (with EntraID connection)
- Tenant B: imys (with EntraID connection)
- Admin user per tenant (created via bootstrap)
- Regular user per tenant (registered via API)
- EntraID test users in Azure AD (with appropriate group memberships)

### Total Test Cases: ~450+
- Positive: ~200
- Negative: ~130
- Security: ~80
- Boundary/Edge: ~40
- Performance: ~10
