const express = require('express');
const { v4: uuidv4 } = require('uuid');
const { requireAuth } = require('../middleware/auth');
const { requirePermission } = require('../middleware/fga');
const { resources, workspaces } = require('../db');
const fga = require('../services/fga');

const router = express.Router();
router.use(requireAuth);

// Resource type configuration — drives forms, actions, and permissions
const RESOURCE_TYPES = {
  compute_instance: {
    label: 'Compute Instance', plural: 'Instances', category: 'compute', icon: 'server',
    roles: ['owner', 'operator', 'editor', 'viewer'],
    actions: {
      start:   { label: 'Start',   requires: 'operator' },
      stop:    { label: 'Stop',    requires: 'operator' },
      restart: { label: 'Restart', requires: 'operator' },
    },
    statuses: ['running', 'stopped', 'terminated', 'pending'],
    fields: [
      { name: 'instance_type', label: 'Instance Type', type: 'select', options: ['t2.micro', 't2.small', 't2.medium', 't2.large', 'c5.xlarge'] },
      { name: 'ami', label: 'Image (AMI)', type: 'text', default: 'ami-ubuntu-22.04' },
    ],
  },
  compute_function: {
    label: 'Function', plural: 'Functions', category: 'compute', icon: 'zap',
    roles: ['owner', 'deployer', 'invoker', 'viewer'],
    actions: {
      deploy: { label: 'Deploy', requires: 'deployer' },
      invoke: { label: 'Invoke', requires: 'invoker' },
    },
    statuses: ['active', 'inactive', 'failed'],
    fields: [
      { name: 'runtime', label: 'Runtime', type: 'select', options: ['nodejs20.x', 'python3.12', 'go1.x', 'rust'] },
      { name: 'memory_mb', label: 'Memory (MB)', type: 'number', default: '256' },
      { name: 'timeout_seconds', label: 'Timeout (s)', type: 'number', default: '30' },
    ],
  },
  storage_bucket: {
    label: 'Storage Bucket', plural: 'Buckets', category: 'storage', icon: 'archive',
    roles: ['owner', 'writer', 'reader', 'lister'],
    actions: {},
    statuses: ['active', 'archived'],
    fields: [
      { name: 'access_level', label: 'Access Level', type: 'select', options: ['private', 'public-read', 'public-read-write'] },
      { name: 'versioning', label: 'Versioning', type: 'select', options: ['enabled', 'disabled'] },
    ],
  },
  storage_volume: {
    label: 'Block Volume', plural: 'Volumes', category: 'storage', icon: 'hard-drive',
    roles: ['owner', 'attacher', 'viewer'],
    actions: {
      attach: { label: 'Attach', requires: 'attacher' },
      detach: { label: 'Detach', requires: 'attacher' },
    },
    statuses: ['available', 'in-use', 'detaching'],
    fields: [
      { name: 'size_gb', label: 'Size (GB)', type: 'number', default: '50' },
      { name: 'volume_type', label: 'Type', type: 'select', options: ['ssd', 'hdd', 'nvme'] },
    ],
  },
  storage_database: {
    label: 'Managed Database', plural: 'Databases', category: 'storage', icon: 'database',
    roles: ['owner', 'admin', 'writer', 'reader'],
    actions: {
      start: { label: 'Start', requires: 'admin' },
      stop:  { label: 'Stop',  requires: 'admin' },
    },
    statuses: ['running', 'stopped', 'maintenance'],
    fields: [
      { name: 'engine', label: 'Engine', type: 'select', options: ['postgresql', 'mysql', 'redis', 'mongodb'] },
      { name: 'version', label: 'Version', type: 'text', default: '16' },
      { name: 'instance_class', label: 'Instance Class', type: 'select', options: ['db.t3.micro', 'db.t3.small', 'db.r5.large'] },
    ],
  },
  network_vpc: {
    label: 'VPC', plural: 'VPCs', category: 'network', icon: 'globe',
    roles: ['owner', 'admin', 'viewer'],
    actions: {},
    statuses: ['active', 'deleting'],
    fields: [
      { name: 'cidr_block', label: 'CIDR Block', type: 'text', default: '10.0.0.0/16' },
    ],
  },
  network_subnet: {
    label: 'Subnet', plural: 'Subnets', category: 'network', icon: 'git-branch',
    roles: ['owner', 'admin', 'viewer'],
    actions: {},
    statuses: ['active'],
    fields: [
      { name: 'cidr_block', label: 'CIDR Block', type: 'text', default: '10.0.1.0/24' },
      { name: 'availability_zone', label: 'AZ', type: 'select', options: ['us-east-1a', 'us-east-1b', 'us-east-1c'] },
    ],
  },
  network_firewall: {
    label: 'Firewall', plural: 'Firewalls', category: 'network', icon: 'shield',
    roles: ['owner', 'editor', 'viewer'],
    actions: {},
    statuses: ['active', 'disabled'],
    fields: [],
  },
  network_lb: {
    label: 'Load Balancer', plural: 'Load Balancers', category: 'network', icon: 'activity',
    roles: ['owner', 'editor', 'viewer'],
    actions: {},
    statuses: ['active', 'provisioning', 'draining'],
    fields: [
      { name: 'lb_type', label: 'Type', type: 'select', options: ['application', 'network', 'gateway'] },
      { name: 'scheme', label: 'Scheme', type: 'select', options: ['internet-facing', 'internal'] },
    ],
  },
};

// Make RESOURCE_TYPES available to views
router.use((req, res, next) => {
  res.locals.RESOURCE_TYPES = RESOURCE_TYPES;
  next();
});

// List resources in a workspace
router.get('/workspaces/:wid/resources/:type', async (req, res) => {
  const { wid, type } = req.params;
  const typeConfig = RESOURCE_TYPES[type];
  if (!typeConfig) return res.status(404).render('error', { title: 'Not Found', message: 'Unknown resource type' });

  const ws = workspaces.getById(wid);
  if (!ws) return res.status(404).render('error', { title: 'Not Found', message: 'Workspace not found' });

  const items = resources.listByWorkspace(wid, type);
  const isAdmin = await fga.checkPermission(req.user.sub, 'workspace', wid, 'admin', req.accessToken);

  res.render('resource-list', {
    title: `${typeConfig.plural} — ${ws.name}`,
    workspace: ws,
    resourceType: type,
    typeConfig,
    resources: items,
    isAdmin,
  });
});

// New resource form
router.get('/workspaces/:wid/resources/:type/new', async (req, res) => {
  const { wid, type } = req.params;
  const typeConfig = RESOURCE_TYPES[type];
  if (!typeConfig) return res.status(404).render('error', { title: 'Not Found', message: 'Unknown resource type' });

  const ws = workspaces.getById(wid);
  if (!ws) return res.status(404).render('error', { title: 'Not Found', message: 'Workspace not found' });

  res.render('resource-form', {
    title: `New ${typeConfig.label}`,
    workspace: ws,
    resourceType: type,
    typeConfig,
    resource: null,
  });
});

// Create resource
router.post('/workspaces/:wid/resources/:type', async (req, res) => {
  const { wid, type } = req.params;
  const typeConfig = RESOURCE_TYPES[type];
  if (!typeConfig) return res.status(400).render('error', { title: 'Bad Request', message: 'Unknown resource type' });

  try {
    const { name } = req.body;
    if (!name || !name.trim()) {
      req.session.flash = { type: 'error', message: 'Name is required' };
      return res.redirect(`/workspaces/${wid}/resources/${type}/new`);
    }

    // Build config from type-specific fields
    const config = {};
    for (const field of typeConfig.fields) {
      if (req.body[field.name] !== undefined && req.body[field.name] !== '') {
        config[field.name] = req.body[field.name];
      } else if (field.default) {
        config[field.name] = field.default;
      }
    }

    const id = uuidv4();
    resources.create(id, wid, type, name.trim(), config, req.user.sub, req.user.email);

    // Write FGA tuples: owner + workspace link
    try {
      await fga.addPermission(req.user.sub, type, id, 'owner', req.accessToken);
      await fga.linkResourceToWorkspace(type, id, wid, req.accessToken);
    } catch (err) {
      console.error('[fga] Failed to write resource tuples:', err.message);
    }

    req.session.flash = { type: 'success', message: `${typeConfig.label} "${name}" created!` };
    res.redirect(`/resources/${type}/${id}`);
  } catch (err) {
    console.error('[resources] Create error:', err);
    req.session.flash = { type: 'error', message: 'Failed to create resource' };
    res.redirect(`/workspaces/${wid}/resources/${type}/new`);
  }
});

// View resource detail
router.get('/resources/:type/:id', async (req, res) => {
  const { type, id } = req.params;
  const typeConfig = RESOURCE_TYPES[type];
  if (!typeConfig) return res.status(404).render('error', { title: 'Not Found', message: 'Unknown resource type' });

  const resource = resources.getById(id);
  if (!resource) return res.status(404).render('error', { title: 'Not Found', message: 'Resource not found' });

  // Check viewer permission
  const canView = await fga.checkPermission(req.user.sub, type, id, 'viewer', req.accessToken);
  if (!canView) {
    return res.status(403).render('error', { title: 'Access Denied', message: `You don't have viewer access to this ${typeConfig.label}.` });
  }

  const ws = workspaces.getById(resource.workspace_id);

  // Get collaborators from FGA
  const tuples = await fga.listObjectTuples(type, id, req.accessToken);
  const collaborators = tuples
    .filter(t => (t.subject_type === 'user' || t.subject_type === 'User'))
    .map(t => ({ userId: t.subject_id, relation: t.relation }));

  // Check permission levels for action buttons
  const isOwner = await fga.checkPermission(req.user.sub, type, id, 'owner', req.accessToken);
  const userPermissions = {};
  for (const [action, cfg] of Object.entries(typeConfig.actions)) {
    userPermissions[action] = await fga.checkPermission(req.user.sub, type, id, cfg.requires, req.accessToken);
  }

  res.render('resource-detail', {
    title: `${resource.name} — ${typeConfig.label}`,
    resource,
    workspace: ws,
    resourceType: type,
    typeConfig,
    collaborators,
    isOwner,
    userPermissions,
  });
});

// Perform action on resource (start, stop, restart, deploy, etc.)
router.post('/resources/:type/:id/action', async (req, res) => {
  const { type, id } = req.params;
  const { action } = req.body;
  const typeConfig = RESOURCE_TYPES[type];
  if (!typeConfig) return res.status(404).render('error', { title: 'Not Found', message: 'Unknown resource type' });

  const actionConfig = typeConfig.actions[action];
  if (!actionConfig) {
    req.session.flash = { type: 'error', message: 'Unknown action' };
    return res.redirect(`/resources/${type}/${id}`);
  }

  // Check permission
  const allowed = await fga.checkPermission(req.user.sub, type, id, actionConfig.requires, req.accessToken);
  if (!allowed) {
    return res.status(403).render('error', {
      title: 'Access Denied',
      message: `You need '${actionConfig.requires}' permission to ${action} this resource.`,
    });
  }

  // Update resource status based on action
  const statusMap = {
    start: 'running', stop: 'stopped', restart: 'running',
    deploy: 'active', invoke: null,
    attach: 'in-use', detach: 'available',
  };
  const newStatus = statusMap[action];
  if (newStatus) {
    resources.updateStatus(id, newStatus);
  }

  req.session.flash = { type: 'success', message: `Action '${action}' performed successfully!` };
  res.redirect(`/resources/${type}/${id}`);
});

// Share resource
router.post('/resources/:type/:id/share', async (req, res) => {
  const { type, id } = req.params;
  const typeConfig = RESOURCE_TYPES[type];
  if (!typeConfig) return res.status(404).render('error', { title: 'Not Found', message: 'Unknown resource type' });

  // Check owner permission
  const isOwner = await fga.checkPermission(req.user.sub, type, id, 'owner', req.accessToken);
  if (!isOwner) {
    return res.status(403).render('error', { title: 'Access Denied', message: 'Only owners can share resources.' });
  }

  try {
    const { user_id, relation } = req.body;
    if (!user_id || !relation) {
      req.session.flash = { type: 'error', message: 'User ID and role are required' };
      return res.redirect(`/resources/${type}/${id}`);
    }

    if (!typeConfig.roles.includes(relation)) {
      req.session.flash = { type: 'error', message: `Invalid role. Must be one of: ${typeConfig.roles.join(', ')}` };
      return res.redirect(`/resources/${type}/${id}`);
    }

    await fga.addPermission(user_id, type, id, relation, req.accessToken);
    req.session.flash = { type: 'success', message: `Shared as ${relation}!` };
    res.redirect(`/resources/${type}/${id}`);
  } catch (err) {
    console.error('[resources] Share error:', err);
    req.session.flash = { type: 'error', message: 'Failed to share: ' + err.message };
    res.redirect(`/resources/${type}/${id}`);
  }
});

// Delete resource
router.post('/resources/:type/:id/delete', async (req, res) => {
  const { type, id } = req.params;

  // Check owner permission
  const isOwner = await fga.checkPermission(req.user.sub, type, id, 'owner', req.accessToken);
  if (!isOwner) {
    return res.status(403).render('error', { title: 'Access Denied', message: 'Only owners can delete resources.' });
  }

  const resource = resources.getById(id);
  const wid = resource?.workspace_id;

  // Clean up FGA tuples
  const tuples = await fga.listObjectTuples(type, id, req.accessToken);
  for (const t of tuples) {
    try {
      await fga.deleteTuple({
        subjectType: t.subject_type?.toLowerCase(),
        subjectId: t.subject_id,
        subjectRelation: t.subject_relation,
        relation: t.relation,
        objectType: type,
        objectId: id,
      }, req.accessToken);
    } catch (e) { /* ignore cleanup errors */ }
  }

  resources.delete(id);
  req.session.flash = { type: 'success', message: 'Resource deleted' };
  res.redirect(wid ? `/workspaces/${wid}` : '/dashboard');
});

module.exports = router;
module.exports.RESOURCE_TYPES = RESOURCE_TYPES;
