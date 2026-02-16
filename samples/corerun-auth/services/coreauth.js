const http = require('http');
const https = require('https');
const { URL } = require('url');

const BASE_URL = process.env.COREAUTH_BASE_URL || 'http://localhost:8000';

/**
 * Make an HTTP request to CoreAuth API
 */
function request(method, urlPath, { body, headers = {}, form } = {}) {
  return new Promise((resolve, reject) => {
    const url = new URL(urlPath, BASE_URL);
    const isHttps = url.protocol === 'https:';
    const lib = isHttps ? https : http;

    let bodyStr;
    const reqHeaders = { ...headers };

    if (form) {
      bodyStr = new URLSearchParams(form).toString();
      reqHeaders['Content-Type'] = 'application/x-www-form-urlencoded';
    } else if (body) {
      bodyStr = JSON.stringify(body);
      reqHeaders['Content-Type'] = 'application/json';
    }

    if (bodyStr) {
      reqHeaders['Content-Length'] = Buffer.byteLength(bodyStr);
    }

    const opts = {
      hostname: url.hostname,
      port: url.port,
      path: url.pathname + url.search,
      method,
      headers: reqHeaders,
    };

    const req = lib.request(opts, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        let parsed;
        try { parsed = JSON.parse(data); } catch { parsed = data; }
        if (res.statusCode >= 200 && res.statusCode < 300) {
          resolve(parsed);
        } else {
          const err = new Error(`CoreAuth API ${res.statusCode}: ${typeof parsed === 'string' ? parsed : JSON.stringify(parsed)}`);
          err.status = res.statusCode;
          err.response = parsed;
          reject(err);
        }
      });
    });

    req.on('error', reject);
    if (bodyStr) req.write(bodyStr);
    req.end();
  });
}

// ── User Management ──

async function listUsers(tenantId, accessToken) {
  return request('GET', `/api/tenants/${tenantId}/users`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function updateUserRole(tenantId, userId, role, accessToken) {
  return request('PUT', `/api/tenants/${tenantId}/users/${userId}/role`, {
    body: { role },
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

// ── Security & Branding ──

async function getSecuritySettings(orgId, accessToken) {
  return request('GET', `/api/organizations/${orgId}/security`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function updateSecuritySettings(orgId, settings, accessToken) {
  return request('PUT', `/api/organizations/${orgId}/security`, {
    body: settings,
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function getBranding(orgId, accessToken) {
  return request('GET', `/api/organizations/${orgId}/branding`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function updateBranding(orgId, data, accessToken) {
  return request('PUT', `/api/organizations/${orgId}/branding`, {
    body: data,
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

// ── OIDC Providers ──

async function listOidcProviders(tenantId, accessToken) {
  return request('GET', `/api/oidc/providers?tenant_id=${tenantId}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function createOidcProvider(data, accessToken) {
  return request('POST', '/api/oidc/providers', {
    body: data,
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function deleteOidcProvider(providerId, accessToken) {
  return request('DELETE', `/api/oidc/providers/${providerId}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function toggleOidcProvider(providerId, isEnabled, accessToken) {
  return request('PATCH', `/api/oidc/providers/${providerId}`, {
    body: { is_enabled: isEnabled },
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function getOidcTemplates() {
  return request('GET', '/api/oidc/templates');
}

async function getMfaStatus(tenantId, accessToken) {
  return request('GET', `/api/tenants/${tenantId}/mfa-status`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

// ── Invitations ──

async function listInvitations(tenantId, accessToken) {
  return request('GET', `/api/tenants/${tenantId}/invitations`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function createInvitation(tenantId, data, accessToken) {
  return request('POST', `/api/tenants/${tenantId}/invitations`, {
    body: data,
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function revokeInvitation(tenantId, invitationId, accessToken) {
  return request('DELETE', `/api/tenants/${tenantId}/invitations/${invitationId}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function resendInvitation(tenantId, invitationId, accessToken) {
  return request('POST', `/api/tenants/${tenantId}/invitations/${invitationId}/resend`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

// ── Groups ──

async function listGroups(tenantId, accessToken) {
  return request('GET', `/api/tenants/${tenantId}/groups`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function createGroup(tenantId, data, accessToken) {
  return request('POST', `/api/tenants/${tenantId}/groups`, {
    body: data,
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function getGroup(tenantId, groupId, accessToken) {
  return request('GET', `/api/tenants/${tenantId}/groups/${groupId}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function deleteGroup(tenantId, groupId, accessToken) {
  return request('DELETE', `/api/tenants/${tenantId}/groups/${groupId}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function listGroupMembers(tenantId, groupId, accessToken) {
  return request('GET', `/api/tenants/${tenantId}/groups/${groupId}/members`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function addGroupMember(tenantId, groupId, data, accessToken) {
  return request('POST', `/api/tenants/${tenantId}/groups/${groupId}/members`, {
    body: data,
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function removeGroupMember(tenantId, groupId, userId, accessToken) {
  return request('DELETE', `/api/tenants/${tenantId}/groups/${groupId}/members/${userId}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

// ── Sessions ──

async function listSessions(userId, accessToken) {
  return request('GET', `/api/sessions?user_id=${userId}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

async function revokeSession(sessionId, accessToken) {
  return request('DELETE', `/api/sessions/${sessionId}`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

module.exports = {
  request,
  BASE_URL,
  listUsers,
  updateUserRole,
  getSecuritySettings,
  updateSecuritySettings,
  getBranding,
  updateBranding,
  listOidcProviders,
  createOidcProvider,
  deleteOidcProvider,
  toggleOidcProvider,
  getOidcTemplates,
  getMfaStatus,
  listInvitations,
  createInvitation,
  revokeInvitation,
  resendInvitation,
  listGroups,
  createGroup,
  getGroup,
  deleteGroup,
  listGroupMembers,
  addGroupMember,
  removeGroupMember,
  listSessions,
  revokeSession,
};
