use crate::error::AuthError;
use chrono::{Datelike, NaiveDate, Utc};
use ciam_database::Database;
use ciam_models::{
    billing::{
        BillingCycle, BillingEvent, BillingOverview, ChangePlanRequest, CheckoutResponse,
        CreateCheckoutRequest, CreateSubscription, Invoice, PaymentMethod, Plan, PlanLimits,
        Subscription, SubscriptionWithPlan, UpdateSubscription, UsageRecord, UsageSummary,
    },
    Organization,
};
use sqlx::Row;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Billing service for managing subscriptions, usage, and Stripe integration
pub struct BillingService {
    db: Arc<Database>,
    stripe_secret_key: Option<String>,
    stripe_webhook_secret: Option<String>,
    base_url: String,
}

impl BillingService {
    pub fn new(
        db: Arc<Database>,
        stripe_secret_key: Option<String>,
        stripe_webhook_secret: Option<String>,
        base_url: String,
    ) -> Self {
        Self {
            db,
            stripe_secret_key,
            stripe_webhook_secret,
            base_url,
        }
    }

    // ========================================================================
    // PLANS
    // ========================================================================

    /// Get all public plans
    pub async fn list_plans(&self) -> Result<Vec<Plan>, AuthError> {
        let plans = sqlx::query_as::<_, Plan>(
            r#"
            SELECT * FROM plans
            WHERE is_public = true
            ORDER BY display_order ASC
            "#,
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(plans)
    }

    /// Get a plan by ID
    pub async fn get_plan(&self, plan_id: &str) -> Result<Plan, AuthError> {
        let plan = sqlx::query_as::<_, Plan>(
            r#"
            SELECT * FROM plans WHERE id = $1
            "#,
        )
        .bind(plan_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
        .ok_or_else(|| AuthError::NotFound(format!("Plan not found: {}", plan_id)))?;

        Ok(plan)
    }

    // ========================================================================
    // SUBSCRIPTIONS
    // ========================================================================

    /// Get subscription for an organization
    pub async fn get_subscription(&self, organization_id: Uuid) -> Result<Subscription, AuthError> {
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT * FROM subscriptions WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
        .ok_or_else(|| AuthError::NotFound("No subscription found".to_string()))?;

        Ok(subscription)
    }

    /// Get subscription with plan details
    pub async fn get_subscription_with_plan(
        &self,
        organization_id: Uuid,
    ) -> Result<SubscriptionWithPlan, AuthError> {
        let subscription = self.get_subscription(organization_id).await?;
        let plan = self.get_plan(&subscription.plan_id).await?;

        Ok(SubscriptionWithPlan { subscription, plan })
    }

    /// Create a new subscription (used during tenant signup)
    pub async fn create_subscription(
        &self,
        request: CreateSubscription,
    ) -> Result<Subscription, AuthError> {
        let trial_ends_at = request.trial_days.map(|days| Utc::now() + chrono::Duration::days(days as i64));

        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscriptions (
                organization_id, plan_id, status, billing_cycle,
                trial_ends_at, stripe_customer_id
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(request.organization_id)
        .bind(&request.plan_id)
        .bind(if trial_ends_at.is_some() { "trialing" } else { "active" })
        .bind(request.billing_cycle.to_string())
        .bind(trial_ends_at)
        .bind(&request.stripe_customer_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        info!(
            organization_id = %request.organization_id,
            plan_id = %request.plan_id,
            "Created subscription"
        );

        Ok(subscription)
    }

    /// Update a subscription
    pub async fn update_subscription(
        &self,
        organization_id: Uuid,
        update: UpdateSubscription,
    ) -> Result<Subscription, AuthError> {
        let mut query = String::from("UPDATE subscriptions SET updated_at = NOW()");
        let mut param_count = 1;

        if update.plan_id.is_some() {
            param_count += 1;
            query.push_str(&format!(", plan_id = ${}", param_count));
        }
        if update.status.is_some() {
            param_count += 1;
            query.push_str(&format!(", status = ${}", param_count));
        }
        if update.billing_cycle.is_some() {
            param_count += 1;
            query.push_str(&format!(", billing_cycle = ${}", param_count));
        }
        if update.cancel_at_period_end.is_some() {
            param_count += 1;
            query.push_str(&format!(", cancel_at_period_end = ${}", param_count));
        }
        if update.stripe_subscription_id.is_some() {
            param_count += 1;
            query.push_str(&format!(", stripe_subscription_id = ${}", param_count));
        }
        if update.current_period_start.is_some() {
            param_count += 1;
            query.push_str(&format!(", current_period_start = ${}", param_count));
        }
        if update.current_period_end.is_some() {
            param_count += 1;
            query.push_str(&format!(", current_period_end = ${}", param_count));
        }

        query.push_str(" WHERE organization_id = $1 RETURNING *");

        // Build the query dynamically - for simplicity, using a simpler approach
        let subscription = sqlx::query_as::<_, Subscription>(&query)
            .bind(organization_id)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(subscription)
    }

    /// Cancel subscription at period end
    pub async fn cancel_subscription(&self, organization_id: Uuid) -> Result<Subscription, AuthError> {
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            UPDATE subscriptions
            SET cancel_at_period_end = true, canceled_at = NOW(), updated_at = NOW()
            WHERE organization_id = $1
            RETURNING *
            "#,
        )
        .bind(organization_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        info!(organization_id = %organization_id, "Subscription marked for cancellation");

        Ok(subscription)
    }

    // ========================================================================
    // USAGE TRACKING
    // ========================================================================

    /// Record user activity (called on every login)
    pub async fn record_user_activity(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AuthError> {
        sqlx::query("SELECT record_user_activity($1, $2)")
            .bind(organization_id)
            .bind(user_id)
            .execute(self.db.pool())
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get current usage for an organization
    pub async fn get_current_usage(&self, organization_id: Uuid) -> Result<UsageRecord, AuthError> {
        let period_start = get_current_month_start();

        let usage = sqlx::query_as::<_, UsageRecord>(
            r#"
            SELECT * FROM usage_records
            WHERE organization_id = $1 AND period_start = $2
            "#,
        )
        .bind(organization_id)
        .bind(period_start)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        // If no record exists, return empty usage
        Ok(usage.unwrap_or_else(|| UsageRecord {
            id: Uuid::new_v4(),
            organization_id,
            period_start,
            period_end: get_current_month_end(),
            mau_count: 0,
            login_count: 0,
            failed_login_count: 0,
            signup_count: 0,
            api_calls: 0,
            webhook_deliveries: 0,
            scim_operations: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }))
    }

    /// Get usage summary with plan limits
    pub async fn get_usage_summary(&self, organization_id: Uuid) -> Result<UsageSummary, AuthError> {
        let usage = self.get_current_usage(organization_id).await?;
        let limits = self.check_plan_limits(organization_id).await?;

        Ok(UsageSummary {
            period_start: usage.period_start,
            period_end: usage.period_end,
            mau_count: limits.mau_current,
            mau_limit: limits.mau_limit,
            mau_percentage: limits.mau_percentage,
            apps_count: limits.apps_current,
            apps_limit: limits.apps_limit,
            connections_count: limits.connections_current,
            connections_limit: limits.connections_limit,
            login_count: usage.login_count,
            within_limits: limits.within_limits,
        })
    }

    /// Check plan limits for an organization
    pub async fn check_plan_limits(&self, organization_id: Uuid) -> Result<PlanLimits, AuthError> {
        let row = sqlx::query(
            r#"
            SELECT * FROM check_plan_limits($1)
            "#,
        )
        .bind(organization_id)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(PlanLimits {
            within_limits: row.get("within_limits"),
            mau_current: row.get("mau_current"),
            mau_limit: row.get("mau_limit"),
            mau_percentage: row.get("mau_percentage"),
            apps_current: row.get("apps_current"),
            apps_limit: row.get("apps_limit"),
            connections_current: row.get("connections_current"),
            connections_limit: row.get("connections_limit"),
        })
    }

    /// Check if organization can add more applications
    pub async fn can_add_application(&self, organization_id: Uuid) -> Result<bool, AuthError> {
        let limits = self.check_plan_limits(organization_id).await?;

        Ok(match limits.apps_limit {
            Some(limit) => limits.apps_current < limit,
            None => true, // Unlimited
        })
    }

    /// Check if organization can add more connections
    pub async fn can_add_connection(&self, organization_id: Uuid) -> Result<bool, AuthError> {
        let limits = self.check_plan_limits(organization_id).await?;

        Ok(match limits.connections_limit {
            Some(limit) => limits.connections_current < limit,
            None => true, // Unlimited
        })
    }

    // ========================================================================
    // INVOICES
    // ========================================================================

    /// List invoices for an organization
    pub async fn list_invoices(
        &self,
        organization_id: Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<Invoice>, AuthError> {
        let invoices = sqlx::query_as::<_, Invoice>(
            r#"
            SELECT * FROM invoices
            WHERE organization_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(organization_id)
        .bind(limit.unwrap_or(10))
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(invoices)
    }

    // ========================================================================
    // PAYMENT METHODS
    // ========================================================================

    /// List payment methods for an organization
    pub async fn list_payment_methods(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<PaymentMethod>, AuthError> {
        let methods = sqlx::query_as::<_, PaymentMethod>(
            r#"
            SELECT * FROM payment_methods
            WHERE organization_id = $1
            ORDER BY is_default DESC, created_at DESC
            "#,
        )
        .bind(organization_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(methods)
    }

    // ========================================================================
    // BILLING OVERVIEW
    // ========================================================================

    /// Get complete billing overview for dashboard
    pub async fn get_billing_overview(
        &self,
        organization_id: Uuid,
    ) -> Result<BillingOverview, AuthError> {
        let subscription = self.get_subscription_with_plan(organization_id).await.ok();
        let usage = self.get_usage_summary(organization_id).await?;
        let payment_methods = self.list_payment_methods(organization_id).await?;
        let recent_invoices = self.list_invoices(organization_id, Some(5)).await?;

        Ok(BillingOverview {
            subscription,
            usage,
            payment_methods,
            recent_invoices,
        })
    }

    // ========================================================================
    // STRIPE INTEGRATION (Stubbed - requires stripe-rust crate)
    // ========================================================================

    /// Create Stripe checkout session
    pub async fn create_checkout_session(
        &self,
        organization_id: Uuid,
        request: CreateCheckoutRequest,
    ) -> Result<CheckoutResponse, AuthError> {
        let _stripe_key = self
            .stripe_secret_key
            .as_ref()
            .ok_or_else(|| AuthError::ConfigurationError("Stripe not configured".to_string()))?;

        let plan = self.get_plan(&request.plan_id).await?;

        let price_id = match request.billing_cycle {
            BillingCycle::Monthly => plan.stripe_price_id_monthly,
            BillingCycle::Yearly => plan.stripe_price_id_yearly,
        }
        .ok_or_else(|| {
            AuthError::ConfigurationError(format!(
                "No Stripe price configured for plan {} with {} billing",
                request.plan_id, request.billing_cycle
            ))
        })?;

        // TODO: Implement actual Stripe API call
        // For now, return a placeholder
        warn!("Stripe checkout session creation not yet implemented");

        Ok(CheckoutResponse {
            checkout_url: format!(
                "{}/billing/checkout?plan={}&org={}",
                self.base_url, request.plan_id, organization_id
            ),
            session_id: format!("cs_test_{}", Uuid::new_v4()),
        })
    }

    /// Create Stripe billing portal session
    pub async fn create_billing_portal_session(
        &self,
        organization_id: Uuid,
        return_url: &str,
    ) -> Result<String, AuthError> {
        let subscription = self.get_subscription(organization_id).await?;

        let customer_id = subscription
            .stripe_customer_id
            .ok_or_else(|| AuthError::NotFound("No Stripe customer found".to_string()))?;

        // TODO: Implement actual Stripe API call
        warn!("Stripe billing portal not yet implemented");

        Ok(format!(
            "{}/billing/portal?customer={}",
            self.base_url, customer_id
        ))
    }

    /// Handle Stripe webhook
    pub async fn handle_stripe_webhook(
        &self,
        payload: &str,
        signature: &str,
    ) -> Result<(), AuthError> {
        let _webhook_secret = self
            .stripe_webhook_secret
            .as_ref()
            .ok_or_else(|| AuthError::ConfigurationError("Stripe webhook secret not configured".to_string()))?;

        // TODO: Verify signature and process webhook
        // This would parse the event and handle:
        // - checkout.session.completed
        // - customer.subscription.updated
        // - customer.subscription.deleted
        // - invoice.paid
        // - invoice.payment_failed

        warn!("Stripe webhook handling not yet implemented");

        Ok(())
    }

    /// Log a billing event
    pub async fn log_billing_event(
        &self,
        organization_id: Uuid,
        event_type: &str,
        stripe_event_id: Option<&str>,
        data: serde_json::Value,
    ) -> Result<BillingEvent, AuthError> {
        let event = sqlx::query_as::<_, BillingEvent>(
            r#"
            INSERT INTO billing_events (organization_id, event_type, stripe_event_id, data, processed_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING *
            "#,
        )
        .bind(organization_id)
        .bind(event_type)
        .bind(stripe_event_id)
        .bind(data)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

        Ok(event)
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn get_current_month_start() -> NaiveDate {
    let now = Utc::now();
    NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap()
}

fn get_current_month_end() -> NaiveDate {
    let start = get_current_month_start();
    // Get last day of month
    let next_month = if start.month() == 12 {
        NaiveDate::from_ymd_opt(start.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(start.year(), start.month() + 1, 1)
    };
    next_month.unwrap() - chrono::Duration::days(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_month_start() {
        let start = get_current_month_start();
        assert_eq!(start.day(), 1);
    }

    #[test]
    fn test_get_current_month_end() {
        let end = get_current_month_end();
        assert!(end.day() >= 28);
    }
}
