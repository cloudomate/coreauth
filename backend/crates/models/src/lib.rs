// Core modules
pub mod organization;
pub mod tenant;  // Kept for backward compatibility (re-exports organization types)
pub mod user;
pub mod role;
pub mod permission;
pub mod session;
pub mod oauth;
pub mod oidc_provider;
pub mod mfa;
pub mod audit;
pub mod organization_member;
pub mod connection;

// New modules for multi-tenant hierarchy
pub mod application;
pub mod action;

// Re-export commonly used types
pub use organization::{
    Organization, CreateOrganization, UpdateOrganization,
    OrganizationSettings, IsolationMode, BrandingSettings,
    SecuritySettings, FeatureFlags,
};
pub use tenant::{Tenant, NewTenant, TenantSettings};  // Backward compatibility aliases
pub use user::{User, NewUser, UserMetadata};
pub use role::{Role, NewRole};
pub use permission::{Permission, NewPermission};
pub use session::{Session, NewSession};
pub use oauth::{OAuthClient, NewOAuthClient};
pub use oidc_provider::{OidcProvider, NewOidcProvider, ClaimMappings, OAuthState, IdTokenClaims};
pub use mfa::{MfaMethod, MfaMethodType};
pub use audit::{AuditLog, CreateAuditLog, AuditEventCategory, AuditStatus, AuditLogBuilder, AuditLogQuery};
pub use organization_member::{OrganizationMember, AddOrganizationMember, UpdateMemberRole, OrganizationMemberWithUser};
pub use connection::{Connection, CreateConnection, UpdateConnection, ConnectionScope, OidcConnectionConfig, SamlConnectionConfig, DatabaseConnectionConfig};
pub use application::{
    Application, ApplicationWithSecret, CreateApplication, UpdateApplication,
    ApplicationType,
};
pub use action::{
    Action, CreateAction, UpdateAction, ActionTrigger,
    ActionExecution, ExecutionStatus, ActionContext, ActionResult,
};
