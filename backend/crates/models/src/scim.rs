use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// SCIM 2.0 Protocol Types
// ============================================================================

/// SCIM User Resource (RFC 7643)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimUser {
    /// SCIM schemas
    pub schemas: Vec<String>,

    /// Unique identifier (our user ID)
    pub id: String,

    /// External identifier from IdP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,

    /// Unique username (typically email)
    pub user_name: String,

    /// User's name components
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ScimName>,

    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Email addresses
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<ScimEmail>,

    /// Phone numbers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phone_numbers: Vec<ScimPhoneNumber>,

    /// User active status
    pub active: bool,

    /// Groups the user belongs to (read-only)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<ScimGroupRef>,

    /// Resource metadata
    pub meta: ScimMeta,
}

impl ScimUser {
    pub fn schemas() -> Vec<String> {
        vec!["urn:ietf:params:scim:schemas:core:2.0:User".to_string()]
    }
}

/// SCIM Name component
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScimName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub honorific_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub honorific_suffix: Option<String>,
}

/// SCIM Email
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimEmail {
    pub value: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub email_type: Option<String>,
    #[serde(default)]
    pub primary: bool,
}

/// SCIM Phone Number
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPhoneNumber {
    pub value: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub phone_type: Option<String>,
    #[serde(default)]
    pub primary: bool,
}

/// SCIM Group Reference (for user's groups)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroupRef {
    pub value: String,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// SCIM Group Resource
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimGroup {
    pub schemas: Vec<String>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<ScimMember>,
    pub meta: ScimMeta,
}

impl ScimGroup {
    pub fn schemas() -> Vec<String> {
        vec!["urn:ietf:params:scim:schemas:core:2.0:Group".to_string()]
    }
}

/// SCIM Group Member
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMember {
    pub value: String,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// SCIM Resource Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimMeta {
    pub resource_type: String,
    pub created: String,
    pub last_modified: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl ScimMeta {
    pub fn new(resource_type: &str, created: DateTime<Utc>, updated: DateTime<Utc>, base_url: &str, id: &str) -> Self {
        Self {
            resource_type: resource_type.to_string(),
            created: created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            last_modified: updated.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            location: Some(format!("{}/scim/v2/{}s/{}", base_url, resource_type, id)),
            version: Some(format!("W/\"{}\"", updated.timestamp())),
        }
    }
}

/// SCIM List Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimListResponse<T> {
    pub schemas: Vec<String>,
    pub total_results: i64,
    pub items_per_page: i64,
    pub start_index: i64,
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
}

impl<T> ScimListResponse<T> {
    pub fn new(resources: Vec<T>, total: i64, start: i64, count: i64) -> Self {
        Self {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:ListResponse".to_string()],
            total_results: total,
            items_per_page: count,
            start_index: start,
            resources,
        }
    }
}

/// SCIM Error Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimError {
    pub schemas: Vec<String>,
    pub detail: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scim_type: Option<String>,
}

impl ScimError {
    pub fn new(status: u16, detail: impl Into<String>) -> Self {
        Self {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            detail: detail.into(),
            status,
            scim_type: None,
        }
    }

    pub fn with_type(mut self, scim_type: &str) -> Self {
        self.scim_type = Some(scim_type.to_string());
        self
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(404, detail)
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        Self::new(409, detail).with_type("uniqueness")
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(400, detail).with_type("invalidValue")
    }

    pub fn unauthorized() -> Self {
        Self::new(401, "Unauthorized")
    }

    pub fn forbidden() -> Self {
        Self::new(403, "Forbidden")
    }

    pub fn internal_error(detail: impl Into<String>) -> Self {
        Self::new(500, detail)
    }
}

/// SCIM Patch Operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimPatchRequest {
    pub schemas: Vec<String>,
    #[serde(rename = "Operations")]
    pub operations: Vec<ScimPatchOp>,
}

/// Single SCIM Patch Operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPatchOp {
    pub op: String,  // "add", "remove", "replace"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// SCIM Service Provider Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceProviderConfig {
    pub schemas: Vec<String>,
    pub documentation_uri: Option<String>,
    pub patch: SupportedFeature,
    pub bulk: BulkFeature,
    pub filter: FilterFeature,
    pub change_password: SupportedFeature,
    pub sort: SupportedFeature,
    pub etag: SupportedFeature,
    pub authentication_schemes: Vec<AuthenticationScheme>,
    pub meta: ScimMeta,
}

impl Default for ServiceProviderConfig {
    fn default() -> Self {
        Self {
            schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:ServiceProviderConfig".to_string()],
            documentation_uri: Some("https://docs.coreauth.io/scim".to_string()),
            patch: SupportedFeature { supported: true },
            bulk: BulkFeature {
                supported: false,
                max_operations: 0,
                max_payload_size: 0,
            },
            filter: FilterFeature {
                supported: true,
                max_results: 100,
            },
            change_password: SupportedFeature { supported: false },
            sort: SupportedFeature { supported: true },
            etag: SupportedFeature { supported: true },
            authentication_schemes: vec![AuthenticationScheme {
                name: "OAuth Bearer Token".to_string(),
                description: "Authentication scheme using the OAuth Bearer Token Standard".to_string(),
                spec_uri: Some("https://www.rfc-editor.org/info/rfc6750".to_string()),
                authentication_type: "oauthbearertoken".to_string(),
                primary: true,
            }],
            meta: ScimMeta {
                resource_type: "ServiceProviderConfig".to_string(),
                created: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                last_modified: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                location: None,
                version: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedFeature {
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkFeature {
    pub supported: bool,
    pub max_operations: i32,
    pub max_payload_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterFeature {
    pub supported: bool,
    pub max_results: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationScheme {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_uri: Option<String>,
    #[serde(rename = "type")]
    pub authentication_type: String,
    pub primary: bool,
}

/// SCIM Resource Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceType {
    pub schemas: Vec<String>,
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub description: String,
    pub schema: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_extensions: Vec<SchemaExtension>,
    pub meta: ScimMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaExtension {
    pub schema: String,
    pub required: bool,
}

// ============================================================================
// Database Models
// ============================================================================

/// SCIM Token (database model)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimToken {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub last_used_ip: Option<String>,
    pub request_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

/// Request to create a SCIM token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScimToken {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response when creating a SCIM token (includes the actual token once)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimTokenResponse {
    pub id: Uuid,
    pub name: String,
    pub token_prefix: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Response when creating a token (includes full token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimTokenWithSecret {
    #[serde(flatten)]
    pub info: ScimTokenResponse,
    pub secret: String,  // The actual bearer token (only shown once)
}

/// SCIM Group (database model)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimGroupRecord {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub display_name: String,
    pub external_id: Option<String>,
    pub role_id: Option<Uuid>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SCIM Group Member (database model)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimGroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// SCIM Configuration (database model)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimConfiguration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub is_enabled: bool,
    pub auto_create_users: bool,
    pub auto_update_users: bool,
    pub auto_deactivate_users: bool,
    pub sync_groups: bool,
    pub attribute_mapping: sqlx::types::Json<serde_json::Value>,
    pub default_role: String,
    pub total_users_provisioned: i32,
    pub total_groups_provisioned: i32,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SCIM Provisioning Log (database model)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimProvisioningLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub token_id: Option<Uuid>,
    pub operation: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub request_path: String,
    pub request_method: String,
    pub request_body: Option<sqlx::types::Json<serde_json::Value>>,
    pub response_status: i32,
    pub response_body: Option<sqlx::types::Json<serde_json::Value>>,
    pub error_message: Option<String>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub duration_ms: Option<i32>,
}

// ============================================================================
// Query Parameters
// ============================================================================

/// SCIM List Query Parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimListQuery {
    /// Filter expression (e.g., userName eq "john@example.com")
    pub filter: Option<String>,
    /// Attribute to sort by
    #[serde(rename = "sortBy")]
    pub sort_by: Option<String>,
    /// Sort order (ascending or descending)
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<String>,
    /// 1-based starting index
    #[serde(rename = "startIndex", default = "default_start_index")]
    pub start_index: i64,
    /// Number of results to return
    #[serde(default = "default_count")]
    pub count: i64,
}

fn default_start_index() -> i64 { 1 }
fn default_count() -> i64 { 100 }

impl Default for ScimListQuery {
    fn default() -> Self {
        Self {
            filter: None,
            sort_by: None,
            sort_order: None,
            start_index: 1,
            count: 100,
        }
    }
}

/// Parsed SCIM filter
#[derive(Debug, Clone)]
pub struct ScimFilter {
    pub attribute: String,
    pub operator: ScimFilterOp,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScimFilterOp {
    Eq,  // equals
    Ne,  // not equals
    Co,  // contains
    Sw,  // starts with
    Ew,  // ends with
    Pr,  // present (has value)
    Gt,  // greater than
    Ge,  // greater or equal
    Lt,  // less than
    Le,  // less or equal
}

impl ScimFilter {
    /// Parse a simple SCIM filter expression
    pub fn parse(filter: &str) -> Option<Self> {
        let parts: Vec<&str> = filter.splitn(3, ' ').collect();
        if parts.len() < 2 {
            return None;
        }

        let attribute = parts[0].to_string();
        let operator = match parts[1].to_lowercase().as_str() {
            "eq" => ScimFilterOp::Eq,
            "ne" => ScimFilterOp::Ne,
            "co" => ScimFilterOp::Co,
            "sw" => ScimFilterOp::Sw,
            "ew" => ScimFilterOp::Ew,
            "pr" => ScimFilterOp::Pr,
            "gt" => ScimFilterOp::Gt,
            "ge" => ScimFilterOp::Ge,
            "lt" => ScimFilterOp::Lt,
            "le" => ScimFilterOp::Le,
            _ => return None,
        };

        let value = if operator == ScimFilterOp::Pr {
            String::new()
        } else if parts.len() >= 3 {
            // Remove surrounding quotes if present
            let v = parts[2];
            if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
                v[1..v.len()-1].to_string()
            } else {
                v.to_string()
            }
        } else {
            return None;
        };

        Some(Self {
            attribute,
            operator,
            value,
        })
    }
}

// ============================================================================
// Create/Update Requests
// ============================================================================

/// Request to create a SCIM user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScimUser {
    pub schemas: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub user_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ScimName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub emails: Vec<ScimEmail>,
    #[serde(default)]
    pub phone_numbers: Vec<ScimPhoneNumber>,
    #[serde(default = "default_active")]
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

fn default_active() -> bool { true }

/// Request to create a SCIM group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScimGroup {
    pub schemas: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub display_name: String,
    #[serde(default)]
    pub members: Vec<ScimMember>,
}
