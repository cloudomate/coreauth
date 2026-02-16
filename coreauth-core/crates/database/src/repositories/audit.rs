use ciam_models::{AuditLog, CreateAuditLog, AuditLogQuery};
use ipnetwork::IpNetwork;
use sqlx::{PgPool, QueryBuilder, Postgres};
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new audit log entry (immutable, append-only)
    pub async fn create(&self, log: CreateAuditLog) -> Result<AuditLog, sqlx::Error> {
        // Convert IpAddr to IpNetwork for database storage
        let actor_ip_network = log.actor_ip_address.map(|ip| {
            match ip {
                std::net::IpAddr::V4(v4) => IpNetwork::new(v4.into(), 32).expect("Valid IPv4"),
                std::net::IpAddr::V6(v6) => IpNetwork::new(v6.into(), 128).expect("Valid IPv6"),
            }
        });

        let audit_log = sqlx::query_as!(
            AuditLog,
            r#"
            INSERT INTO audit_logs (
                tenant_id, event_type, event_category, event_action,
                actor_type, actor_id, actor_name, actor_ip_address, actor_user_agent,
                target_type, target_id, target_name,
                description, metadata,
                status, error_message,
                request_id, session_id
            )
            VALUES ($1, $2, $3::audit_event_category, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            RETURNING
                id, tenant_id,
                event_type, event_category as "event_category: _", event_action,
                actor_type, actor_id, actor_name, actor_ip_address::text as actor_ip_address, actor_user_agent,
                target_type, target_id, target_name,
                description, metadata,
                status, error_message,
                request_id, session_id,
                created_at
            "#,
            log.tenant_id,
            log.event_type,
            log.event_category as _,
            log.event_action,
            log.actor_type,
            log.actor_id,
            log.actor_name,
            actor_ip_network,
            log.actor_user_agent,
            log.target_type,
            log.target_id,
            log.target_name,
            log.description,
            log.metadata,
            log.status.to_string(),
            log.error_message,
            log.request_id,
            log.session_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(audit_log)
    }

    /// Query audit logs with filters
    pub async fn query(&self, query: AuditLogQuery) -> Result<Vec<AuditLog>, sqlx::Error> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                id, tenant_id,
                event_type, event_category, event_action,
                actor_type, actor_id, actor_name, actor_ip_address::text as actor_ip_address, actor_user_agent,
                target_type, target_id, target_name,
                description, metadata,
                status, error_message,
                request_id, session_id,
                created_at
            FROM audit_logs
            WHERE tenant_id =
            "#
        );

        builder.push_bind(query.tenant_id);

        // Add event type filter
        if let Some(event_types) = &query.event_types {
            if !event_types.is_empty() {
                builder.push(" AND event_type = ANY(");
                builder.push_bind(event_types);
                builder.push(")");
            }
        }

        // Add event category filter
        if let Some(categories) = &query.event_categories {
            if !categories.is_empty() {
                builder.push(" AND event_category = ANY(");
                let category_strs: Vec<String> = categories.iter()
                    .map(|c| format!("{:?}", c).to_lowercase())
                    .collect();
                builder.push_bind(category_strs);
                builder.push("::audit_event_category[])");
            }
        }

        // Add actor filter
        if let Some(actor_id) = &query.actor_id {
            builder.push(" AND actor_id = ");
            builder.push_bind(actor_id);
        }

        // Add target filter
        if let Some(target_id) = &query.target_id {
            builder.push(" AND target_id = ");
            builder.push_bind(target_id);
        }

        // Add status filter
        if let Some(status) = &query.status {
            builder.push(" AND status = ");
            builder.push_bind(status.to_string());
        }

        // Add date range filters
        if let Some(from_date) = query.from_date {
            builder.push(" AND created_at >= ");
            builder.push_bind(from_date);
        }

        if let Some(to_date) = query.to_date {
            builder.push(" AND created_at <= ");
            builder.push_bind(to_date);
        }

        // Order by created_at descending (most recent first)
        builder.push(" ORDER BY created_at DESC");

        // Add limit and offset
        if let Some(limit) = query.limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit);
        }

        if let Some(offset) = query.offset {
            builder.push(" OFFSET ");
            builder.push_bind(offset);
        }

        let query = builder.build_query_as::<AuditLog>();
        let logs = query.fetch_all(&self.pool).await?;

        Ok(logs)
    }

    /// Get a single audit log by ID
    pub async fn get_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<AuditLog>, sqlx::Error> {
        let log = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT
                id, tenant_id,
                event_type, event_category as "event_category: _", event_action,
                actor_type, actor_id, actor_name, actor_ip_address::text as actor_ip_address, actor_user_agent,
                target_type, target_id, target_name,
                description, metadata,
                status, error_message,
                request_id, session_id,
                created_at
            FROM audit_logs
            WHERE tenant_id = $1 AND id = $2
            "#,
            tenant_id,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(log)
    }

    /// Get audit logs count for a tenant
    pub async fn count(&self, tenant_id: Uuid) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM audit_logs WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0))
    }

    /// Get recent security events for a tenant
    pub async fn get_security_events(
        &self,
        tenant_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT
                id, tenant_id,
                event_type, event_category as "event_category: _", event_action,
                actor_type, actor_id, actor_name, actor_ip_address::text as actor_ip_address, actor_user_agent,
                target_type, target_id, target_name,
                description, metadata,
                status, error_message,
                request_id, session_id,
                created_at
            FROM audit_logs
            WHERE tenant_id = $1 AND event_category = 'security'
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            tenant_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Get failed login attempts for a user
    pub async fn get_failed_logins(
        &self,
        tenant_id: Uuid,
        actor_id: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT
                id, tenant_id,
                event_type, event_category as "event_category: _", event_action,
                actor_type, actor_id, actor_name, actor_ip_address::text as actor_ip_address, actor_user_agent,
                target_type, target_id, target_name,
                description, metadata,
                status, error_message,
                request_id, session_id,
                created_at
            FROM audit_logs
            WHERE tenant_id = $1
                AND actor_id = $2
                AND event_type = 'user.login.failed'
                AND created_at >= $3
            ORDER BY created_at DESC
            "#,
            tenant_id,
            actor_id,
            since
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Export audit logs to JSON (for archival/compliance)
    pub async fn export(
        &self,
        tenant_id: Uuid,
        from_date: chrono::DateTime<chrono::Utc>,
        to_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT
                id, tenant_id,
                event_type, event_category as "event_category: _", event_action,
                actor_type, actor_id, actor_name, actor_ip_address::text as actor_ip_address, actor_user_agent,
                target_type, target_id, target_name,
                description, metadata,
                status, error_message,
                request_id, session_id,
                created_at
            FROM audit_logs
            WHERE tenant_id = $1
                AND created_at >= $2
                AND created_at <= $3
            ORDER BY created_at ASC
            "#,
            tenant_id,
            from_date,
            to_date
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }
}
