#!/bin/sh
# CoreRun Sample Bootstrap
# Auto-provisions tenant, admin user, OAuth application,
# and generates proxy.yaml with the actual client credentials.
# Runs as a one-shot init container after backend is healthy.
set -e

SENTINEL="/data/.bootstrapped"
CREDS_FILE="/data/credentials.json"
PROXY_CONFIG="/proxy-config/proxy.yaml"

# ── Proxy config generator ──────────────────────────────────────
write_proxy_config() {
  cat > "$PROXY_CONFIG" << PROXYEOF
server:
  listen: "0.0.0.0:4000"
  upstream: "http://app:3001"

coreauth:
  url: "http://backend:${BACKEND_PORT:-3000}"
  client_id: "${CLIENT_ID}"
  client_secret: "${CLIENT_SECRET}"
  callback_url: "http://localhost:4000/auth/callback"

session:
  secret: "${PROXY_SESSION_SECRET:-change-this-to-a-32-byte-random-hex-string-in-production}"
  cookie_name: "coreauth_session"
  cookie_domain: ""
  max_age_seconds: 86400
  secure: false

fga:
  store_name: "${FGA_STORE_NAME:-corerun-auth}"

routes:
  # CoreAuth OAuth/login endpoints — forward to CoreAuth backend
  - match: { path: "/authorize" }
    target: coreauth
    auth: none
  - match: { path: "/login/**" }
    target: coreauth
    auth: none
  - match: { path: "/login" }
    target: coreauth
    auth: none
  - match: { path: "/signup/**" }
    target: coreauth
    auth: none
  - match: { path: "/signup" }
    target: coreauth
    auth: none
  - match: { path: "/logout" }
    target: coreauth
    auth: none
  - match: { path: "/logged-out" }
    target: coreauth
    auth: none
  - match: { path: "/mfa/**" }
    target: coreauth
    auth: none
  - match: { path: "/consent/**" }
    target: coreauth
    auth: none
  - match: { path: "/consent" }
    target: coreauth
    auth: none
  - match: { path: "/verify-email" }
    target: coreauth
    auth: none
  - match: { path: "/oauth/**" }
    target: coreauth
    auth: none
  - match: { path: "/.well-known/**" }
    target: coreauth
    auth: none

  # Static assets
  - match: { path: "/app.css" }
    auth: none
  - match: { path: "/favicon.ico" }
    auth: none

  # API routes — return 401 not redirect
  - match: { path: "/api/**" }
    auth: required
    on_unauthenticated: status401

  # App pages — require auth
  - match: { path: "/dashboard" }
    auth: required
    on_unauthenticated: redirect_login
  - match: { path: "/workspaces/**" }
    auth: required
    on_unauthenticated: redirect_login
  - match: { path: "/workspaces" }
    auth: required
    on_unauthenticated: redirect_login
  - match: { path: "/resources/**" }
    auth: required
    on_unauthenticated: redirect_login
  - match: { path: "/admin/**" }
    auth: required
    on_unauthenticated: redirect_login

  # Everything else — optional auth
  - match: { path: "/**" }
    auth: optional
PROXYEOF
  echo "[proxy] Config written to ${PROXY_CONFIG}"
}

# ── Idempotency check ──────────────────────────────────────────
if [ -f "$SENTINEL" ]; then
  echo "[bootstrap] Already bootstrapped. Remove volume to re-run."
  # Re-generate proxy config if it's missing (volume might have been cleared)
  if [ ! -f "$PROXY_CONFIG" ] && [ -f "$CREDS_FILE" ]; then
    CLIENT_ID=$(cat "$CREDS_FILE" | sed -n 's/.*"client_id" *: *"\([^"]*\)".*/\1/p')
    CLIENT_SECRET=$(cat "$CREDS_FILE" | sed -n 's/.*"client_secret" *: *"\([^"]*\)".*/\1/p')
    write_proxy_config
  fi
  exit 0
fi

echo "============================================"
echo "  CoreRun Sample Bootstrap"
echo "============================================"
echo ""

API="${BACKEND_URL:-http://backend:3000}"

# ── Step 1: Create tenant + admin user ─────────────────────────
echo "[1/6] Creating tenant '${TENANT_SLUG}'..."

TENANT_RESP=$(curl -s -w "\n%{http_code}" -X POST "${API}/api/tenants" \
  -H "Content-Type: application/json" \
  -d "{
    \"name\": \"${TENANT_NAME}\",
    \"slug\": \"${TENANT_SLUG}\",
    \"admin_email\": \"${ADMIN_EMAIL}\",
    \"admin_password\": \"${ADMIN_PASSWORD}\",
    \"admin_full_name\": \"${ADMIN_FULL_NAME:-Admin User}\"
  }")

TENANT_HTTP=$(echo "$TENANT_RESP" | tail -1)
TENANT_BODY=$(echo "$TENANT_RESP" | sed '$d')

if [ "$TENANT_HTTP" = "200" ] || [ "$TENANT_HTTP" = "201" ]; then
  TENANT_ID=$(echo "$TENANT_BODY" | sed -n 's/.*"tenant_id" *: *"\([^"]*\)".*/\1/p')
  echo "       Tenant created: ${TENANT_ID}"
elif [ "$TENANT_HTTP" = "409" ]; then
  echo "       Tenant '${TENANT_SLUG}' already exists (409). Continuing..."
else
  echo "       ERROR: Failed to create tenant (HTTP ${TENANT_HTTP})"
  echo "       Response: ${TENANT_BODY}"
  exit 1
fi

# ── Step 2: Login as admin to get JWT ──────────────────────────
echo "[2/6] Logging in as admin..."

LOGIN_RESP=$(curl -s -w "\n%{http_code}" -X POST "${API}/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{
    \"tenant_id\": \"${TENANT_SLUG}\",
    \"email\": \"${ADMIN_EMAIL}\",
    \"password\": \"${ADMIN_PASSWORD}\"
  }")

LOGIN_HTTP=$(echo "$LOGIN_RESP" | tail -1)
LOGIN_BODY=$(echo "$LOGIN_RESP" | sed '$d')

if [ "$LOGIN_HTTP" != "200" ]; then
  echo "       ERROR: Login failed (HTTP ${LOGIN_HTTP})"
  echo "       Response: ${LOGIN_BODY}"
  exit 1
fi

ACCESS_TOKEN=$(echo "$LOGIN_BODY" | sed -n 's/.*"access_token" *: *"\([^"]*\)".*/\1/p')

if [ -z "$ACCESS_TOKEN" ]; then
  echo "       ERROR: Could not extract access_token"
  echo "       Response: ${LOGIN_BODY}"
  exit 1
fi

echo "       Authenticated successfully."

# Extract tenant_id from login response if we didn't get it from create (409 case)
if [ -z "$TENANT_ID" ]; then
  TENANT_ID=$(echo "$LOGIN_BODY" | sed -n 's/.*"tenant_id" *: *"\([^"]*\)".*/\1/p')
fi

if [ -z "$TENANT_ID" ]; then
  echo "       WARNING: Could not determine tenant_id."
fi

# ── Step 3: Create OAuth application ──────────────────────────
echo "[3/6] Creating application '${APP_NAME}'..."

CALLBACK_JSON=$(echo "${APP_CALLBACK_URLS}" | sed 's/,/","/g' | sed 's/^/["/' | sed 's/$/"]/')

APP_RESP=$(curl -s -w "\n%{http_code}" -X POST "${API}/api/organizations/${TENANT_ID}/applications" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -d "{
    \"name\": \"${APP_NAME}\",
    \"slug\": \"${APP_SLUG:-corerun}\",
    \"app_type\": \"${APP_TYPE:-webapp}\",
    \"callback_urls\": ${CALLBACK_JSON}
  }")

APP_HTTP=$(echo "$APP_RESP" | tail -1)
APP_BODY=$(echo "$APP_RESP" | sed '$d')

if [ "$APP_HTTP" = "200" ] || [ "$APP_HTTP" = "201" ]; then
  CLIENT_ID=$(echo "$APP_BODY" | sed -n 's/.*"client_id" *: *"\([^"]*\)".*/\1/p')
  CLIENT_SECRET=$(echo "$APP_BODY" | sed -n 's/.*"client_secret_plain" *: *"\([^"]*\)".*/\1/p')
  echo "       Application created."
elif [ "$APP_HTTP" = "409" ]; then
  echo "       Application already exists (409)."
  echo "       ERROR: Cannot retrieve credentials for existing app."
  echo "       Delete volumes and restart: docker compose down -v && docker compose up --build"
  exit 1
else
  echo "       ERROR: Failed to create application (HTTP ${APP_HTTP})"
  echo "       Response: ${APP_BODY}"
  exit 1
fi

if [ -z "$CLIENT_ID" ] || [ -z "$CLIENT_SECRET" ]; then
  echo "       ERROR: Could not extract client credentials from response"
  echo "       Response: ${APP_BODY}"
  exit 1
fi

# ── Step 4: Configure tenant security settings ───────────────
if [ "${REQUIRE_EMAIL_VERIFICATION}" = "true" ] || [ "${REQUIRE_EMAIL_VERIFICATION}" = "1" ]; then
  echo "[4/6] Configuring tenant security: email verification = required..."
  SEC_RESP=$(curl -s -w "\n%{http_code}" -X PUT "${API}/api/organizations/${TENANT_ID}/security" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -d '{"require_email_verification": true}')
  SEC_HTTP=$(echo "$SEC_RESP" | tail -1)
  if [ "$SEC_HTTP" = "200" ]; then
    echo "       Email verification enabled for tenant."
  else
    echo "       WARNING: Could not update security settings (HTTP ${SEC_HTTP})"
  fi
else
  echo "[4/6] Email verification: disabled (default)"
fi

# Set tenant branding (app_name drives {{org_name}} in email templates)
BRAND_RESP=$(curl -s -w "\n%{http_code}" -X PUT \
  "${API}/api/organizations/${TENANT_ID}/branding" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -d "{\"app_name\": \"${TENANT_NAME}\", \"primary_color\": \"#6366f1\"}")
BRAND_HTTP=$(echo "$BRAND_RESP" | tail -1)
if [ "$BRAND_HTTP" = "200" ]; then
  echo "       Branding set: app_name=${TENANT_NAME}, primary_color=#6366f1"
else
  echo "       WARNING: Could not update branding (HTTP ${BRAND_HTTP})"
fi

# ── Step 5: Configure custom email templates ─────────────────
echo "[5/6] Configuring custom email templates for ${TENANT_NAME}..."

# Email verification template — branded card layout
VERIFY_RESP=$(curl -s -w "\n%{http_code}" -X PUT \
  "${API}/api/organizations/${TENANT_ID}/email-templates/email_verification" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -d '{
    "subject": "Verify your email for {{org_name}}",
    "html_body": "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif;margin:0;padding:0;background:#f4f4f5}*{box-sizing:border-box}.wrap{max-width:560px;margin:40px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,.08)}.header{background:{{primary_color}};padding:32px;text-align:center}.header h1{color:#fff;margin:0;font-size:20px;font-weight:600}.body{padding:32px}.body h2{margin:0 0 8px;font-size:22px;color:#18181b}.body p{color:#52525b;line-height:1.7;margin:8px 0}.btn{display:inline-block;padding:12px 32px;background:{{primary_color}};color:#fff !important;text-decoration:none;border-radius:8px;font-weight:600;margin:24px 0}.link{word-break:break-all;color:#71717a;font-size:13px}.footer{padding:24px 32px;background:#fafafa;text-align:center;font-size:12px;color:#a1a1aa}</style></head><body><div class=\"wrap\"><div class=\"header\"><h1>{{org_name}}</h1></div><div class=\"body\"><h2>Verify your email</h2><p>Hi {{user_name}},</p><p>Thanks for signing up! Please verify your email address to get started.</p><p style=\"text-align:center\"><a href=\"{{verification_link}}\" class=\"btn\">Verify Email Address</a></p><p>Or copy this link into your browser:</p><p class=\"link\">{{verification_link}}</p><p style=\"color:#a1a1aa;font-size:13px\">This link expires at {{expires_at}}.</p></div><div class=\"footer\">{{org_name}} &mdash; Powered by CoreAuth</div></div></body></html>",
    "text_body": "Hi {{user_name}},\n\nThanks for signing up for {{org_name}}!\n\nPlease verify your email address by visiting:\n{{verification_link}}\n\nThis link expires at {{expires_at}}.\n\n-- The {{org_name}} Team"
  }')
VERIFY_HTTP=$(echo "$VERIFY_RESP" | tail -1)
if [ "$VERIFY_HTTP" = "200" ] || [ "$VERIFY_HTTP" = "201" ]; then
  echo "       email_verification template set."
else
  echo "       WARNING: email_verification template failed (HTTP ${VERIFY_HTTP})"
fi

# Password reset template — branded card with security notice
RESET_RESP=$(curl -s -w "\n%{http_code}" -X PUT \
  "${API}/api/organizations/${TENANT_ID}/email-templates/password_reset" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -d '{
    "subject": "Reset your {{org_name}} password",
    "html_body": "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif;margin:0;padding:0;background:#f4f4f5}*{box-sizing:border-box}.wrap{max-width:560px;margin:40px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,.08)}.header{background:{{primary_color}};padding:32px;text-align:center}.header h1{color:#fff;margin:0;font-size:20px;font-weight:600}.body{padding:32px}.body h2{margin:0 0 8px;font-size:22px;color:#18181b}.body p{color:#52525b;line-height:1.7;margin:8px 0}.btn{display:inline-block;padding:12px 32px;background:{{primary_color}};color:#fff !important;text-decoration:none;border-radius:8px;font-weight:600;margin:24px 0}.notice{background:#fef3c7;border-left:4px solid #f59e0b;padding:12px 16px;border-radius:0 8px 8px 0;margin:16px 0;font-size:13px;color:#92400e}.link{word-break:break-all;color:#71717a;font-size:13px}.footer{padding:24px 32px;background:#fafafa;text-align:center;font-size:12px;color:#a1a1aa}</style></head><body><div class=\"wrap\"><div class=\"header\"><h1>{{org_name}}</h1></div><div class=\"body\"><h2>Reset your password</h2><p>Hi {{user_name}},</p><p>We received a request to reset your password. Click the button below to choose a new one.</p><p style=\"text-align:center\"><a href=\"{{reset_link}}\" class=\"btn\">Reset Password</a></p><p>Or copy this link into your browser:</p><p class=\"link\">{{reset_link}}</p><div class=\"notice\">This request came from IP: {{ip_address}}. If you did not request this, you can safely ignore this email.</div><p style=\"color:#a1a1aa;font-size:13px\">This link expires at {{expires_at}}.</p></div><div class=\"footer\">{{org_name}} &mdash; Powered by CoreAuth</div></div></body></html>",
    "text_body": "Hi {{user_name}},\n\nWe received a request to reset your {{org_name}} password.\n\nReset your password here:\n{{reset_link}}\n\nThis request came from IP: {{ip_address}}\nThis link expires at {{expires_at}}.\n\nIf you did not request this, you can safely ignore this email.\n\n-- The {{org_name}} Team"
  }')
RESET_HTTP=$(echo "$RESET_RESP" | tail -1)
if [ "$RESET_HTTP" = "200" ] || [ "$RESET_HTTP" = "201" ]; then
  echo "       password_reset template set."
else
  echo "       WARNING: password_reset template failed (HTTP ${RESET_HTTP})"
fi

# User invitation template — branded card with info box
INVITE_RESP=$(curl -s -w "\n%{http_code}" -X PUT \
  "${API}/api/organizations/${TENANT_ID}/email-templates/user_invitation" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${ACCESS_TOKEN}" \
  -d '{
    "subject": "{{invited_by_name}} invited you to {{tenant_name}}",
    "html_body": "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{font-family:-apple-system,BlinkMacSystemFont,Segoe UI,Roboto,sans-serif;margin:0;padding:0;background:#f4f4f5}*{box-sizing:border-box}.wrap{max-width:560px;margin:40px auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 1px 3px rgba(0,0,0,.08)}.header{background:{{primary_color}};padding:32px;text-align:center}.header h1{color:#fff;margin:0;font-size:20px;font-weight:600}.body{padding:32px}.body h2{margin:0 0 8px;font-size:22px;color:#18181b}.body p{color:#52525b;line-height:1.7;margin:8px 0}.btn{display:inline-block;padding:12px 32px;background:{{primary_color}};color:#fff !important;text-decoration:none;border-radius:8px;font-weight:600;margin:24px 0}.info{background:#eff6ff;border-left:4px solid {{primary_color}};padding:12px 16px;border-radius:0 8px 8px 0;margin:16px 0;font-size:14px;color:#1e40af}.link{word-break:break-all;color:#71717a;font-size:13px}.footer{padding:24px 32px;background:#fafafa;text-align:center;font-size:12px;color:#a1a1aa}</style></head><body><div class=\"wrap\"><div class=\"header\"><h1>{{org_name}}</h1></div><div class=\"body\"><h2>You are invited!</h2><p><strong>{{invited_by_name}}</strong> has invited you to join <strong>{{tenant_name}}</strong> as a <strong>{{role_name}}</strong>.</p><p style=\"text-align:center\"><a href=\"{{invitation_link}}\" class=\"btn\">Accept Invitation</a></p><p>Or copy this link into your browser:</p><p class=\"link\">{{invitation_link}}</p><div class=\"info\">This invitation expires at {{expires_at}}.</div></div><div class=\"footer\">{{org_name}} &mdash; Powered by CoreAuth</div></div></body></html>",
    "text_body": "You have been invited!\n\n{{invited_by_name}} has invited you to join {{tenant_name}} as a {{role_name}}.\n\nAccept the invitation here:\n{{invitation_link}}\n\nThis invitation expires at {{expires_at}}.\n\n-- The {{org_name}} Team"
  }')
INVITE_HTTP=$(echo "$INVITE_RESP" | tail -1)
if [ "$INVITE_HTTP" = "200" ] || [ "$INVITE_HTTP" = "201" ]; then
  echo "       user_invitation template set."
else
  echo "       WARNING: user_invitation template failed (HTTP ${INVITE_HTTP})"
fi

echo "       Custom email templates configured."

# ── Step 6: Generate proxy config ─────────────────────────────
echo "[6/6] Generating proxy configuration..."
write_proxy_config

# ── Save credentials ──────────────────────────────────────────
cat > "$CREDS_FILE" << CREDS_EOF
{
  "tenant_id": "${TENANT_ID}",
  "tenant_slug": "${TENANT_SLUG}",
  "admin_email": "${ADMIN_EMAIL}",
  "client_id": "${CLIENT_ID}",
  "client_secret": "${CLIENT_SECRET}",
  "api_url": "${API}"
}
CREDS_EOF

# ── Create sentinel ───────────────────────────────────────────
date -u > "$SENTINEL"

# ── Summary ───────────────────────────────────────────────────
echo ""
echo "============================================"
echo "  CoreRun Bootstrap Complete"
echo "============================================"
echo ""
echo "  Tenant:        ${TENANT_NAME} (${TENANT_SLUG})"
echo "  Tenant ID:     ${TENANT_ID}"
echo "  Admin Email:   ${ADMIN_EMAIL}"
echo "  Admin Password: ${ADMIN_PASSWORD}"
echo ""
echo "  OAuth Application:"
echo "    Client ID:     ${CLIENT_ID}"
echo "    Client Secret: ${CLIENT_SECRET}"
echo ""
echo "  Proxy config:  ${PROXY_CONFIG}"
echo "  Credentials:   ${CREDS_FILE}"
echo ""
echo "  Open http://localhost:4000 to get started!"
echo ""
echo "============================================"
