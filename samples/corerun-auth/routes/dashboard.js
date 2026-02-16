const express = require('express');
const { requireAuth } = require('../middleware/auth');
const { workspaces, resources } = require('../db');
const fga = require('../services/fga');

const router = express.Router();

router.get('/dashboard', requireAuth, async (req, res) => {
  try {
    const allWorkspaces = workspaces.listAll();
    const accessible = [];

    for (const ws of allWorkspaces) {
      if (ws.owner_id === req.user.sub) {
        accessible.push({ ...ws, role: 'admin' });
        continue;
      }
      const canView = await fga.checkPermission(req.user.sub, 'workspace', ws.id, 'viewer', req.accessToken);
      if (canView) {
        const isAdmin = await fga.checkPermission(req.user.sub, 'workspace', ws.id, 'admin', req.accessToken);
        accessible.push({ ...ws, role: isAdmin ? 'admin' : 'viewer' });
      }
    }

    // Add resource counts
    for (const ws of accessible) {
      const counts = resources.countByWorkspace(ws.id);
      ws.computeCount = (counts.compute_instance || 0) + (counts.compute_function || 0);
      ws.storageCount = (counts.storage_bucket || 0) + (counts.storage_volume || 0) + (counts.storage_database || 0);
      ws.networkCount = (counts.network_vpc || 0) + (counts.network_subnet || 0) + (counts.network_firewall || 0) + (counts.network_lb || 0);
      ws.totalCount = ws.computeCount + ws.storageCount + ws.networkCount;
    }

    res.render('dashboard', { title: 'Dashboard', workspaces: accessible });
  } catch (err) {
    console.error('[dashboard] Error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load dashboard' });
  }
});

module.exports = router;
