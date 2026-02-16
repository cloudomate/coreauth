const { initFga, getStoreId } = require('../services/fga');

/**
 * Require authentication.
 * Behind the proxy, req.user is populated from X-CoreAuth-* headers
 * in the global middleware (server.js). If the proxy did not inject
 * headers, the user is not authenticated.
 */
function requireAuth(req, res, next) {
  if (!req.user || !req.accessToken) {
    return res.redirect('/auth/login?redirect=' + encodeURIComponent(req.originalUrl));
  }

  // Ensure FGA is initialized (idempotent)
  if (!getStoreId() && req.accessToken) {
    initFga(req.accessToken).catch(err => {
      console.error('[fga] Deferred init error:', err.message);
    });
  }

  next();
}

/**
 * Optional auth — user may or may not be present.
 * req.user is already populated by global middleware if headers present.
 */
function optionalAuth(req, res, next) {
  next();
}

module.exports = { requireAuth, optionalAuth };
