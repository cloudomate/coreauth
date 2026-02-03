// Tenant context for request handling

use ciam_models::Tenant;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tenant: Option<Tenant>,
}

impl TenantContext {
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            tenant_id,
            tenant: None,
        }
    }

    pub fn with_tenant(tenant_id: Uuid, tenant: Tenant) -> Self {
        Self {
            tenant_id,
            tenant: Some(tenant),
        }
    }
}
