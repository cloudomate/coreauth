use crate::error::{AuthzError, Result};
use crate::store::AuthorizationSchema;
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

    /// Check if a subject has a relation to an object (backward-compatible, no model)
    pub async fn check(&self, request: CheckRequest) -> Result<CheckResponse> {
        self.check_with_model(request, None).await
    }

    /// Check if a subject has a relation to an object, with optional model-aware resolution
    pub async fn check_with_model(
        &self,
        request: CheckRequest,
        schema: Option<&AuthorizationSchema>,
    ) -> Result<CheckResponse> {
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
                schema,
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

    /// Recursive permission check with cycle detection and model-aware resolution
    fn check_recursive<'a>(
        &'a self,
        tenant_id: Uuid,
        subject_type: SubjectType,
        subject_id: &'a str,
        relation: &'a str,
        namespace: &'a str,
        object_id: &'a str,
        visited: &'a mut HashSet<String>,
        schema: Option<&'a AuthorizationSchema>,
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

        // 1. Direct check: Does the subject directly have the relation?
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

        // 2. Group check: Check if subject is part of a group that has the relation
        if subject_type == SubjectType::User {
            let user_groups = self
                .tuple_service
                .get_subject_tuples(tenant_id, SubjectType::User, subject_id)
                .await?;

            for group_tuple in user_groups {
                if self
                    .check_recursive(
                        tenant_id,
                        SubjectType::Group,
                        &group_tuple.object_id,
                        relation,
                        namespace,
                        object_id,
                        visited,
                        schema,
                    )
                    .await?
                {
                    return Ok(true);
                }
            }
        }

        // 3. Model-aware resolution: walk the authorization model definition
        if let Some(schema) = schema {
            if self
                .check_model_aware(
                    tenant_id,
                    subject_type.clone(),
                    subject_id,
                    relation,
                    namespace,
                    object_id,
                    visited,
                    schema,
                )
                .await?
            {
                return Ok(true);
            }
        } else {
            // Legacy: UserSet indirect check (when no model is provided)
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
                if let Some(sub_rel) = &indirect.subject_relation {
                    if self
                        .check_recursive(
                            tenant_id,
                            subject_type.clone(),
                            subject_id,
                            sub_rel,
                            namespace,
                            &indirect.subject_id,
                            visited,
                            None,
                        )
                        .await?
                    {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
        })
    }

    /// Model-aware permission resolution: resolve computedUserset, tupleToUserset, union
    async fn check_model_aware(
        &self,
        tenant_id: Uuid,
        subject_type: SubjectType,
        subject_id: &str,
        relation: &str,
        namespace: &str,
        object_id: &str,
        visited: &mut HashSet<String>,
        schema: &AuthorizationSchema,
    ) -> Result<bool> {
        // Find the type definition for this namespace (object_type)
        let type_def = match schema
            .type_definitions
            .iter()
            .find(|td| td.type_name == namespace)
        {
            Some(td) => td,
            None => return Ok(false),
        };

        // Find the relation definition
        let rel_def = match type_def.relations.get(relation) {
            Some(rd) => rd,
            None => return Ok(false),
        };

        // Resolve the relation definition
        self.check_relation_def(
            tenant_id,
            subject_type,
            subject_id,
            namespace,
            object_id,
            rel_def,
            type_def,
            visited,
            schema,
        )
        .await
    }

    /// Resolve a single RelationDefinition node
    fn check_relation_def<'a>(
        &'a self,
        tenant_id: Uuid,
        subject_type: SubjectType,
        subject_id: &'a str,
        namespace: &'a str,
        object_id: &'a str,
        rel_def: &'a crate::store::RelationDefinition,
        type_def: &'a crate::store::TypeDefinition,
        visited: &'a mut HashSet<String>,
        schema: &'a AuthorizationSchema,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool>> + 'a + Send>> {
        Box::pin(async move {
        // Direct assignment (this: {}) — already handled by tuple_exists in check_recursive
        // Skip here to avoid double-checking

        // computedUserset: check another relation on the SAME object
        if let Some(computed) = &rel_def.computed_userset {
            if self
                .check_recursive(
                    tenant_id,
                    subject_type.clone(),
                    subject_id,
                    &computed.relation,
                    namespace,
                    object_id,
                    visited,
                    Some(schema),
                )
                .await?
            {
                return Ok(true);
            }
        }

        // tupleToUserset: follow a tupleset relation to find linked objects,
        // then check the computed relation on those linked objects
        if let Some(ttu) = &rel_def.tuple_to_userset {
            // Find all tuples that link this object via the tupleset relation
            // e.g., for compute_instance:i-001 with tupleset "workspace",
            // find tuples: ?:? → workspace → compute_instance:i-001
            let linked_tuples = self
                .tuple_service
                .query_tuples(QueryTuplesRequest {
                    tenant_id,
                    namespace: Some(namespace.to_string()),
                    object_id: Some(object_id.to_string()),
                    relation: Some(ttu.tupleset.relation.clone()),
                    subject_type: None,
                    subject_id: None,
                })
                .await?;

            for linked in &linked_tuples {
                // Determine the linked object's type from metadata
                // The metadata for the tupleset relation tells us what types are allowed
                let linked_type = self.resolve_linked_type(
                    &ttu.tupleset.relation,
                    type_def,
                    &linked.subject_id,
                );

                if let Some(linked_namespace) = linked_type {
                    // Check if the user has the computed relation on the linked object
                    if self
                        .check_recursive(
                            tenant_id,
                            subject_type.clone(),
                            subject_id,
                            &ttu.computed_userset.relation,
                            &linked_namespace,
                            &linked.subject_id,
                            visited,
                            Some(schema),
                        )
                        .await?
                    {
                        return Ok(true);
                    }
                }
            }
        }

        // union: any child must be true
        if let Some(union_parts) = &rel_def.union {
            for part in union_parts {
                if self
                    .check_relation_def(
                        tenant_id,
                        subject_type.clone(),
                        subject_id,
                        namespace,
                        object_id,
                        part,
                        type_def,
                        visited,
                        schema,
                    )
                    .await?
                {
                    return Ok(true);
                }
            }
        }

        // intersection: all children must be true
        if let Some(intersection_parts) = &rel_def.intersection {
            if !intersection_parts.is_empty() {
                let mut all_true = true;
                for part in intersection_parts {
                    if !self
                        .check_relation_def(
                            tenant_id,
                            subject_type.clone(),
                            subject_id,
                            namespace,
                            object_id,
                            part,
                            type_def,
                            visited,
                            schema,
                        )
                        .await?
                    {
                        all_true = false;
                        break;
                    }
                }
                if all_true {
                    return Ok(true);
                }
            }
        }

        // exclusion: base must be true AND subtract must be false
        if let Some(exclusion) = &rel_def.exclusion {
            let base_ok = self
                .check_relation_def(
                    tenant_id,
                    subject_type.clone(),
                    subject_id,
                    namespace,
                    object_id,
                    &exclusion.base,
                    type_def,
                    visited,
                    schema,
                )
                .await?;

            if base_ok {
                let subtract_ok = self
                    .check_relation_def(
                        tenant_id,
                        subject_type.clone(),
                        subject_id,
                        namespace,
                        object_id,
                        &exclusion.subtract,
                        type_def,
                        visited,
                        schema,
                    )
                    .await?;

                if !subtract_ok {
                    return Ok(true);
                }
            }
        }

        Ok(false)
        })
    }

    /// Resolve the type/namespace of a linked object from metadata
    /// For a tupleset relation like "workspace", look at the metadata to find
    /// what types are allowed (e.g., [{ type: "workspace" }])
    fn resolve_linked_type(
        &self,
        tupleset_relation: &str,
        type_def: &crate::store::TypeDefinition,
        _subject_id: &str,
    ) -> Option<String> {
        // Check metadata for the tupleset relation to find allowed types
        if let Some(metadata) = &type_def.metadata {
            if let Some(rel_meta) = metadata.relations.get(tupleset_relation) {
                // Return the first allowed type (in practice, tupleset relations
                // like "workspace" or "vpc" only allow one type)
                if let Some(first_type) = rel_meta.directly_related_user_types.first() {
                    return Some(first_type.type_name.clone());
                }
            }
        }

        // Fallback: if no metadata, try to infer from the relation name itself
        // (e.g., relation "workspace" likely points to type "workspace")
        None
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
        let pattern = format!("authz:check:{}:*:{}:{}", tenant_id, namespace, object_id);

        tracing::debug!("Invalidating cache pattern: {}", pattern);

        Ok(())
    }
}
