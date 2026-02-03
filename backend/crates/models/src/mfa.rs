use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MfaMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub method_type: MfaMethodType,
    pub secret: Option<String>,
    pub verified: bool,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum MfaMethodType {
    Totp,
    Sms,
    Email,
    Webauthn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMfaMethod {
    pub user_id: Uuid,
    pub method_type: MfaMethodType,
    pub secret: Option<String>,
    pub name: Option<String>,
}
