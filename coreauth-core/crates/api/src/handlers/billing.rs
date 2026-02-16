use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use ciam_auth::BillingService;
use ciam_models::{
    billing::{
        BillingOverview, CheckoutResponse, CreateCheckoutRequest, Invoice,
        PaymentMethod, Plan, Subscription, SubscriptionWithPlan, UsageSummary,
    },
    User,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use super::auth::ErrorResponse;

// Type alias for API results
type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ErrorResponse>)>;

fn internal_error(msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    error!("{}", msg);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new("internal_error", msg)),
    )
}

fn bad_request(msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new("bad_request", msg)),
    )
}

fn not_found(msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("not_found", msg)),
    )
}

/// Application state containing billing service
#[derive(Clone)]
pub struct BillingState {
    pub billing_service: Arc<BillingService>,
}

// ============================================================================
// PLANS
// ============================================================================

/// List all available plans
pub async fn list_plans(
    State(state): State<BillingState>,
) -> ApiResult<Vec<Plan>> {
    let plans = state
        .billing_service
        .list_plans()
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(Json(plans))
}

/// Get a specific plan
pub async fn get_plan(
    State(state): State<BillingState>,
    Path(plan_id): Path<String>,
) -> ApiResult<Plan> {
    let plan = state
        .billing_service
        .get_plan(&plan_id)
        .await
        .map_err(|e| not_found(&e.to_string()))?;

    Ok(Json(plan))
}

// ============================================================================
// SUBSCRIPTIONS
// ============================================================================

/// Get current organization's subscription
pub async fn get_subscription(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
) -> ApiResult<SubscriptionWithPlan> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let subscription = state
        .billing_service
        .get_subscription_with_plan(org_id)
        .await
        .map_err(|e| not_found(&e.to_string()))?;

    Ok(Json(subscription))
}

/// Get subscription for a specific organization (admin)
pub async fn get_org_subscription(
    State(state): State<BillingState>,
    Path(org_id): Path<Uuid>,
    Extension(_user): Extension<User>,
) -> ApiResult<SubscriptionWithPlan> {
    // TODO: Check if user has admin access to this org

    let subscription = state
        .billing_service
        .get_subscription_with_plan(org_id)
        .await
        .map_err(|e| not_found(&e.to_string()))?;

    Ok(Json(subscription))
}

/// Cancel subscription
pub async fn cancel_subscription(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
) -> ApiResult<Subscription> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let subscription = state
        .billing_service
        .cancel_subscription(org_id)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    info!(organization_id = %org_id, user_id = %user.id, "Subscription cancelled");

    Ok(Json(subscription))
}

// ============================================================================
// USAGE
// ============================================================================

/// Get current usage summary
pub async fn get_usage(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
) -> ApiResult<UsageSummary> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let usage = state
        .billing_service
        .get_usage_summary(org_id)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(Json(usage))
}

/// Get usage for a specific organization (admin)
pub async fn get_org_usage(
    State(state): State<BillingState>,
    Path(org_id): Path<Uuid>,
    Extension(_user): Extension<User>,
) -> ApiResult<UsageSummary> {
    // TODO: Check if user has admin access to this org

    let usage = state
        .billing_service
        .get_usage_summary(org_id)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(Json(usage))
}

// ============================================================================
// INVOICES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListInvoicesQuery {
    pub limit: Option<i64>,
}

/// List invoices for current organization
pub async fn list_invoices(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
    Query(query): Query<ListInvoicesQuery>,
) -> ApiResult<Vec<Invoice>> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let invoices = state
        .billing_service
        .list_invoices(org_id, query.limit)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(Json(invoices))
}

// ============================================================================
// PAYMENT METHODS
// ============================================================================

/// List payment methods
pub async fn list_payment_methods(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
) -> ApiResult<Vec<PaymentMethod>> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let methods = state
        .billing_service
        .list_payment_methods(org_id)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(Json(methods))
}

// ============================================================================
// BILLING OVERVIEW
// ============================================================================

/// Get complete billing overview for dashboard
pub async fn get_billing_overview(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
) -> ApiResult<BillingOverview> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let overview = state
        .billing_service
        .get_billing_overview(org_id)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(Json(overview))
}

// ============================================================================
// STRIPE CHECKOUT
// ============================================================================

/// Create Stripe checkout session for upgrading/subscribing
pub async fn create_checkout(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
    Json(request): Json<CreateCheckoutRequest>,
) -> ApiResult<CheckoutResponse> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let response = state
        .billing_service
        .create_checkout_session(org_id, request)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    info!(
        organization_id = %org_id,
        user_id = %user.id,
        "Created checkout session"
    );

    Ok(Json(response))
}

/// Create Stripe billing portal session
#[derive(Debug, Deserialize)]
pub struct BillingPortalRequest {
    pub return_url: String,
}

#[derive(Debug, Serialize)]
pub struct BillingPortalResponse {
    pub portal_url: String,
}

pub async fn create_billing_portal(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
    Json(request): Json<BillingPortalRequest>,
) -> ApiResult<BillingPortalResponse> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let portal_url = state
        .billing_service
        .create_billing_portal_session(org_id, &request.return_url)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(Json(BillingPortalResponse { portal_url }))
}

// ============================================================================
// STRIPE WEBHOOK
// ============================================================================

/// Handle Stripe webhook events
pub async fn stripe_webhook(
    State(state): State<BillingState>,
    headers: axum::http::HeaderMap,
    body: String,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| bad_request("Missing Stripe signature"))?;

    state
        .billing_service
        .handle_stripe_webhook(&body, signature)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(StatusCode::OK)
}

// ============================================================================
// PLAN LIMITS CHECK
// ============================================================================

#[derive(Debug, Serialize)]
pub struct PlanLimitsResponse {
    pub can_add_application: bool,
    pub can_add_connection: bool,
    pub mau_percentage: i32,
    pub within_limits: bool,
}

/// Check if organization is within plan limits
pub async fn check_limits(
    State(state): State<BillingState>,
    Extension(user): Extension<User>,
) -> ApiResult<PlanLimitsResponse> {
    let org_id = user
        .default_tenant_id
        .ok_or_else(|| bad_request("User must belong to an organization"))?;

    let limits = state
        .billing_service
        .check_plan_limits(org_id)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    let can_add_app = state
        .billing_service
        .can_add_application(org_id)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    let can_add_conn = state
        .billing_service
        .can_add_connection(org_id)
        .await
        .map_err(|e| internal_error(&e.to_string()))?;

    Ok(Json(PlanLimitsResponse {
        can_add_application: can_add_app,
        can_add_connection: can_add_conn,
        mau_percentage: limits.mau_percentage,
        within_limits: limits.within_limits,
    }))
}
