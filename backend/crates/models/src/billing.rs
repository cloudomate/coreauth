use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// PLAN
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_monthly_cents: i32,
    pub price_yearly_cents: Option<i32>,
    pub mau_limit: i32,
    pub app_limit: Option<i32>,
    pub connection_limit: Option<i32>,
    pub action_limit: Option<i32>,
    pub features: serde_json::Value,
    pub stripe_price_id_monthly: Option<String>,
    pub stripe_price_id_yearly: Option<String>,
    pub is_public: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanFeatures {
    pub mfa: bool,
    pub sso: bool,
    pub custom_domain: bool,
    pub audit_logs: bool,
    pub support: String, // community, email, priority, dedicated
    #[serde(default)]
    pub sla: bool,
    #[serde(default)]
    pub scim: bool,
}

impl Plan {
    pub fn get_features(&self) -> Result<PlanFeatures, serde_json::Error> {
        serde_json::from_value(self.features.clone())
    }

    pub fn is_unlimited_apps(&self) -> bool {
        self.app_limit.is_none()
    }

    pub fn is_unlimited_connections(&self) -> bool {
        self.connection_limit.is_none()
    }

    pub fn monthly_price_dollars(&self) -> f64 {
        self.price_monthly_cents as f64 / 100.0
    }

    pub fn yearly_price_dollars(&self) -> Option<f64> {
        self.price_yearly_cents.map(|c| c as f64 / 100.0)
    }
}

// ============================================================================
// SUBSCRIPTION
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Trialing,
    Active,
    PastDue,
    Canceled,
    Paused,
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trialing => write!(f, "trialing"),
            Self::Active => write!(f, "active"),
            Self::PastDue => write!(f, "past_due"),
            Self::Canceled => write!(f, "canceled"),
            Self::Paused => write!(f, "paused"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BillingCycle {
    Monthly,
    Yearly,
}

impl std::fmt::Display for BillingCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Monthly => write!(f, "monthly"),
            Self::Yearly => write!(f, "yearly"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: String,
    pub status: String,
    pub billing_cycle: String,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_payment_method_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Subscription {
    pub fn is_active(&self) -> bool {
        self.status == "active" || self.status == "trialing"
    }

    pub fn is_trialing(&self) -> bool {
        self.status == "trialing"
    }

    pub fn trial_days_remaining(&self) -> Option<i64> {
        self.trial_ends_at.map(|ends_at| {
            let now = Utc::now();
            if ends_at > now {
                (ends_at - now).num_days()
            } else {
                0
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionWithPlan {
    #[serde(flatten)]
    pub subscription: Subscription,
    pub plan: Plan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscription {
    pub organization_id: Uuid,
    pub plan_id: String,
    pub billing_cycle: BillingCycle,
    pub trial_days: Option<i32>,
    pub stripe_customer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscription {
    pub plan_id: Option<String>,
    pub status: Option<String>,
    pub billing_cycle: Option<String>,
    pub cancel_at_period_end: Option<bool>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_payment_method_id: Option<String>,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
}

// ============================================================================
// USAGE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UsageRecord {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub mau_count: i32,
    pub login_count: i32,
    pub failed_login_count: i32,
    pub signup_count: i32,
    pub api_calls: i32,
    pub webhook_deliveries: i32,
    pub scim_operations: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActiveUser {
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub period: NaiveDate,
    pub first_active_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub login_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub mau_count: i32,
    pub mau_limit: i32,
    pub mau_percentage: i32,
    pub apps_count: i32,
    pub apps_limit: Option<i32>,
    pub connections_count: i32,
    pub connections_limit: Option<i32>,
    pub login_count: i32,
    pub within_limits: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanLimits {
    pub within_limits: bool,
    pub mau_current: i32,
    pub mau_limit: i32,
    pub mau_percentage: i32,
    pub apps_current: i32,
    pub apps_limit: Option<i32>,
    pub connections_current: i32,
    pub connections_limit: Option<i32>,
}

// ============================================================================
// INVOICE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Void,
    Uncollectible,
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Open => write!(f, "open"),
            Self::Paid => write!(f, "paid"),
            Self::Void => write!(f, "void"),
            Self::Uncollectible => write!(f, "uncollectible"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub stripe_invoice_id: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub invoice_number: Option<String>,
    pub amount_cents: i32,
    pub amount_paid_cents: i32,
    pub currency: String,
    pub status: String,
    pub description: Option<String>,
    pub invoice_pdf_url: Option<String>,
    pub hosted_invoice_url: Option<String>,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub voided_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invoice {
    pub fn amount_dollars(&self) -> f64 {
        self.amount_cents as f64 / 100.0
    }

    pub fn amount_paid_dollars(&self) -> f64 {
        self.amount_paid_cents as f64 / 100.0
    }

    pub fn is_paid(&self) -> bool {
        self.status == "paid"
    }
}

// ============================================================================
// PAYMENT METHOD
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub stripe_payment_method_id: String,
    #[sqlx(rename = "type")]
    pub method_type: String,
    pub is_default: bool,
    pub card_brand: Option<String>,
    pub card_last4: Option<String>,
    pub card_exp_month: Option<i32>,
    pub card_exp_year: Option<i32>,
    pub billing_details: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PaymentMethod {
    pub fn display_name(&self) -> String {
        match (self.card_brand.as_ref(), self.card_last4.as_ref()) {
            (Some(brand), Some(last4)) => format!("{} ending in {}", brand, last4),
            _ => self.method_type.clone(),
        }
    }

    pub fn is_expired(&self) -> bool {
        if let (Some(month), Some(year)) = (self.card_exp_month, self.card_exp_year) {
            let now = Utc::now();
            let current_year = now.format("%Y").to_string().parse::<i32>().unwrap_or(0);
            let current_month = now.format("%m").to_string().parse::<i32>().unwrap_or(0);

            year < current_year || (year == current_year && month < current_month)
        } else {
            false
        }
    }
}

// ============================================================================
// BILLING EVENT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BillingEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub event_type: String,
    pub stripe_event_id: Option<String>,
    pub data: serde_json::Value,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// API REQUESTS/RESPONSES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCheckoutRequest {
    pub plan_id: String,
    pub billing_cycle: BillingCycle,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingPortalResponse {
    pub portal_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingOverview {
    pub subscription: Option<SubscriptionWithPlan>,
    pub usage: UsageSummary,
    pub payment_methods: Vec<PaymentMethod>,
    pub recent_invoices: Vec<Invoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePlanRequest {
    pub plan_id: String,
    pub billing_cycle: Option<BillingCycle>,
}
