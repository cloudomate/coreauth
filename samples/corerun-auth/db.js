const Database = require('better-sqlite3');
const path = require('path');

const DB_PATH = path.join(__dirname, 'corerun.db');

let db;

function getDb() {
  if (!db) {
    db = new Database(DB_PATH);
    db.pragma('journal_mode = WAL');
    db.pragma('foreign_keys = ON');
    initSchema();
  }
  return db;
}

function initSchema() {
  db.exec(`
    CREATE TABLE IF NOT EXISTS workspaces (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      region TEXT DEFAULT 'us-east-1',
      owner_id TEXT NOT NULL,
      owner_email TEXT,
      created_at TEXT DEFAULT (datetime('now')),
      updated_at TEXT DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS resources (
      id TEXT PRIMARY KEY,
      workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
      resource_type TEXT NOT NULL,
      name TEXT NOT NULL,
      status TEXT DEFAULT 'active',
      config TEXT DEFAULT '{}',
      owner_id TEXT NOT NULL,
      owner_email TEXT,
      created_at TEXT DEFAULT (datetime('now')),
      updated_at TEXT DEFAULT (datetime('now'))
    );

    CREATE TABLE IF NOT EXISTS fga_config (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    );
  `);
}

// Workspace helpers
const workspaces = {
  create(id, name, description, region, ownerId, ownerEmail) {
    const stmt = getDb().prepare(
      'INSERT INTO workspaces (id, name, description, region, owner_id, owner_email) VALUES (?, ?, ?, ?, ?, ?)'
    );
    return stmt.run(id, name, description || null, region || 'us-east-1', ownerId, ownerEmail);
  },

  getById(id) {
    return getDb().prepare('SELECT * FROM workspaces WHERE id = ?').get(id);
  },

  listAll() {
    return getDb().prepare('SELECT * FROM workspaces ORDER BY created_at DESC').all();
  },

  update(id, name, description) {
    const stmt = getDb().prepare(
      "UPDATE workspaces SET name = ?, description = ?, updated_at = datetime('now') WHERE id = ?"
    );
    return stmt.run(name, description, id);
  },

  delete(id) {
    return getDb().prepare('DELETE FROM workspaces WHERE id = ?').run(id);
  },
};

// Resource helpers
const resources = {
  create(id, workspaceId, resourceType, name, config, ownerId, ownerEmail) {
    const stmt = getDb().prepare(
      'INSERT INTO resources (id, workspace_id, resource_type, name, config, owner_id, owner_email) VALUES (?, ?, ?, ?, ?, ?, ?)'
    );
    return stmt.run(id, workspaceId, resourceType, name, JSON.stringify(config || {}), ownerId, ownerEmail);
  },

  getById(id) {
    const row = getDb().prepare('SELECT * FROM resources WHERE id = ?').get(id);
    if (row && row.config) {
      try { row.config = JSON.parse(row.config); } catch { row.config = {}; }
    }
    return row;
  },

  listByWorkspace(workspaceId, resourceType) {
    let q = 'SELECT * FROM resources WHERE workspace_id = ?';
    const params = [workspaceId];
    if (resourceType) {
      q += ' AND resource_type = ?';
      params.push(resourceType);
    }
    q += ' ORDER BY created_at DESC';
    return getDb().prepare(q).all(...params).map(row => {
      if (row.config) try { row.config = JSON.parse(row.config); } catch { row.config = {}; }
      return row;
    });
  },

  updateStatus(id, status) {
    const stmt = getDb().prepare(
      "UPDATE resources SET status = ?, updated_at = datetime('now') WHERE id = ?"
    );
    return stmt.run(status, id);
  },

  delete(id) {
    return getDb().prepare('DELETE FROM resources WHERE id = ?').run(id);
  },

  countByWorkspace(workspaceId) {
    const rows = getDb().prepare(
      'SELECT resource_type, COUNT(*) as count FROM resources WHERE workspace_id = ? GROUP BY resource_type'
    ).all(workspaceId);
    const counts = {};
    for (const row of rows) counts[row.resource_type] = row.count;
    return counts;
  },
};

// FGA config helpers
const fgaConfig = {
  get(key) {
    const row = getDb().prepare('SELECT value FROM fga_config WHERE key = ?').get(key);
    return row ? row.value : null;
  },

  set(key, value) {
    const stmt = getDb().prepare(
      'INSERT OR REPLACE INTO fga_config (key, value) VALUES (?, ?)'
    );
    return stmt.run(key, value);
  },
};

module.exports = { getDb, workspaces, resources, fgaConfig };
