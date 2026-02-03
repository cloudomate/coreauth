use crate::error::{AuthzError, Result};
use crate::tuple::{QueryTuplesRequest, SubjectType, TupleService};
use ciam_cache::Cache;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Permission check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRequest {
    pub tenant_id: Uuid,
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub relation: String,
    pub namespace: String,
    pub object_id: String,
    #[serde(default)]
    pub context: HashMap<String, serde_json::Value>,
}

/// Permission check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResponse {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Expand response showing all subjects with a relation to an object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandResponse {
    pub subjects: Vec<SubjectInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectInfo {
    pub subject_type: SubjectType,
    pub subject_id: String,
    pub via_relation: Option<String>,
}

pub struct PolicyEngine {
    tuple_service: TupleService,
    cache: Cache,
}

impl PolicyEngine {
    pub fn new(tuple_service: TupleService, cache: Cache) -> Self {
        Self {
            tuple_service,
            cache,
        }
    }

    /// Check if a subject has a relation to an object
    /// Implements Zanzibar-style permission checking with graph traversal
    pub async fn check(&self, request: CheckRequest) -> Result<CheckResponse> {
        // Generate cache key
        let cache_key = format!(
            "authz:check:{}:{}:{}:{}:{}:{}",
            request.tenant_id,
            format!("{:?}", request.subject_type).to_lowercase(),
            request.subject_id,
            request.relation,
            request.namespace,
            request.object_id
        );

        // Check cache first
        if let Ok(Some(cached)) = self.cache.get::<bool>(&cache_key).await {
            return Ok(CheckResponse {
                allowed: cached,
                reason: Some("From cache".to_string()),
            });
        }

        // Perform the check
        let allowed = self
            .check_recursive(
                request.tenant_id,
                request.subject_type.clone(),
                &request.subject_id,
                &request.relation,
                &request.namespace,
                &request.object_id,
                &mut HashSet::new(),
            )
            .await?;

        // Cache the result for 60 seconds
        let _ = self.cache.set(&cache_key, &allowed, Some(60)).await;

        Ok(CheckResponse {
            allowed,
            reason: if allowed {
                Some("Permission granted".to_string())
            } else {
                Some("Permission denied".to_string())
            },
        })
    }

    /// Recursive permission check with cycle detection
    fn check_recursive<'a>(
        &'a self,
        tenant_id: Uuid,
        subject_type: SubjectType,
        subject_id: &'a str,
        relation: &'a str,
        namespace: &'a str,
        object_id: &'a str,
        visited: &'a mut HashSet<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + 'a + Send>> {
        Box::pin(async move {
        // Create a visit key to detect cycles
        let visit_key = format!(
            "{}:{}:{}:{}:{}",
            format!("{:?}", subject_type).to_lowercase(),
            subject_id,
            relation,
            namespace,
            object_id
        );

        if visited.contains(&visit_key) {
            return Ok(false); // Cycle detected
        }
        visited.insert(visit_key.clone());

        // Direct check: Does the subject directly have the relation?
        if self
            .tuple_service
            .tuple_exists(
                tenant_id,
                namespace,
                object_id,
                relation,
                subject_type.clone(),
                subject_id,
            )
            .await?
        {
            return Ok(true);
        }

        // Userset check: Check if subject is part of a userset that has the relation
        // For example: user:alice#member@group:admins, and group:admins#viewer@document:doc1
        if subject_type == SubjectType::User {
            // Get all groups the user is a member of
            let user_groups = self
                .tuple_service
                .get_subject_tuples(tenant_id, SubjectType::User, subject_id)
                .await?;

            for group_tuple in user_groups {
                // Check if any of these groups have the required relation to the object
                if self
                    .check_recursive(
                        tenant_id,
                        SubjectType::Group,
                        &group_tuple.object_id,
                        relation,
                        namespace,
                        object_id,
                        visited,
                    )
                    .await?
                {
                    return Ok(true);
                }
            }
        }

        // Hierarchical check: Check for parent relations
        // For example: if checking "view" permission, also check for "edit" or "owner"
        // This requires relation hierarchy configuration (not implemented in basic version)

        // Computed usersets: Check for indirect relations
        // For example: document:doc1#viewer@folder:folder1#member
        // This means "all members of folder1 can view doc1"
        let indirect_tuples = self
            .tuple_service
            .query_tuples(QueryTuplesRequest {
                tenant_id,
                namespace: Some(namespace.to_string()),
                object_id: Some(object_id.to_string()),
                relation: Some(relation.to_string()),
                subject_type: Some(SubjectType::UserSet),
                subject_id: None,
            })
            .await?;

        for indirect in indirect_tuples {
            // Parse the userset reference
            // Format: namespace:object_id#relation
            if let Some(sub_rel) = &indirect.subject_relation {
                // Check if our subject has the sub_relation to the subject_id
                if self
                    .check_recursive(
                        tenant_id,
                        subject_type.clone(),
                        subject_id,
                        sub_rel,
                        namespace,
                        &indirect.subject_id,
                        visited,
                    )
                    .await?
                {
                    return Ok(true);
                }
            }
        }

        Ok(false)
        })
    }

    /// Expand a relation to show all subjects that have it
    pub async fn expand(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        object_id: &str,
        relation: &str,
    ) -> Result<ExpandResponse> {
        let tuples = self
            .tuple_service
            .query_tuples(QueryTuplesRequest {
                tenant_id,
                namespace: Some(namespace.to_string()),
                object_id: Some(object_id.to_string()),
                relation: Some(relation.to_string()),
                subject_type: None,
                subject_id: None,
            })
            .await?;

        let subjects = tuples
            .into_iter()
            .map(|t| SubjectInfo {
                subject_type: t.subject_type,
                subject_id: t.subject_id,
                via_relation: t.subject_relation,
            })
            .collect();

        Ok(ExpandResponse { subjects })
    }

    /// Invalidate cache for a specific tuple
    pub async fn invalidate_cache(
        &self,
        tenant_id: Uuid,
        namespace: &str,
        object_id: &str,
    ) -> Result<()> {
        // In a production system, you would invalidate all affected cache entries
        // For now, we'll just invalidate the specific object pattern
        let pattern = format!("authz:check:{}:*:{}:{}", tenant_id, namespace, object_id);

        tracing::debug!("Invalidating cache pattern: {}", pattern);

        // Note: This requires Redis SCAN and DEL operations
        // For now, we'll just log it. In production, implement proper cache invalidation

        Ok(())
    }
}
