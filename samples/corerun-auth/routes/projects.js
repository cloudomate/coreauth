const express = require('express');
const { v4: uuidv4 } = require('uuid');
const { requireAuth } = require('../middleware/auth');
const { requirePermission } = require('../middleware/fga');
const { projects } = require('../db');
const fga = require('../services/fga');

const router = express.Router();

// All project routes require auth
router.use(requireAuth);

// New project form
router.get('/new', (req, res) => {
  res.render('project-form', {
    title: 'New Project',
    project: null,
    action: '/projects',
  });
});

// Create project
router.post('/', async (req, res) => {
  try {
    const { name, description } = req.body;
    if (!name || !name.trim()) {
      req.session.flash = { type: 'error', message: 'Project name is required' };
      return res.redirect('/projects/new');
    }

    const id = uuidv4();
    projects.create(id, name.trim(), description?.trim() || null, req.user.sub, req.user.email);

    // Write FGA tuple: user is owner of this project
    try {
      await fga.addProjectOwner(req.user.sub, id, req.accessToken);
    } catch (err) {
      console.error('[fga] Failed to write owner tuple:', err.message);
      // Project still created locally, FGA can be retried
    }

    req.session.flash = { type: 'success', message: `Project "${name}" created!` };
    res.redirect(`/projects/${id}`);
  } catch (err) {
    console.error('[projects] Create error:', err);
    req.session.flash = { type: 'error', message: 'Failed to create project' };
    res.redirect('/projects/new');
  }
});

// View project (requires viewer permission)
router.get('/:id', requirePermission('viewer'), async (req, res) => {
  try {
    const project = projects.getById(req.params.id);
    if (!project) {
      return res.status(404).render('error', { title: 'Not Found', message: 'Project not found' });
    }

    // Get collaborators from FGA
    const tuples = await fga.listProjectTuples(project.id, req.accessToken);
    const collaborators = tuples
      .filter(t => t.subject_type === 'user')
      .map(t => ({
        userId: t.subject_id,
        relation: t.relation,
      }));

    // Check current user's permission level
    const isOwner = await fga.checkPermission(req.user.sub, project.id, 'owner', req.accessToken);
    const isEditor = isOwner || await fga.checkPermission(req.user.sub, project.id, 'editor', req.accessToken);

    res.render('project-detail', {
      title: project.name,
      project,
      collaborators,
      isOwner,
      isEditor,
    });
  } catch (err) {
    console.error('[projects] View error:', err);
    res.render('error', { title: 'Error', message: 'Failed to load project' });
  }
});

// Edit project form (requires editor permission)
router.get('/:id/edit', requirePermission('editor'), (req, res) => {
  const project = projects.getById(req.params.id);
  if (!project) {
    return res.status(404).render('error', { title: 'Not Found', message: 'Project not found' });
  }
  res.render('project-form', {
    title: `Edit: ${project.name}`,
    project,
    action: `/projects/${project.id}`,
  });
});

// Update project (requires editor permission)
router.post('/:id', requirePermission('editor'), (req, res) => {
  try {
    const { name, description } = req.body;
    if (!name || !name.trim()) {
      req.session.flash = { type: 'error', message: 'Project name is required' };
      return res.redirect(`/projects/${req.params.id}/edit`);
    }
    projects.update(req.params.id, name.trim(), description?.trim() || null);
    req.session.flash = { type: 'success', message: 'Project updated!' };
    res.redirect(`/projects/${req.params.id}`);
  } catch (err) {
    console.error('[projects] Update error:', err);
    req.session.flash = { type: 'error', message: 'Failed to update project' };
    res.redirect(`/projects/${req.params.id}/edit`);
  }
});

// Delete project (requires owner permission)
router.post('/:id/delete', requirePermission('owner'), async (req, res) => {
  try {
    // Remove FGA tuples
    const tuples = await fga.listProjectTuples(req.params.id, req.accessToken);
    for (const t of tuples) {
      try {
        await fga.deleteTuple(t.subject_id, t.relation, req.params.id, req.accessToken);
      } catch (err) {
        console.warn('[fga] Tuple delete warning:', err.message);
      }
    }

    projects.delete(req.params.id);
    req.session.flash = { type: 'success', message: 'Project deleted' };
    res.redirect('/dashboard');
  } catch (err) {
    console.error('[projects] Delete error:', err);
    req.session.flash = { type: 'error', message: 'Failed to delete project' };
    res.redirect(`/projects/${req.params.id}`);
  }
});

// Share project (requires owner permission)
router.post('/:id/share', requirePermission('owner'), async (req, res) => {
  try {
    const { user_id, relation } = req.body;
    if (!user_id || !relation) {
      req.session.flash = { type: 'error', message: 'User ID and role are required' };
      return res.redirect(`/projects/${req.params.id}`);
    }

    const validRelations = ['editor', 'viewer'];
    if (!validRelations.includes(relation)) {
      req.session.flash = { type: 'error', message: 'Invalid role. Must be editor or viewer.' };
      return res.redirect(`/projects/${req.params.id}`);
    }

    await fga.writeTuple(user_id, relation, req.params.id, req.accessToken);
    req.session.flash = { type: 'success', message: `Shared as ${relation}!` };
    res.redirect(`/projects/${req.params.id}`);
  } catch (err) {
    console.error('[projects] Share error:', err);
    req.session.flash = { type: 'error', message: 'Failed to share project: ' + err.message };
    res.redirect(`/projects/${req.params.id}`);
  }
});

module.exports = router;
