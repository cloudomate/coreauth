# Authorization

## Overview

CIAM implements a sophisticated authorization system combining:
- **RBAC** (Role-Based Access Control)
- **ReBAC** (Relationship-Based Access Control) - Zanzibar-style
- **ABAC** (Attribute-Based Access Control)

## Role-Based Access Control (RBAC)

### Roles

Roles are tenant-scoped and assigned to users.

**Common Roles:**
- `admin` - Tenant administrator
- `user` - Regular user
- `developer` - API access
- `support` - Customer support

### Role Assignment

Roles can be assigned:
1. Manually by tenant admins
2. Automatically via OIDC group sync
3. During invitation acceptance

## Relationship-Based Access Control (ReBAC)

### Zanzibar Model

CIAM implements Google's Zanzibar authorization model for fine-grained permissions.

### Relation Tuples

The core concept is the **relation tuple**:

```
subject has relation to object
```

**Format:**
```
{subject_type}:{subject_id}#{subject_relation}@{namespace}:{object_id}#{relation}
```

**Examples:**
```
user:alice#viewer@document:doc123
user:bob#editor@folder:projects
group:admins#member@organization:acme
application:api-service#reader@database:customers
```

### Tuple Components

| Component | Description | Example |
|-----------|-------------|---------|
| subject_type | Type of entity | user, application, group, userset |
| subject_id | ID of subject | alice, api-service |
| subject_relation | Optional relation | member, owner |
| namespace | Resource type | document, folder, organization |
| object_id | Resource ID | doc123, folder-1 |
| relation | Permission | viewer, editor, owner |

### Creating Relation Tuples

**Endpoint:** `POST /api/authz/tuples`

**Request:**
```json
{
  "tenant_id": "uuid",
  "namespace": "document",
  "object_id": "doc123",
  "relation": "viewer",
  "subject_type": "user",
  "subject_id": "alice",
  "subject_relation": null
}
```

**Response:**
```json
{
  "id": "uuid",
  "tenant_id": "uuid",
  "namespace": "document",
  "object_id": "doc123",
  "relation": "viewer",
  "subject_type": "user",
  "subject_id": "alice",
  "subject_relation": null,
  "created_at": "2024-01-01T00:00:00Z"
}
```

### Deleting Relation Tuples

**Endpoint:** `DELETE /api/authz/tuples`

**Request:** (Same as create)

### Querying Tuples

**Endpoint:** `POST /api/authz/tuples/query`

**Request:**
```json
{
  "tenant_id": "uuid",
  "namespace": "document",
  "object_id": "doc123",
  "relation": "viewer"
}
```

**Response:**
```json
[
  {
    "id": "uuid",
    "subject_type": "user",
    "subject_id": "alice",
    ...
  },
  {
    "id": "uuid",
    "subject_type": "group",
    "subject_id": "viewers",
    ...
  }
]
```

### Permission Checking

**Endpoint:** `POST /api/authz/check`

**Request:**
```json
{
  "tenant_id": "uuid",
  "subject_type": "user",
  "subject_id": "alice",
  "relation": "edit",
  "namespace": "document",
  "object_id": "doc123",
  "context": {}
}
```

**Response:**
```json
{
  "allowed": true,
  "reason": "Permission granted"
}
```

### Permission Resolution

The policy engine performs **graph traversal** to resolve permissions:

#### 1. Direct Check
```
Does tuple exist?
user:alice#editor@document:doc123
```

#### 2. Userset Check
```
Is user part of a group with permission?
user:alice#member@group:editors
group:editors#editor@document:doc123
```

#### 3. Hierarchical Check
```
Does user have parent relation?
user:alice#owner@document:doc123
(owner implies editor, viewer)
```

#### 4. Computed Usersets
```
Indirect relations via objects
user:alice#member@folder:projects#viewer@document:doc123
(all members of folder can view document)
```

### Example Permission Models

#### Document Management

```
// Alice is viewer of doc123
user:alice#viewer@document:doc123

// Bob is editor of doc123
user:bob#editor@document:doc123

// Engineering group has owner access
group:engineering#owner@document:doc123

// Alice is member of engineering group
user:alice#member@group:engineering
```

**Resolution:**
- Alice can view (direct)
- Alice can edit (via group ownership)
- Bob can edit (direct)

#### Folder Hierarchies

```
// Folder contains documents
folder:projects#parent@document:doc123

// Members of folder can view documents
folder:projects#member@document:doc123#viewer

// Alice is member of folder
user:alice#member@folder:projects
```

**Resolution:**
- Alice can view doc123 (via folder membership)

#### Organization Structure

```
// Organization owns resources
organization:acme#owner@database:customers

// Admins group has admin access to org
group:admins#admin@organization:acme

// Bob is member of admins
user:bob#member@group:admins
```

**Resolution:**
- Bob can access customers database (via org admin)

## Forward Authentication

### Overview

Forward auth allows downstream applications to delegate authorization to CIAM.

### Nginx Integration

**nginx.conf:**
```nginx
location /api {
    auth_request /auth;
    proxy_pass http://backend;
}

location = /auth {
    internal;
    proxy_pass http://ciam:8003/authz/forward-auth;
    proxy_pass_request_body off;
    proxy_set_header Content-Length "";
    proxy_set_header X-Tenant-ID $tenant_id;
    proxy_set_header X-Subject-Type "user";
    proxy_set_header X-Subject-ID $user_id;
    proxy_set_header X-Relation "access";
    proxy_set_header X-Namespace "api";
    proxy_set_header X-Object-ID $request_uri;
}
```

**Endpoint:** `POST /authz/forward-auth`

**Request:**
```json
{
  "tenant_id": "uuid",
  "subject_type": "user",
  "subject_id": "alice",
  "relation": "access",
  "namespace": "api",
  "object_id": "/users"
}
```

**Response:**
- `200 OK` - Access allowed
- `403 Forbidden` - Access denied

### Traefik Integration

**traefik.yml:**
```yaml
http:
  middlewares:
    ciam-auth:
      forwardAuth:
        address: http://ciam:8003/authz/forward-auth
        authResponseHeaders:
          - X-User-ID
          - X-Tenant-ID
```

**Endpoint:** `GET /authz/forward-auth`

**Headers:**
```
X-Tenant-ID: uuid
X-Subject-Type: user
X-Subject-ID: alice
X-Relation: access
X-Namespace: api
X-Object-ID: /users
```

## Service Principals

### Application Registration

**Endpoint:** `POST /api/applications`

**Request:**
```json
{
  "tenant_id": "uuid",
  "name": "API Service",
  "description": "Backend API service",
  "application_type": "service",
  "redirect_uris": [],
  "allowed_scopes": ["read", "write"],
  "metadata": {}
}
```

**Response:**
```json
{
  "application": {
    "id": "uuid",
    "client_id": "app_abc123...",
    "application_type": "service",
    ...
  },
  "client_secret": "secret_xyz789..."
}
```

**IMPORTANT:** Save the `client_secret` - it's only shown once!

### Client Credentials Authentication

**Endpoint:** `POST /api/applications/authenticate`

**Request:**
```json
{
  "client_id": "app_abc123...",
  "client_secret": "secret_xyz789..."
}
```

**Response:**
```json
{
  "application": { ... },
  "access_token": "app_token_..."
}
```

### Application Types

| Type | Description | Use Case |
|------|-------------|----------|
| service | Machine-to-machine | Backend services, APIs |
| webapp | Confidential client | Server-side web apps |
| spa | Public client | Single-page applications |
| native | Mobile/desktop | Mobile and desktop apps |

### Application Permissions

Applications can be subjects in relation tuples:

```
// API service can read customers database
application:api-service#reader@database:customers

// Web app can write to logs
application:web-app#writer@logs:system
```

### Secret Rotation

**Endpoint:** `POST /api/applications/{app_id}/tenants/{tenant_id}/rotate-secret`

**Response:**
```json
{
  "application": { ... },
  "client_secret": "new_secret_..."
}
```

**Best Practice:** Rotate secrets regularly (e.g., every 90 days)

## Caching

### Authorization Check Cache

- **Storage:** Redis
- **TTL:** 60 seconds
- **Key Format:** `authz:check:{tenant_id}:{subject_type}:{subject_id}:{relation}:{namespace}:{object_id}`

### Cache Invalidation

**Endpoint:** `POST /api/authz/cache/invalidate`

**Automatic Invalidation:**
- When tuples are created
- When tuples are deleted
- When roles are modified

## Expansion

### Expand Relation

Show all subjects with a relation to an object.

**Endpoint:** `GET /api/authz/expand/{tenant_id}/{namespace}/{object_id}/{relation}`

**Response:**
```json
{
  "subjects": [
    {
      "subject_type": "user",
      "subject_id": "alice",
      "via_relation": null
    },
    {
      "subject_type": "group",
      "subject_id": "viewers",
      "via_relation": "member"
    }
  ]
}
```

## Best Practices

### Tuple Design

1. **Granular Permissions**
   ```
   ✅ user:alice#editor@document:doc123
   ❌ user:alice#all_access@system:everything
   ```

2. **Use Groups**
   ```
   ✅ user:alice#member@group:editors
      group:editors#editor@document:doc123
   ❌ user:alice#editor@document:doc1
      user:alice#editor@document:doc2
      user:alice#editor@document:doc3
   ```

3. **Hierarchical Structures**
   ```
   ✅ folder:projects#member@document:doc123#viewer
   ❌ Flat, disconnected tuples
   ```

### Performance

1. **Cache Checks** - Most checks cached for 60s
2. **Batch Operations** - Create multiple tuples together
3. **Index Usage** - Queries use PostgreSQL indexes
4. **Avoid Deep Hierarchies** - Keep graph depth < 10

### Security

1. **Validate Inputs** - Always validate tenant context
2. **Principle of Least Privilege** - Grant minimum required permissions
3. **Regular Audits** - Review tuple grants periodically
4. **Monitor Access** - Log authorization decisions
5. **Rotate Secrets** - Application secrets every 90 days

## Example Use Cases

### Multi-Tenant SaaS

```
// Tenant isolation
user:alice#member@tenant:acme
user:alice#admin@tenant:acme

// Resource access within tenant
user:alice#owner@workspace:engineering
workspace:engineering#member@project:api-v2#editor
```

### Document Management System

```
// Folder hierarchy
folder:projects#parent@folder:2024
folder:2024#parent@document:roadmap

// User access
user:alice#member@folder:projects#viewer
// Alice can view all nested documents

// Group access
group:leadership#owner@folder:projects
user:bob#member@group:leadership
// Bob can edit all documents as owner
```

### API Gateway

```
// Service authentication
application:api-gateway#caller@service:user-service

// User access via gateway
user:alice#reader@api:users
// Alice can call GET /api/users

user:bob#writer@api:users
// Bob can call POST/PUT/DELETE /api/users
```

## Advanced Topics

### Computed Usersets

Define permissions based on relationships:

```
// Format: {object}#{relation}@{target}#{target_relation}
folder:projects#member@document:doc123#viewer
```

Means: "All members of folder 'projects' can view document 'doc123'"

### Hierarchical Permissions

Define relation hierarchies:

```
owner > editor > viewer
```

- Owner implies editor and viewer permissions
- Editor implies viewer permissions
- Viewer has only view permissions

(Note: Hierarchy implementation pending in codebase)

### Wildcard Permissions

Use usersets for wildcard-style permissions:

```
organization:acme#member@*:*#viewer
```

Means: "All members of organization 'acme' can view all resources"

(Note: Wildcard syntax pending - use computed usersets)
