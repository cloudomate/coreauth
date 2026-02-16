use ciam_models::{Connection, ConnectionScope, CreateConnection, UpdateConnection};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConnectionRepository {
    pool: PgPool,
}

impl ConnectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new connection
    pub async fn create(&self, request: CreateConnection) -> Result<Connection, sqlx::Error> {
        let scope_str = match request.scope {
            ConnectionScope::Platform => "platform",
            ConnectionScope::Organization => "organization",
        };

        let connection = sqlx::query_as!(
            Connection,
            r#"
            INSERT INTO connections (name, type, scope, tenant_id, config, is_enabled)
            VALUES ($1, $2, $3, $4, $5, true)
            RETURNING
                id,
                name,
                type as connection_type,
                scope as "scope: ConnectionScope",
                tenant_id as organization_id,
                config,
                is_enabled,
                created_at,
                updated_at
            "#,
            request.name,
            request.connection_type,
            scope_str,
            request.organization_id,
            request.config
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(connection)
    }

    /// Get connection by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Connection>, sqlx::Error> {
        let connection = sqlx::query_as!(
            Connection,
            r#"
            SELECT
                id,
                name,
                type as connection_type,
                scope as "scope: ConnectionScope",
                tenant_id as organization_id,
                config,
                is_enabled,
                created_at,
                updated_at
            FROM connections
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(connection)
    }

    /// Get all platform-level connections
    pub async fn get_platform_connections(&self) -> Result<Vec<Connection>, sqlx::Error> {
        let connections = sqlx::query_as!(
            Connection,
            r#"
            SELECT
                id,
                name,
                type as connection_type,
                scope as "scope: ConnectionScope",
                tenant_id as organization_id,
                config,
                is_enabled,
                created_at,
                updated_at
            FROM connections
            WHERE scope = 'platform' AND is_enabled = true
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(connections)
    }

    /// Get all connections for an organization
    pub async fn get_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<Connection>, sqlx::Error> {
        let connections = sqlx::query_as!(
            Connection,
            r#"
            SELECT
                id,
                name,
                type as connection_type,
                scope as "scope: ConnectionScope",
                tenant_id as organization_id,
                config,
                is_enabled,
                created_at,
                updated_at
            FROM connections
            WHERE tenant_id = $1 AND is_enabled = true
            ORDER BY created_at ASC
            "#,
            organization_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(connections)
    }

    /// Get primary SSO connection for an organization (if exists)
    pub async fn get_org_sso_connection(
        &self,
        organization_id: Uuid,
    ) -> Result<Option<Connection>, sqlx::Error> {
        let connection = sqlx::query_as!(
            Connection,
            r#"
            SELECT
                id,
                name,
                type as connection_type,
                scope as "scope: ConnectionScope",
                tenant_id as organization_id,
                config,
                is_enabled,
                created_at,
                updated_at
            FROM connections
            WHERE tenant_id = $1
              AND scope = 'organization'
              AND type IN ('oidc', 'saml', 'oauth2')
              AND is_enabled = true
            ORDER BY created_at ASC
            LIMIT 1
            "#,
            organization_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(connection)
    }

    /// Get database connection (platform-level)
    pub async fn get_database_connection(&self) -> Result<Option<Connection>, sqlx::Error> {
        let connection = sqlx::query_as!(
            Connection,
            r#"
            SELECT
                id,
                name,
                type as connection_type,
                scope as "scope: ConnectionScope",
                tenant_id as organization_id,
                config,
                is_enabled,
                created_at,
                updated_at
            FROM connections
            WHERE scope = 'platform'
              AND type = 'database'
              AND is_enabled = true
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(connection)
    }

    /// Resolve connection for user login
    /// Priority: Org-level SSO > Platform SSO > Database
    pub async fn resolve_for_login(
        &self,
        organization_id: Option<Uuid>,
    ) -> Result<Vec<Connection>, sqlx::Error> {
        let mut connections = Vec::new();

        // If organization provided, try to get org-level connection first
        if let Some(org_id) = organization_id {
            if let Some(org_conn) = self.get_org_sso_connection(org_id).await? {
                connections.push(org_conn);
            }
        }

        // Add platform connections as fallback
        let platform_conns = self.get_platform_connections().await?;
        connections.extend(platform_conns);

        Ok(connections)
    }

    /// Update connection
    pub async fn update(
        &self,
        id: Uuid,
        updates: UpdateConnection,
    ) -> Result<Connection, sqlx::Error> {
        // Build dynamic update query
        let mut query = String::from("UPDATE connections SET ");
        let mut params: Vec<String> = Vec::new();
        let mut param_count = 1;

        if let Some(name) = &updates.name {
            params.push(format!("name = ${}", param_count));
            param_count += 1;
        }

        if let Some(config) = &updates.config {
            params.push(format!("config = ${}", param_count));
            param_count += 1;
        }

        if let Some(is_enabled) = &updates.is_enabled {
            params.push(format!("is_enabled = ${}", param_count));
            param_count += 1;
        }

        // Always update updated_at
        params.push("updated_at = NOW()".to_string());

        query.push_str(&params.join(", "));
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        // Execute update based on which fields are provided
        let connection = if let (Some(name), Some(config), Some(is_enabled)) =
            (&updates.name, &updates.config, &updates.is_enabled)
        {
            sqlx::query_as!(
                Connection,
                r#"
                UPDATE connections
                SET name = $1, config = $2, is_enabled = $3, updated_at = NOW()
                WHERE id = $4
                RETURNING
                    id,
                    name,
                    type as connection_type,
                    scope as "scope: ConnectionScope",
                    tenant_id as organization_id,
                    config,
                    is_enabled,
                    created_at,
                    updated_at
                "#,
                name,
                config,
                is_enabled,
                id
            )
            .fetch_one(&self.pool)
            .await?
        } else if let (Some(name), Some(config)) = (&updates.name, &updates.config) {
            sqlx::query_as!(
                Connection,
                r#"
                UPDATE connections
                SET name = $1, config = $2, updated_at = NOW()
                WHERE id = $3
                RETURNING
                    id,
                    name,
                    type as connection_type,
                    scope as "scope: ConnectionScope",
                    tenant_id as organization_id,
                    config,
                    is_enabled,
                    created_at,
                    updated_at
                "#,
                name,
                config,
                id
            )
            .fetch_one(&self.pool)
            .await?
        } else if let (Some(name), Some(is_enabled)) = (&updates.name, &updates.is_enabled) {
            sqlx::query_as!(
                Connection,
                r#"
                UPDATE connections
                SET name = $1, is_enabled = $2, updated_at = NOW()
                WHERE id = $3
                RETURNING
                    id,
                    name,
                    type as connection_type,
                    scope as "scope: ConnectionScope",
                    tenant_id as organization_id,
                    config,
                    is_enabled,
                    created_at,
                    updated_at
                "#,
                name,
                is_enabled,
                id
            )
            .fetch_one(&self.pool)
            .await?
        } else if let Some(name) = &updates.name {
            sqlx::query_as!(
                Connection,
                r#"
                UPDATE connections
                SET name = $1, updated_at = NOW()
                WHERE id = $2
                RETURNING
                    id,
                    name,
                    type as connection_type,
                    scope as "scope: ConnectionScope",
                    tenant_id as organization_id,
                    config,
                    is_enabled,
                    created_at,
                    updated_at
                "#,
                name,
                id
            )
            .fetch_one(&self.pool)
            .await?
        } else if let Some(config) = &updates.config {
            sqlx::query_as!(
                Connection,
                r#"
                UPDATE connections
                SET config = $1, updated_at = NOW()
                WHERE id = $2
                RETURNING
                    id,
                    name,
                    type as connection_type,
                    scope as "scope: ConnectionScope",
                    tenant_id as organization_id,
                    config,
                    is_enabled,
                    created_at,
                    updated_at
                "#,
                config,
                id
            )
            .fetch_one(&self.pool)
            .await?
        } else if let Some(is_enabled) = &updates.is_enabled {
            sqlx::query_as!(
                Connection,
                r#"
                UPDATE connections
                SET is_enabled = $1, updated_at = NOW()
                WHERE id = $2
                RETURNING
                    id,
                    name,
                    type as connection_type,
                    scope as "scope: ConnectionScope",
                    tenant_id as organization_id,
                    config,
                    is_enabled,
                    created_at,
                    updated_at
                "#,
                is_enabled,
                id
            )
            .fetch_one(&self.pool)
            .await?
        } else {
            // No updates provided, just return current
            return self
                .get_by_id(id)
                .await?
                .ok_or_else(|| sqlx::Error::RowNotFound);
        };

        Ok(connection)
    }

    /// Delete connection
    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let result: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            DELETE FROM connections
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List all connections (for admin)
    pub async fn list_all(&self) -> Result<Vec<Connection>, sqlx::Error> {
        let connections = sqlx::query_as!(
            Connection,
            r#"
            SELECT
                id,
                name,
                type as connection_type,
                scope as "scope: ConnectionScope",
                tenant_id as organization_id,
                config,
                is_enabled,
                created_at,
                updated_at
            FROM connections
            ORDER BY scope ASC, created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(connections)
    }

    /// Count connections by scope
    pub async fn count_by_scope(&self, scope: ConnectionScope) -> Result<i64, sqlx::Error> {
        let scope_str = match scope {
            ConnectionScope::Platform => "platform",
            ConnectionScope::Organization => "organization",
        };

        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM connections
            WHERE scope = $1
            "#,
            scope_str
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count)
    }
}
