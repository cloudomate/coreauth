const express = require('express');
const session = require('express-session');
const SqliteStore = require('better-sqlite3-session-store')(session);
const Database = require('better-sqlite3');
const path = require('path');

// Load .env manually (no dotenv dependency)
const fs = require('fs');
const envPath = path.join(__dirname, '.env');
if (fs.existsSync(envPath)) {
  fs.readFileSync(envPath, 'utf8').split('\n').forEach(line => {
    const trimmed = line.trim();
    if (trimmed && !trimmed.startsWith('#')) {
      const [key, ...rest] = trimmed.split('=');
      if (key && rest.length > 0) {
        process.env[key.trim()] = rest.join('=').trim();
      }
    }
  });
}

const { getDb } = require('./db');
const dashboardRoutes = require('./routes/dashboard');
const workspaceRoutes = require('./routes/workspaces');
const resourceRoutes = require('./routes/resources');
const adminRoutes = require('./routes/admin');
const apiRoutes = require('./routes/api');

const app = express();
const PORT = process.env.PORT || 3001;

// View engine
app.set('view engine', 'ejs');
app.set('views', path.join(__dirname, 'views'));

// Middleware
app.use(express.urlencoded({ extended: true }));
app.use(express.json());
app.use(express.static(path.join(__dirname, 'public')));

// Lightweight session for flash messages only (auth handled by proxy)
app.use(session({
  store: new SqliteStore({
    client: new Database(path.join(__dirname, 'sessions.db')),
    expired: { clear: true, intervalMs: 900000 },
  }),
  secret: process.env.SESSION_SECRET || 'dev-secret',
  resave: false,
  saveUninitialized: false,
  cookie: { maxAge: 24 * 60 * 60 * 1000 },
}));

// Read identity from proxy-injected X-CoreAuth-* headers
app.use((req, res, next) => {
  const userId = req.headers['x-coreauth-user-id'];
  const email = req.headers['x-coreauth-user-email'];

  if (userId && email) {
    req.user = {
      sub: userId,
      email: email,
      name: email.split('@')[0],
      tenant_id: req.headers['x-coreauth-tenant-id'] || null,
      tenant_slug: req.headers['x-coreauth-tenant-slug'] || null,
      role: req.headers['x-coreauth-role'] || 'member',
      is_platform_admin: req.headers['x-coreauth-is-platform-admin'] === 'true',
    };
    req.accessToken = req.headers['x-coreauth-token'] || null;
  }

  // Make user available in all views
  res.locals.user = req.user || null;
  res.locals.flash = req.session?.flash || null;
  if (req.session) delete req.session.flash;
  next();
});

// Root route — redirect based on auth status
app.get('/', (req, res) => {
  if (req.user) {
    return res.redirect('/dashboard');
  }
  res.redirect('/auth/login');
});

// Routes
app.use('/', dashboardRoutes);
app.use('/workspaces', workspaceRoutes);
app.use('/', resourceRoutes);
app.use('/admin', adminRoutes);
app.use('/api', apiRoutes);

// Error handler
app.use((err, req, res, _next) => {
  console.error('Unhandled error:', err);
  res.status(500).render('error', {
    title: 'Error',
    message: err.message || 'Something went wrong',
  });
});

// Start
async function start() {
  getDb();
  console.log('[db] SQLite initialized');
  console.log('[fga] FGA will be initialized on first authenticated request');

  app.listen(PORT, () => {
    console.log(`\n  CoreRun-Auth running on port ${PORT} (upstream for proxy)`);
    console.log(`  Access via proxy at ${process.env.APP_URL || 'http://localhost:4000'}`);
    console.log(`  CoreAuth API at ${process.env.COREAUTH_BASE_URL || 'http://localhost:8000'}\n`);
  });
}

start().catch(err => {
  console.error('Failed to start:', err);
  process.exit(1);
});
