const { checkPermission } = require('../services/fga');

/**
 * Express middleware factory: require FGA permission on a resource.
 * @param {string} objectType - The FGA object type (e.g., 'workspace', 'compute_instance')
 * @param {string} relation - The required relation (e.g., 'viewer', 'operator', 'owner')
 * @param {string} paramName - The route param containing the object ID (default: 'id')
 * @param {object} options - Optional settings
 * @param {function} options.getOwnerId - Function (objectId) => ownerId to bypass FGA for resource owners
 */
function requirePermission(objectType, relation, paramName = 'id', { getOwnerId } = {}) {
  return async (req, res, next) => {
    const objectId = req.params[paramName];
    const userId = req.user.sub;
    const accessToken = req.accessToken;

    if (!objectId || !userId) {
      return res.status(400).render('error', {
        title: 'Bad Request',
        message: 'Missing resource or user information',
      });
    }

    // Owner bypass: resource owners always have access
    if (getOwnerId) {
      try {
        const ownerId = getOwnerId(objectId);
        if (ownerId && ownerId === userId) {
          return next();
        }
      } catch (e) { /* fall through to FGA check */ }
    }

    try {
      const allowed = await checkPermission(userId, objectType, objectId, relation, accessToken);
      if (allowed) {
        return next();
      }
      return res.status(403).render('error', {
        title: 'Access Denied',
        message: `You don't have '${relation}' access to this ${objectType.replace(/_/g, ' ')}.`,
      });
    } catch (err) {
      console.error('[fga] Permission check failed:', err.message);
      return res.status(500).render('error', {
        title: 'Error',
        message: 'Failed to check permissions',
      });
    }
  };
}

module.exports = { requirePermission };
