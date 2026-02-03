// Backward compatibility module - re-exports organization types as tenant types
pub use crate::organization::{
    Organization as Tenant,
    CreateOrganization as NewTenant,
    UpdateOrganization as UpdateTenant,
    OrganizationSettings as TenantSettings,
    IsolationMode,
    BrandingSettings,
    SecuritySettings,
    FeatureFlags,
};
