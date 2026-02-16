-- Migration: 006_billing.sql
-- Description: SaaS billing infrastructure - plans, subscriptions, usage tracking

-- ============================================================================
-- SUBSCRIPTION PLANS
-- ============================================================================
CREATE TABLE IF NOT EXISTS plans (
    id TEXT PRIMARY KEY,                    -- 'free', 'starter', 'pro', 'enterprise'
    name TEXT NOT NULL,
    description TEXT,
    price_monthly_cents INTEGER NOT NULL DEFAULT 0,
    price_yearly_cents INTEGER,             -- Annual discount (optional)
    mau_limit INTEGER NOT NULL,             -- Monthly Active Users limit
    app_limit INTEGER,                      -- NULL = unlimited
    connection_limit INTEGER,               -- SSO connections limit
    action_limit INTEGER,                   -- Actions/hooks limit
    features JSONB NOT NULL DEFAULT '{}',   -- Feature flags (sso, mfa, custom_domain, etc.)
    stripe_price_id_monthly TEXT,
    stripe_price_id_yearly TEXT,
    is_public BOOLEAN NOT NULL DEFAULT true,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default plans
INSERT INTO plans (id, name, description, price_monthly_cents, price_yearly_cents, mau_limit, app_limit, connection_limit, action_limit, features, display_order) VALUES
('free', 'Free', 'For personal projects and testing', 0, NULL, 1000, 1, 2, 1,
 '{"mfa": true, "sso": false, "custom_domain": false, "audit_logs": false, "support": "community"}', 1),
('starter', 'Starter', 'For small teams getting started', 2900, 29000, 10000, 5, 10, 5,
 '{"mfa": true, "sso": true, "custom_domain": false, "audit_logs": true, "support": "email"}', 2),
('pro', 'Pro', 'For growing businesses', 14900, 149000, 50000, NULL, NULL, NULL,
 '{"mfa": true, "sso": true, "custom_domain": true, "audit_logs": true, "support": "priority"}', 3),
('enterprise', 'Enterprise', 'For large organizations with custom needs', 0, NULL, 0, NULL, NULL, NULL,
 '{"mfa": true, "sso": true, "custom_domain": true, "audit_logs": true, "support": "dedicated", "sla": true, "scim": true}', 4)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- ORGANIZATION SUBSCRIPTIONS
-- ============================================================================
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    plan_id TEXT NOT NULL REFERENCES plans(id),
    status TEXT NOT NULL DEFAULT 'trialing',  -- trialing, active, past_due, canceled, paused
    billing_cycle TEXT NOT NULL DEFAULT 'monthly',  -- monthly, yearly
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    trial_ends_at TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    canceled_at TIMESTAMPTZ,
    stripe_customer_id TEXT,
    stripe_subscription_id TEXT,
    stripe_payment_method_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id)
);

CREATE INDEX idx_subscriptions_organization_id ON subscriptions(organization_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_stripe_customer_id ON subscriptions(stripe_customer_id);
CREATE INDEX idx_subscriptions_stripe_subscription_id ON subscriptions(stripe_subscription_id);

-- ============================================================================
-- USAGE RECORDS (Monthly aggregates)
-- ============================================================================
CREATE TABLE IF NOT EXISTS usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    period_start DATE NOT NULL,             -- First day of month
    period_end DATE NOT NULL,               -- Last day of month
    mau_count INTEGER NOT NULL DEFAULT 0,   -- Monthly Active Users
    login_count INTEGER NOT NULL DEFAULT 0, -- Total logins
    failed_login_count INTEGER NOT NULL DEFAULT 0,
    signup_count INTEGER NOT NULL DEFAULT 0,
    api_calls INTEGER NOT NULL DEFAULT 0,
    webhook_deliveries INTEGER NOT NULL DEFAULT 0,
    scim_operations INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, period_start)
);

CREATE INDEX idx_usage_records_organization_period ON usage_records(organization_id, period_start DESC);

-- ============================================================================
-- ACTIVE USERS (For MAU tracking - tracks unique users per month)
-- ============================================================================
CREATE TABLE IF NOT EXISTS active_users (
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period DATE NOT NULL,                   -- First day of month (e.g., 2026-02-01)
    first_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    login_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (organization_id, user_id, period)
);

CREATE INDEX idx_active_users_period ON active_users(period);
CREATE INDEX idx_active_users_organization_period ON active_users(organization_id, period);

-- ============================================================================
-- INVOICES (Synced from Stripe)
-- ============================================================================
CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    subscription_id UUID REFERENCES subscriptions(id) ON DELETE SET NULL,
    stripe_invoice_id TEXT UNIQUE,
    stripe_payment_intent_id TEXT,
    invoice_number TEXT,
    amount_cents INTEGER NOT NULL,
    amount_paid_cents INTEGER NOT NULL DEFAULT 0,
    currency TEXT NOT NULL DEFAULT 'usd',
    status TEXT NOT NULL,                   -- draft, open, paid, void, uncollectible
    description TEXT,
    invoice_pdf_url TEXT,
    hosted_invoice_url TEXT,
    period_start TIMESTAMPTZ,
    period_end TIMESTAMPTZ,
    due_date TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    voided_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoices_organization_id ON invoices(organization_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_stripe_invoice_id ON invoices(stripe_invoice_id);

-- ============================================================================
-- PAYMENT METHODS (Cached from Stripe)
-- ============================================================================
CREATE TABLE IF NOT EXISTS payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    stripe_payment_method_id TEXT NOT NULL UNIQUE,
    type TEXT NOT NULL,                     -- card, bank_account, etc.
    is_default BOOLEAN NOT NULL DEFAULT false,
    card_brand TEXT,                        -- visa, mastercard, amex, etc.
    card_last4 TEXT,
    card_exp_month INTEGER,
    card_exp_year INTEGER,
    billing_details JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payment_methods_organization_id ON payment_methods(organization_id);

-- ============================================================================
-- BILLING EVENTS (Audit trail for billing operations)
-- ============================================================================
CREATE TABLE IF NOT EXISTS billing_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,               -- subscription.created, payment.succeeded, etc.
    stripe_event_id TEXT UNIQUE,
    data JSONB NOT NULL DEFAULT '{}',
    processed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_events_organization_id ON billing_events(organization_id);
CREATE INDEX idx_billing_events_event_type ON billing_events(event_type);
CREATE INDEX idx_billing_events_created_at ON billing_events(created_at DESC);

-- ============================================================================
-- FUNCTIONS
-- ============================================================================

-- Function to get current month start
CREATE OR REPLACE FUNCTION get_current_month_start()
RETURNS DATE AS $$
BEGIN
    RETURN DATE_TRUNC('month', CURRENT_DATE)::DATE;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function to record user activity (for MAU tracking)
CREATE OR REPLACE FUNCTION record_user_activity(
    p_organization_id UUID,
    p_user_id UUID
) RETURNS VOID AS $$
DECLARE
    v_period DATE;
    v_period_end DATE;
BEGIN
    v_period := get_current_month_start();
    v_period_end := (v_period + INTERVAL '1 month' - INTERVAL '1 day')::DATE;

    -- Upsert into active_users
    INSERT INTO active_users (organization_id, user_id, period, first_active_at, last_active_at, login_count)
    VALUES (p_organization_id, p_user_id, v_period, NOW(), NOW(), 1)
    ON CONFLICT (organization_id, user_id, period)
    DO UPDATE SET
        last_active_at = NOW(),
        login_count = active_users.login_count + 1;

    -- Update or create usage_records with current MAU count
    INSERT INTO usage_records (organization_id, period_start, period_end, mau_count, login_count)
    VALUES (p_organization_id, v_period, v_period_end, 1, 1)
    ON CONFLICT (organization_id, period_start)
    DO UPDATE SET
        mau_count = (
            SELECT COUNT(DISTINCT user_id)
            FROM active_users
            WHERE organization_id = p_organization_id AND period = v_period
        ),
        login_count = usage_records.login_count + 1,
        updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Function to check if organization is within plan limits
CREATE OR REPLACE FUNCTION check_plan_limits(
    p_organization_id UUID
) RETURNS TABLE (
    within_limits BOOLEAN,
    mau_current INTEGER,
    mau_limit INTEGER,
    mau_percentage INTEGER,
    apps_current INTEGER,
    apps_limit INTEGER,
    connections_current INTEGER,
    connections_limit INTEGER
) AS $$
DECLARE
    v_plan plans%ROWTYPE;
    v_mau_current INTEGER;
    v_apps_current INTEGER;
    v_connections_current INTEGER;
BEGIN
    -- Get the plan for this organization
    SELECT p.* INTO v_plan
    FROM subscriptions s
    JOIN plans p ON p.id = s.plan_id
    WHERE s.organization_id = p_organization_id
    AND s.status IN ('trialing', 'active');

    -- If no active subscription, use free plan limits
    IF NOT FOUND THEN
        SELECT * INTO v_plan FROM plans WHERE id = 'free';
    END IF;

    -- Get current usage
    SELECT COALESCE(mau_count, 0) INTO v_mau_current
    FROM usage_records
    WHERE organization_id = p_organization_id
    AND period_start = get_current_month_start();

    SELECT COUNT(*) INTO v_apps_current
    FROM applications
    WHERE organization_id = p_organization_id;

    SELECT COUNT(*) INTO v_connections_current
    FROM connections
    WHERE organization_id = p_organization_id;

    RETURN QUERY SELECT
        (v_mau_current < v_plan.mau_limit OR v_plan.mau_limit = 0) AS within_limits,
        COALESCE(v_mau_current, 0) AS mau_current,
        v_plan.mau_limit AS mau_limit,
        CASE WHEN v_plan.mau_limit > 0 THEN (v_mau_current * 100 / v_plan.mau_limit) ELSE 0 END AS mau_percentage,
        v_apps_current AS apps_current,
        v_plan.app_limit AS apps_limit,
        v_connections_current AS connections_current,
        v_plan.connection_limit AS connections_limit;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Auto-update updated_at timestamps
CREATE OR REPLACE FUNCTION update_billing_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_subscriptions_updated_at
    BEFORE UPDATE ON subscriptions
    FOR EACH ROW
    EXECUTE FUNCTION update_billing_updated_at();

CREATE TRIGGER trigger_usage_records_updated_at
    BEFORE UPDATE ON usage_records
    FOR EACH ROW
    EXECUTE FUNCTION update_billing_updated_at();

CREATE TRIGGER trigger_invoices_updated_at
    BEFORE UPDATE ON invoices
    FOR EACH ROW
    EXECUTE FUNCTION update_billing_updated_at();

CREATE TRIGGER trigger_payment_methods_updated_at
    BEFORE UPDATE ON payment_methods
    FOR EACH ROW
    EXECUTE FUNCTION update_billing_updated_at();

-- ============================================================================
-- DEFAULT SUBSCRIPTION FOR EXISTING ORGANIZATIONS
-- ============================================================================
-- Create free subscriptions for any existing organizations that don't have one
INSERT INTO subscriptions (organization_id, plan_id, status, billing_cycle, trial_ends_at)
SELECT
    o.id,
    'free',
    'active',
    'monthly',
    NULL
FROM organizations o
WHERE NOT EXISTS (
    SELECT 1 FROM subscriptions s WHERE s.organization_id = o.id
)
ON CONFLICT (organization_id) DO NOTHING;
