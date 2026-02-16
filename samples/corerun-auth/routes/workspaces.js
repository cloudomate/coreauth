const express = require('express');
const { v4: uuidv4 } = require('uuid');
const { requireAuth } = require('../middleware/auth');
const { requirePermission } = require('../middleware/fga');
const { workspaces, resources } = require('../db');
const fga = require('../services/fga');

const router = express.Router();
router.use(requireAuth);

const wsOwner = { getOwnerId: (id) => workspaces.getById(id)?.owner_id };

// List workspaces
router.get('/', async (req, res) => {
  try {
    const allWorkspaces = workspaces.listAll();
    const accessible = [];

    for (const ws of allWorkspaces) {
      // Owner always has access
      if (ws.owner_id === req.user.sub) {
        accessible.push({ ...ws, role: 'admin' });
        continue;
      }
      const canView = await fga.checkPermission(req.user.sub, 'workspace', ws.id, 'viewer', req.accessToken);
      if (canView) {
        const isAdmin = await fga.checkPermission(req.user.sub, 'workspace', ws.id, 'admin', req.accessToken);
        const isMember = isAdmin || await fga.checkPermission(req.user.sub, 'workspace', ws.id, 'member', req.accessToken);
        accessible.push({ ...ws, role: isAdmin ? 'admin' : isMember ? 'member' : 'viewer' });
      }
    }

    // Add resource counts
    for (const ws of accessible) {
      const counts = resources.countByWorkspace(ws.id);
      ws.computeCount = (counts.compute_instance || 0) + (counts.compute_function || 0);
      ws.storageCount = (counts.storage_bucket || 0) + (counts.storage_volume || 0) + (counts.storage_database || 0);
      ws.networkCount = (counts.network_vpc || 0) + (counts.network_subnet || 0) + (counts.network_firewall || 0) + (counts.network_lb || 0);
    }

    res.render('workspace-list', { title: 'Workspaces', workspaces: accessible });
  } catch (err) {
    console.error('[workspaces] List error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load workspaces' });
  }
});

// New workspace form
router.get('/new', (req, res) => {
  res.render('workspace-form', { title: 'New Workspace', workspace: null });
});

// Create workspace
router.post('/', async (req, res) => {
  try {
    const { name, description, region } = req.body;
    if (!name || !name.trim()) {
      req.session.flash = { type: 'error', message: 'Workspace name is required' };
      return res.redirect('/workspaces/new');
    }

    const id = uuidv4();
    workspaces.create(id, name.trim(), description?.trim() || null, region || 'us-east-1', req.user.sub, req.user.email);

    // Ensure FGA store is initialized before writing tuples
    if (!fga.getStoreId()) {
      try {
        await fga.initFga(req.accessToken);
      } catch (err) {
        console.error('[fga] Init before tuple write failed:', err.message);
      }
    }

    // Write FGA tuple: user is admin of workspace
    try {
      await fga.addPermission(req.user.sub, 'workspace', id, 'admin', req.accessToken);
    } catch (err) {
      console.error('[fga] Failed to write admin tuple:', err.message);
    }

    req.session.flash = { type: 'success', message: `Workspace "${name}" created!` };
    res.redirect(`/workspaces/${id}`);
  } catch (err) {
    console.error('[workspaces] Create error:', err);
    req.session.flash = { type: 'error', message: 'Failed to create workspace' };
    res.redirect('/workspaces/new');
  }
});

// View workspace detail
router.get('/:id', requirePermission('workspace', 'viewer', 'id', wsOwner), async (req, res) => {
  try {
    const ws = workspaces.getById(req.params.id);
    if (!ws) {
      return res.status(404).render('error', { title: 'Not Found', message: 'Workspace not found' });
    }

    // Get resource counts
    const counts = resources.countByWorkspace(ws.id);

    // Get collaborators from FGA
    const tuples = await fga.listObjectTuples('workspace', ws.id, req.accessToken);
    const collaborators = tuples
      .filter(t => t.subject_type === 'user' || t.subject_type === 'User')
      .map(t => ({ userId: t.subject_id, relation: t.relation }));

    // Check permissions
    const isAdmin = await fga.checkPermission(req.user.sub, 'workspace', ws.id, 'admin', req.accessToken);

    res.render('workspace-detail', {
      title: ws.name,
      workspace: ws,
      counts,
      collaborators,
      isAdmin,
    });
  } catch (err) {
    console.error('[workspaces] View error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load workspace' });
  }
});

// Delete workspace
router.post('/:id/delete', requirePermission('workspace', 'admin', 'id', wsOwner), async (req, res) => {
  try {
    // Delete all FGA tuples for the workspace
    const tuples = await fga.listObjectTuples('workspace', req.params.id, req.accessToken);
    for (const t of tuples) {
      try {
        await fga.deleteTuple({
          subjectType: t.subject_type?.toLowerCase(),
          subjectId: t.subject_id,
          subjectRelation: t.subject_relation,
          relation: t.relation,
          objectType: 'workspace',
          objectId: req.params.id,
        }, req.accessToken);
      } catch (e) { /* ignore cleanup errors */ }
    }

    workspaces.delete(req.params.id);
    req.session.flash = { type: 'success', message: 'Workspace deleted' };
    res.redirect('/workspaces');
  } catch (err) {
    console.error('[workspaces] Delete error:', err);
    req.session.flash = { type: 'error', message: 'Failed to delete workspace' };
    res.redirect(`/workspaces/${req.params.id}`);
  }
});

// Share workspace
router.post('/:id/share', requirePermission('workspace', 'admin', 'id', wsOwner), async (req, res) => {
  try {
    const { user_id, relation } = req.body;
    if (!user_id || !relation) {
      req.session.flash = { type: 'error', message: 'User ID and role are required' };
      return res.redirect(`/workspaces/${req.params.id}`);
    }

    const validRelations = ['admin', 'member', 'viewer'];
    if (!validRelations.includes(relation)) {
      req.session.flash = { type: 'error', message: 'Invalid role' };
      return res.redirect(`/workspaces/${req.params.id}`);
    }

    await fga.addPermission(user_id, 'workspace', req.params.id, relation, req.accessToken);
    req.session.flash = { type: 'success', message: `Shared workspace as ${relation}!` };
    res.redirect(`/workspaces/${req.params.id}`);
  } catch (err) {
    console.error('[workspaces] Share error:', err);
    req.session.flash = { type: 'error', message: 'Failed to share workspace: ' + err.message };
    res.redirect(`/workspaces/${req.params.id}`);
  }
});

module.exports = router;
