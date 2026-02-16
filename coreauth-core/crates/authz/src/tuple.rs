use crate::error::{AuthzError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Subject types in relation tuples
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "subject_type", rename_all = "lowercase")]
pub enum SubjectType {
    User,        // Individual user
    Application, // Service principal
    Group,       // User group
    UserSet,     // Set of users (for hierarchical relations)
}

impl std::convert::TryFrom<String> for SubjectType {
    type Error = String;

    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "user" => Ok(SubjectType::User),
            "application" => Ok(SubjectType::Application),
            "group" => Ok(SubjectType::Group),
            "userset" => Ok(SubjectType::UserSet),
            _ => Err(format!("Invalid subject type: {}", s)),
        }
    }
}

/// Relation tuple representing a relationship between subject and object
/// Format: subject has relation to object
/// Example: user:alice#viewer@document:doc123
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RelationTuple {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub namespace: String,           // e.g., "document", "folder", "organization"
    pub object_id: String,           // e.g., "doc123"
    pub relation: String,            // e.g., "viewer", "editor", "owner"
    #[sqlx(try_from = "String")]
    pub subject_type: SubjectType,
    pub subject_id: String,          // e.g., "alice", "app-service-1"
    pub subject_relation: Option<String>, // For userset subjects, e.g., "member"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTupleRequest {
    pub tenant_id: Uuid,
    pub namespace: String,
    pub object_id: String,
    pub relation: String,
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub subject_relation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTuplesRequest {
    pub tenant_id: Uuid,
    pub namespace: Option<String>,
    pub object_id: Option<String>,
    pub relation: Option<String>,
    pub subject_type: Option<SubjectType>,
    pub subject_id: Option<String>,
}

#[derive(Clone)]
pub struct TupleService {
    pool: PgPool,
}

impl TupleService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new relation tuple
    pub async fn create_tuple(&self, request: CreateTupleRequest) -> Result<RelationTuple> {
        let tuple = sqlx::query_as::<_, RelationTuple>(
            r#"
            INSERT INTO relation_tuples
                (tenant_id, namespace, object_id, relation, subject_type, subject_id, subject_relation)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (tenant_id, namespace, object_id, relation, subject_type, subject_id, COALESCE(subject_relation, ''))
            DO NOTHING
            RETURNING *
            "#,
        )
        .bind(request.tenant_id)
        .bind(&request.namespace)
        .bind(&request.object_id)
        .bind(&request.relation)
        .bind(&request.subject_type)
        .bind(&request.subject_id)
        .bind(&request.subject_relation)
        .fetch_one(&self.pool)
        .await?;

        tracing::info!(
            "Created tuple: {}:{}#{}@{}:{}",
            format!("{:?}", tuple.subject_type).to_lowercase(),
            tuple.subject_id,
            tuple.subject_relation.as_deref().unwrap_or(""),
            tuple.namespace,
            tuple.object_id
        );

        Ok(tuple)
    }

    /// Delete a relation tuple
    pub async fn delete_tuple(&self, request: CreateTupleRequest) -> Result<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM relation_tuples
            WHERE tenant_id = $1
              AND namespace = $2
              AND object_id = $3
              AND relation = $4
              AND subject_type = $5
              AND subject_id = $6
              AND (subject_relation = $7 OR (subject_relation IS NULL AND $7 IS NULL))
            "#,
        )
        .bind(request.tenant_id)
        .bind(&request.namespace)
        .bind(&request.object_id)
        .bind(&request.relation)
        .bind(&request.subject_type)
        .bind(&request.subject_id)
        .bind(&request.subject_relation)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthzError::NotFound("Relation tuple not found".to_string()));
        }

        tracing::info!(
            "Deleted tuple: {}:{}@{}:{}",
            format!("{:?}", request.subject_type).to_lowercase(),
            request.subject_id,
            request.namespace,
            request.object_id
        );

        Ok(())
    }

    /// Query relation tuples
    pub async fn query_tuples(&self, request: QueryTuplesRequest) -> Result<Vec<RelationTuple>> {
        let mut query = String::from("SELECT * FROM relation_tuples WHERE tenant_id = $1");
        let mut bind_count = 2;

        if request.namespace.is_some() {
            query.push_str(&format!(" AND namespace = ${}", bind_count));
            bind_count += 1;
        }
        if request.object_id.is_some() {
            query.push_str(&format!(" AND object_id = ${}", bind_count));
            bind_count += 1;
        }
        if request.relation.is_some() {
            query.push_str(&format!(" AND relation = ${}", bind_count));
            bind_count += 1;
        }
        if request.subject_type.is_some() {
            query.push_str(&format!(" AND subject_type = ${}", bind_count));
            bind_count += 1;
        }
        if request.subject_id.is_some() {
            query.push_str(&format!(" AND subject_id = ${}", bind_count));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query_as::<_, RelationTuple>(&query).bind(request.tenant_id);

        if let Some(ns) = &request.namespace {
            q = q.bind(ns);
        }
        if let Some(oid) = &request.object_id {
            q = q.bind(oid);
        }
        if let Some(rel) = &request.relation {
            q = q.bind(rel);
        }
        if let Some(st) = &request.subject_type {
            q = q.bind(st);
        }
        if let Some(sid) = &request.subject_id {
            q = q.bind(sid);
        }

        Ok(q.fetch_all(&self.pool).await?)
    }

    /// Get all tuples for a specific object
    pub async fn get_object_tuples(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        object_id: &str,
    ) -> Result<Vec<RelationTuple>> {
        Ok(sqlx::query_as::<_, RelationTuple>(
            r#"
            SELECT * FROM relation_tuples
            WHERE tenant_id = $1 AND namespace = $2 AND object_id = $3
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id)
        .bind(namespace)
        .bind(object_id)
        .fetch_all(&self.pool)
        .await?)
    }

    /// Get all tuples for a specific subject
    pub async fn get_subject_tuples(
        &self,
        tenant_id: Uuid,
        subject_type: SubjectType,
        subject_id: &str,
    ) -> Result<Vec<RelationTuple>> {
        Ok(sqlx::query_as::<_, RelationTuple>(
            r#"
            SELECT * FROM relation_tuples
            WHERE tenant_id = $1 AND subject_type = $2 AND subject_id = $3
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id)
        .bind(subject_type)
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await?)
    }

    /// Check if a specific tuple exists
    pub async fn tuple_exists(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        object_id: &str,
        relation: &str,
        subject_type: SubjectType,
        subject_id: &str,
    ) -> Result<bool> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM relation_tuples
                WHERE tenant_id = $1
                  AND namespace = $2
                  AND object_id = $3
                  AND relation = $4
                  AND subject_type = $5
                  AND subject_id = $6
            )
            "#,
        )
        .bind(tenant_id)
        .bind(namespace)
        .bind(object_id)
        .bind(relation)
        .bind(subject_type)
        .bind(subject_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
