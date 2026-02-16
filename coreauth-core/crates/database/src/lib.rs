pub mod connection;
pub mod error;
pub mod repositories;
pub mod tenant_router;

pub use connection::{Database, DatabaseConfig};
pub use error::{DatabaseError, Result};
pub use repositories::{
    connection::ConnectionRepository,
    organization_member::OrganizationMemberRepository,
    roles::RoleRepository,
    sessions::SessionRepository,
    tenants::TenantRepository,
    users::UserRepository,
};
pub use tenant_router::{
    IsolationMode, TenantDatabaseRouter, TenantRecord, TenantRouterConfig, TenantRouterStats,
};
