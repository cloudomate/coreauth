const express = require('express');
const { requireAuth } = require('../middleware/auth');
const coreauth = require('../services/coreauth');

const router = express.Router();

router.use(requireAuth);

// Search users by email (for sharing)
router.get('/users/search', async (req, res) => {
  try {
    const { q } = req.query;
    const tenantId = req.user.tenant_id;
    if (!q || !tenantId) {
      return res.json([]);
    }

    let users = [];
    try {
      const result = await coreauth.listUsers(tenantId, req.accessToken);
      users = Array.isArray(result) ? result : result.users || [];
    } catch (err) {
      console.error('[api] Search users error:', err.message);
      return res.json([]);
    }

    // Filter by email match
    const query = q.toLowerCase();
    const matches = users
      .filter(u => u.email?.toLowerCase().includes(query) && u.id !== req.user.sub)
      .slice(0, 10)
      .map(u => ({ id: u.id, email: u.email, name: u.full_name || u.email }));

    res.json(matches);
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

module.exports = router;
