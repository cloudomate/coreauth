const coreauth = require('./coreauth');
const { fgaConfig } = require('../db');

const STORE_NAME = process.env.FGA_STORE_NAME || 'corerun-auth';

// Cloud infrastructure authorization model — 14 types with AWS-like granular permissions
// Uses snake_case to match Rust backend struct field names
const AUTH_MODEL = {
  schema_version: '1.1',
  type_definitions: [
    { type: 'user', relations: {} },
    { type: 'application', relations: {} },
    {
      type: 'group',
      relations: {
        member: { this: { types: ['user', 'application'] } },
      },
      metadata: {
        relations: {
          member: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }] },
        },
      },
    },
    {
      type: 'workspace',
      relations: {
        admin: { this: { types: ['user', 'group#member'] } },
        member: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'admin' } },
        ]},
        viewer: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'member' } },
        ]},
        billing_admin: { this: { types: ['user'] } },
      },
      metadata: {
        relations: {
          admin: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          member: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          viewer: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          billing_admin: { directly_related_user_types: [{ type: 'user' }] },
        },
      },
    },
    // ── Compute ─────────────────────────────────────────────
    {
      type: 'compute_instance',
      relations: {
        workspace: { this: { types: ['workspace'] } },
        owner: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        operator: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        editor: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'operator' } },
        ]},
        viewer: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'editor' } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'viewer' } } },
        ]},
      },
      metadata: {
        relations: {
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          operator: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          editor: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          viewer: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
    {
      type: 'compute_function',
      relations: {
        workspace: { this: { types: ['workspace'] } },
        owner: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        deployer: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        invoker: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'deployer' } },
        ]},
        viewer: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'invoker' } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'viewer' } } },
        ]},
      },
      metadata: {
        relations: {
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          deployer: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          invoker: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          viewer: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
    // ── Storage ─────────────────────────────────────────────
    {
      type: 'storage_bucket',
      relations: {
        workspace: { this: { types: ['workspace'] } },
        owner: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        writer: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        reader: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'writer' } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'viewer' } } },
        ]},
        lister: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'reader' } },
        ]},
      },
      metadata: {
        relations: {
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          writer: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          reader: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          lister: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
    {
      type: 'storage_volume',
      relations: {
        workspace: { this: { types: ['workspace'] } },
        owner: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        attacher: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        viewer: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'attacher' } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'viewer' } } },
        ]},
      },
      metadata: {
        relations: {
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          attacher: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          viewer: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
    {
      type: 'storage_database',
      relations: {
        workspace: { this: { types: ['workspace'] } },
        owner: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        admin: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        writer: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'admin' } },
        ]},
        reader: { union: [
          { this: { types: ['user', 'application', 'group#member'] } },
          { computed_userset: { relation: 'writer' } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'viewer' } } },
        ]},
      },
      metadata: {
        relations: {
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          admin: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          writer: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
          reader: { directly_related_user_types: [{ type: 'user' }, { type: 'application' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
    // ── Networking ──────────────────────────────────────────
    {
      type: 'network_vpc',
      relations: {
        workspace: { this: { types: ['workspace'] } },
        owner: { union: [
          { this: { types: ['user', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        admin: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        viewer: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'admin' } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'viewer' } } },
        ]},
      },
      metadata: {
        relations: {
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          admin: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          viewer: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
    {
      type: 'network_subnet',
      relations: {
        vpc: { this: { types: ['network_vpc'] } },
        workspace: { this: { types: ['workspace'] } },
        owner: { union: [
          { this: { types: ['user', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'vpc' }, computed_userset: { relation: 'owner' } } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        admin: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        viewer: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'admin' } },
          { tuple_to_userset: { tupleset: { relation: 'vpc' }, computed_userset: { relation: 'viewer' } } },
        ]},
      },
      metadata: {
        relations: {
          vpc: { directly_related_user_types: [{ type: 'network_vpc' }] },
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          admin: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          viewer: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
    {
      type: 'network_firewall',
      relations: {
        workspace: { this: { types: ['workspace'] } },
        vpc: { this: { types: ['network_vpc'] } },
        owner: { union: [
          { this: { types: ['user', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        editor: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        viewer: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'editor' } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'viewer' } } },
        ]},
      },
      metadata: {
        relations: {
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          vpc: { directly_related_user_types: [{ type: 'network_vpc' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          editor: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          viewer: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
    {
      type: 'network_lb',
      relations: {
        workspace: { this: { types: ['workspace'] } },
        vpc: { this: { types: ['network_vpc'] } },
        owner: { union: [
          { this: { types: ['user', 'group#member'] } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'admin' } } },
        ]},
        editor: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'owner' } },
        ]},
        viewer: { union: [
          { this: { types: ['user', 'group#member'] } },
          { computed_userset: { relation: 'editor' } },
          { tuple_to_userset: { tupleset: { relation: 'workspace' }, computed_userset: { relation: 'viewer' } } },
        ]},
      },
      metadata: {
        relations: {
          workspace: { directly_related_user_types: [{ type: 'workspace' }] },
          vpc: { directly_related_user_types: [{ type: 'network_vpc' }] },
          owner: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          editor: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
          viewer: { directly_related_user_types: [{ type: 'user' }, { type: 'group', relation: 'member' }] },
        },
      },
    },
  ],
};

/**
 * Initialize FGA store and model (idempotent)
 */
async function initFga(accessToken) {
  let storeId = fgaConfig.get('store_id');

  if (storeId) {
    console.log(`[fga] Using existing store: ${storeId}`);
    return storeId;
  }

  console.log('[fga] Creating FGA store...');
  try {
    const store = await coreauth.request('POST', '/api/fga/stores', {
      body: { name: STORE_NAME, description: 'Cloud infrastructure authorization store' },
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    storeId = store.id;
    fgaConfig.set('store_id', storeId);
    console.log(`[fga] Store created: ${storeId}`);
  } catch (err) {
    if (err.status === 409) {
      console.log('[fga] Store already exists, fetching...');
      const stores = await coreauth.request('GET', '/api/fga/stores', {
        headers: { Authorization: `Bearer ${accessToken}` },
      });
      const existing = stores.find(s => s.name === STORE_NAME);
      if (existing) {
        storeId = existing.id;
        fgaConfig.set('store_id', storeId);
      } else {
        throw new Error('Could not find existing FGA store');
      }
    } else {
      throw err;
    }
  }

  // Write authorization model (wrapped in { schema: ... } to match WriteModelRequest)
  console.log('[fga] Writing cloud infrastructure authorization model...');
  try {
    const model = await coreauth.request('POST', `/api/fga/stores/${storeId}/models`, {
      body: { schema: AUTH_MODEL },
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    fgaConfig.set('model_version', String(model.version || model.id || '1'));
    console.log('[fga] Authorization model written (14 types, cloud infrastructure)');
  } catch (err) {
    console.warn('[fga] Model write warning (may already exist):', err.message);
  }

  return storeId;
}

function getStoreId() {
  return fgaConfig.get('store_id');
}

/**
 * Write a relation tuple (generic)
 */
async function writeTuple({ subjectType, subjectId, subjectRelation, relation, objectType, objectId }, accessToken) {
  const storeId = getStoreId();
  if (!storeId) throw new Error('FGA store not initialized');

  const tupleData = {
    subject_type: subjectType || 'user',
    subject_id: subjectId,
    relation,
    object_type: objectType,
    object_id: objectId,
  };
  if (subjectRelation) tupleData.subject_relation = subjectRelation;

  return coreauth.request('POST', `/api/fga/stores/${storeId}/tuples`, {
    body: { writes: [tupleData] },
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

/**
 * Delete a relation tuple (generic)
 */
async function deleteTuple({ subjectType, subjectId, subjectRelation, relation, objectType, objectId }, accessToken) {
  const storeId = getStoreId();
  if (!storeId) throw new Error('FGA store not initialized');

  const tupleData = {
    subject_type: subjectType || 'user',
    subject_id: subjectId,
    relation,
    object_type: objectType,
    object_id: objectId,
  };
  if (subjectRelation) tupleData.subject_relation = subjectRelation;

  return coreauth.request('POST', `/api/fga/stores/${storeId}/tuples`, {
    body: { deletes: [tupleData] },
    headers: { Authorization: `Bearer ${accessToken}` },
  });
}

/**
 * Check permission (generic — accepts objectType)
 */
async function checkPermission(subjectId, objectType, objectId, relation, accessToken) {
  const storeId = getStoreId();
  if (!storeId) return false;

  try {
    const result = await coreauth.request('POST', `/api/fga/stores/${storeId}/check`, {
      body: {
        subject_type: 'user',
        subject_id: subjectId,
        relation,
        object_type: objectType,
        object_id: objectId,
      },
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return result.allowed === true;
  } catch (err) {
    console.error('[fga] Permission check error:', err.message);
    return false;
  }
}

/**
 * Link a resource to its parent workspace (writes the workspace tupleset relation)
 */
async function linkResourceToWorkspace(resourceType, resourceId, workspaceId, accessToken) {
  return writeTuple({
    subjectType: 'userset',
    subjectId: workspaceId,
    relation: 'workspace',
    objectType: resourceType,
    objectId: resourceId,
  }, accessToken);
}

/**
 * Link a subnet to its VPC
 */
async function linkSubnetToVpc(subnetId, vpcId, accessToken) {
  return writeTuple({
    subjectType: 'userset',
    subjectId: vpcId,
    relation: 'vpc',
    objectType: 'network_subnet',
    objectId: subnetId,
  }, accessToken);
}

/**
 * List tuples for any object type
 */
async function listObjectTuples(objectType, objectId, accessToken) {
  const storeId = getStoreId();
  if (!storeId) return [];

  try {
    const tuples = await coreauth.request('GET',
      `/api/fga/stores/${storeId}/tuples?object_type=${objectType}&object_id=${objectId}`, {
      headers: { Authorization: `Bearer ${accessToken}` },
    });
    return Array.isArray(tuples) ? tuples : [];
  } catch (err) {
    console.error('[fga] List tuples error:', err.message);
    return [];
  }
}

/**
 * Grant a user a role on any object
 */
async function addPermission(userId, objectType, objectId, relation, accessToken) {
  return writeTuple({
    subjectType: 'user',
    subjectId: userId,
    relation,
    objectType,
    objectId,
  }, accessToken);
}

/**
 * Revoke a user's role on any object
 */
async function removePermission(userId, objectType, objectId, relation, accessToken) {
  return deleteTuple({
    subjectType: 'user',
    subjectId: userId,
    relation,
    objectType,
    objectId,
  }, accessToken);
}

module.exports = {
  AUTH_MODEL,
  initFga,
  getStoreId,
  writeTuple,
  deleteTuple,
  checkPermission,
  linkResourceToWorkspace,
  linkSubnetToVpc,
  listObjectTuples,
  addPermission,
  removePermission,
};
