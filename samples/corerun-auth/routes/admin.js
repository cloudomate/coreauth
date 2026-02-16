const express = require('express');
const { requireAuth } = require('../middleware/auth');
const coreauth = require('../services/coreauth');

const router = express.Router();

// All admin routes require auth
router.use(requireAuth);

// ── Users ──

router.get('/users', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    if (!tenantId) {
      return res.render('error', { title: 'Error', message: 'No tenant ID found in your session' });
    }

    let users = [];
    try {
      users = await coreauth.listUsers(tenantId, req.accessToken);
      if (!Array.isArray(users)) users = users.users || [];
      // Flatten metadata.full_name into top-level full_name for templates
      users = users.map(u => ({
        ...u,
        full_name: u.full_name || (u.metadata && u.metadata.full_name) || null,
      }));
    } catch (err) {
      console.error('[admin] List users error:', err.message);
    }

    let mfaStatus = null;
    try {
      mfaStatus = await coreauth.getMfaStatus(tenantId, req.accessToken);
    } catch (err) {
      console.error('[admin] MFA status error:', err.message);
    }

    let invitations = [];
    try {
      invitations = await coreauth.listInvitations(tenantId, req.accessToken);
      if (!Array.isArray(invitations)) invitations = [];
    } catch (err) {
      console.error('[admin] List invitations error:', err.message);
    }

    res.render('users', {
      title: 'User Management',
      users,
      mfaStatus,
      invitations,
      tenantId,
    });
  } catch (err) {
    console.error('[admin] Users error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load users' });
  }
});

// Update user role
router.post('/users/:userId/role', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    const { role } = req.body;
    await coreauth.updateUserRole(tenantId, req.params.userId, role, req.accessToken);
    req.session.flash = { type: 'success', message: `User role updated to ${role}` };
  } catch (err) {
    console.error('[admin] Update role error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to update role: ' + err.message };
  }
  res.redirect('/admin/users');
});

// ── Invitations ──

router.post('/invitations', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    const { email, role_id, expires_in_days } = req.body;
    await coreauth.createInvitation(tenantId, {
      email,
      role_id: role_id || undefined,
      expires_in_days: expires_in_days ? parseInt(expires_in_days) : 7,
    }, req.accessToken);
    req.session.flash = { type: 'success', message: `Invitation sent to ${email}` };
  } catch (err) {
    console.error('[admin] Create invitation error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to send invitation: ' + err.message };
  }
  res.redirect('/admin/users');
});

router.post('/invitations/:invitationId/revoke', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    await coreauth.revokeInvitation(tenantId, req.params.invitationId, req.accessToken);
    req.session.flash = { type: 'success', message: 'Invitation revoked' };
  } catch (err) {
    console.error('[admin] Revoke invitation error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to revoke invitation: ' + err.message };
  }
  res.redirect('/admin/users');
});

router.post('/invitations/:invitationId/resend', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    await coreauth.resendInvitation(tenantId, req.params.invitationId, req.accessToken);
    req.session.flash = { type: 'success', message: 'Invitation resent' };
  } catch (err) {
    console.error('[admin] Resend invitation error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to resend invitation: ' + err.message };
  }
  res.redirect('/admin/users');
});

// ── Groups ──

router.get('/groups', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    if (!tenantId) {
      return res.render('error', { title: 'Error', message: 'No tenant ID found in your session' });
    }

    let groupsResult = { groups: [], total: 0 };
    try {
      groupsResult = await coreauth.listGroups(tenantId, req.accessToken);
      if (Array.isArray(groupsResult)) {
        groupsResult = { groups: groupsResult.map(g => ({ group: g, member_count: 0 })), total: groupsResult.length };
      }
    } catch (err) {
      console.error('[admin] List groups error:', err.message);
    }

    res.render('groups', {
      title: 'Groups',
      groups: groupsResult.groups || [],
      total: groupsResult.total || 0,
      tenantId,
    });
  } catch (err) {
    console.error('[admin] Groups error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load groups' });
  }
});

router.post('/groups', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    const { name, slug, description } = req.body;
    await coreauth.createGroup(tenantId, {
      name,
      slug: slug || name.toLowerCase().replace(/[^a-z0-9]+/g, '-'),
      description: description || undefined,
    }, req.accessToken);
    req.session.flash = { type: 'success', message: `Group "${name}" created` };
  } catch (err) {
    console.error('[admin] Create group error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to create group: ' + err.message };
  }
  res.redirect('/admin/groups');
});

router.get('/groups/:groupId', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    let groupResult = {};
    try {
      groupResult = await coreauth.getGroup(tenantId, req.params.groupId, req.accessToken);
    } catch (err) {
      return res.render('error', { title: 'Not Found', message: 'Group not found' });
    }

    let members = [];
    try {
      members = await coreauth.listGroupMembers(tenantId, req.params.groupId, req.accessToken);
      if (!Array.isArray(members)) members = [];
      members = members.map(m => ({
        ...m,
        full_name: m.full_name || (m.metadata && m.metadata.full_name) || null,
      }));
    } catch (err) {
      console.error('[admin] List group members error:', err.message);
    }

    let users = [];
    try {
      users = await coreauth.listUsers(tenantId, req.accessToken);
      if (!Array.isArray(users)) users = users.users || [];
      users = users.map(u => ({
        ...u,
        full_name: u.full_name || (u.metadata && u.metadata.full_name) || null,
      }));
    } catch (err) {
      console.error('[admin] List users for group error:', err.message);
    }

    const group = groupResult.group || groupResult;

    res.render('group-detail', {
      title: group.name || 'Group',
      group,
      members,
      users,
      memberCount: groupResult.member_count || members.length,
      tenantId,
    });
  } catch (err) {
    console.error('[admin] Group detail error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load group' });
  }
});

router.post('/groups/:groupId/members', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    const { user_id, role } = req.body;
    await coreauth.addGroupMember(tenantId, req.params.groupId, {
      user_id,
      role: role || 'member',
    }, req.accessToken);
    req.session.flash = { type: 'success', message: 'Member added to group' };
  } catch (err) {
    console.error('[admin] Add group member error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to add member: ' + err.message };
  }
  res.redirect(`/admin/groups/${req.params.groupId}`);
});

router.post('/groups/:groupId/members/:userId/remove', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    await coreauth.removeGroupMember(tenantId, req.params.groupId, req.params.userId, req.accessToken);
    req.session.flash = { type: 'success', message: 'Member removed from group' };
  } catch (err) {
    console.error('[admin] Remove group member error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to remove member: ' + err.message };
  }
  res.redirect(`/admin/groups/${req.params.groupId}`);
});

router.post('/groups/:groupId/delete', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    await coreauth.deleteGroup(tenantId, req.params.groupId, req.accessToken);
    req.session.flash = { type: 'success', message: 'Group deleted' };
  } catch (err) {
    console.error('[admin] Delete group error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to delete group: ' + err.message };
  }
  res.redirect('/admin/groups');
});

// ── Sessions ──

router.get('/sessions', async (req, res) => {
  try {
    let sessions = [];
    try {
      sessions = await coreauth.listSessions(req.user.sub, req.accessToken);
      if (!Array.isArray(sessions)) sessions = [];
    } catch (err) {
      console.error('[admin] List sessions error:', err.message);
    }

    res.render('sessions', {
      title: 'Active Sessions',
      sessions,
    });
  } catch (err) {
    console.error('[admin] Sessions error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load sessions' });
  }
});

router.post('/sessions/:sessionId/revoke', async (req, res) => {
  try {
    await coreauth.revokeSession(req.params.sessionId, req.accessToken);
    req.session.flash = { type: 'success', message: 'Session revoked' };
  } catch (err) {
    console.error('[admin] Revoke session error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to revoke session: ' + err.message };
  }
  res.redirect('/admin/sessions');
});

// ── Settings ──

router.get('/settings', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    if (!tenantId) {
      return res.render('error', { title: 'Error', message: 'No tenant ID found in your session' });
    }

    let security = {};
    try {
      const secResult = await coreauth.getSecuritySettings(tenantId, req.accessToken);
      security = secResult.security || secResult;
    } catch (err) {
      console.error('[admin] Security settings error:', err.message);
    }

    let providers = [];
    try {
      const provResult = await coreauth.listOidcProviders(tenantId, req.accessToken);
      providers = Array.isArray(provResult) ? provResult : [];
    } catch (err) {
      console.error('[admin] OIDC providers error:', err.message);
    }

    let templates = [];
    try {
      templates = await coreauth.getOidcTemplates();
      if (!Array.isArray(templates)) templates = [];
    } catch (err) {
      console.error('[admin] OIDC templates error:', err.message);
    }

    let branding = {};
    try {
      branding = await coreauth.getBranding(tenantId, req.accessToken);
    } catch (err) {
      console.error('[admin] Branding settings error:', err.message);
    }

    const callbackUrl = `${process.env.COREAUTH_BASE_URL || 'http://localhost:8000'}/api/oidc/callback`;

    res.render('settings', {
      title: 'Settings',
      security,
      providers,
      templates,
      branding,
      tenantId,
      callbackUrl,
    });
  } catch (err) {
    console.error('[admin] Settings error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load settings' });
  }
});

router.post('/settings/branding', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    const { app_name, logo_url, primary_color } = req.body;

    await coreauth.updateBranding(tenantId, {
      app_name: app_name || null,
      logo_url: logo_url || null,
      primary_color: primary_color || null,
    }, req.accessToken);

    req.session.flash = { type: 'success', message: 'Branding updated!' };
    res.redirect('/admin/settings');
  } catch (err) {
    console.error('[admin] Branding update error:', err);
    req.session.flash = { type: 'error', message: 'Failed to update branding: ' + err.message };
    res.redirect('/admin/settings');
  }
});

router.post('/settings/mfa', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    const { mfa_required, mfa_methods } = req.body;

    await coreauth.updateSecuritySettings(tenantId, {
      mfa_required: mfa_required === 'on',
      mfa_methods: mfa_methods ? (Array.isArray(mfa_methods) ? mfa_methods : [mfa_methods]) : ['totp'],
    }, req.accessToken);

    req.session.flash = { type: 'success', message: 'MFA settings updated!' };
    res.redirect('/admin/settings');
  } catch (err) {
    console.error('[admin] MFA update error:', err);
    req.session.flash = { type: 'error', message: 'Failed to update MFA settings: ' + err.message };
    res.redirect('/admin/settings');
  }
});

router.post('/settings/sso', async (req, res) => {
  try {
    const tenantId = req.user.tenant_id;
    const { name, provider_type, client_id, client_secret, domain, azure_tenant_id, admin_group_id } = req.body;

    // Build provider config from template + user input
    let templates = [];
    try { templates = await coreauth.getOidcTemplates(); } catch {}
    const template = templates.find(t => t.provider_type === provider_type);

    let issuer = '', authEndpoint = '', tokenEndpoint = '', jwksUri = '';

    if (template) {
      // Replace placeholders based on provider type
      const replacePlaceholders = (url) => {
        let result = url;
        if (domain) result = result.replace(/\{domain\}/g, domain);
        if (azure_tenant_id) result = result.replace(/\{tenant_id\}/g, azure_tenant_id);
        return result;
      };
      issuer = replacePlaceholders(template.issuer_pattern);
      authEndpoint = replacePlaceholders(template.authorization_endpoint);
      tokenEndpoint = replacePlaceholders(template.token_endpoint);
      jwksUri = replacePlaceholders(template.jwks_uri);
    } else {
      // Custom provider — use raw fields
      issuer = req.body.issuer || '';
      authEndpoint = req.body.authorization_endpoint || '';
      tokenEndpoint = req.body.token_endpoint || '';
      jwksUri = req.body.jwks_uri || '';
    }

    // Build group_role_mappings if admin group is specified
    const group_role_mappings = admin_group_id ? { [admin_group_id]: 'admin' } : null;

    await coreauth.createOidcProvider({
      tenant_id: tenantId,
      name: name || (template ? template.display_name : 'Custom OIDC'),
      provider_type: provider_type || 'custom',
      client_id,
      client_secret,
      issuer,
      authorization_endpoint: authEndpoint,
      token_endpoint: tokenEndpoint,
      jwks_uri: jwksUri,
      scopes: template ? template.scopes : ['openid', 'profile', 'email'],
      groups_claim: template ? template.groups_claim : null,
      group_role_mappings,
      allowed_group_id: admin_group_id || null,
    }, req.accessToken);

    req.session.flash = { type: 'success', message: `SSO connection "${name || template?.display_name}" added!` };
    res.redirect('/admin/settings');
  } catch (err) {
    console.error('[admin] SSO create error:', err);
    req.session.flash = { type: 'error', message: 'Failed to add SSO connection: ' + err.message };
    res.redirect('/admin/settings');
  }
});

router.post('/settings/sso/:providerId/toggle', async (req, res) => {
  try {
    const isEnabled = req.body.is_enabled === 'true';
    await coreauth.toggleOidcProvider(req.params.providerId, isEnabled, req.accessToken);
    req.session.flash = { type: 'success', message: `Connection ${isEnabled ? 'enabled' : 'disabled'}` };
  } catch (err) {
    console.error('[admin] SSO toggle error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to update connection: ' + err.message };
  }
  res.redirect('/admin/settings');
});

router.post('/settings/sso/:providerId/delete', async (req, res) => {
  try {
    await coreauth.deleteOidcProvider(req.params.providerId, req.accessToken);
    req.session.flash = { type: 'success', message: 'SSO connection removed' };
  } catch (err) {
    console.error('[admin] SSO delete error:', err.message);
    req.session.flash = { type: 'error', message: 'Failed to remove connection: ' + err.message };
  }
  res.redirect('/admin/settings');
});

module.exports = router;
