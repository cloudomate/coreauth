-- ============================================================
-- CoreAuth CIAM - Platform Features
-- ============================================================
-- Billing, webhooks, SCIM, actions/hooks
-- ============================================================

-- ============================================================
-- BILLING: Subscription Plans
-- ============================================================

CREATE TABLE plans (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,

    -- Pricing
    price_monthly_cents INTEGER DEFAULT 0,
    price_yearly_cents INTEGER DEFAULT 0,

    -- Limits
    mau_limit INTEGER NOT NULL,
    app_limit INTEGER,
    connection_limit INTEGER,
    action_limit INTEGER,

    -- Features
    features JSONB DEFAULT '{}',

    -- Stripe
    stripe_price_id_monthly VARCHAR(255),
    stripe_price_id_yearly VARCHAR(255),

    display_order INTEGER DEFAULT 0,
    is_public BOOLEAN DEFAULT true,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default plans
INSERT INTO plans (id, name, description, price_monthly_cents, price_yearly_cents, mau_limit, app_limit, connection_limit, action_limit, features, display_order) VALUES
    ('free', 'Free', 'For personal projects and development', 0, 0, 1000, 1, 2, 5,
     '{"social_login": true, "basic_mfa": true, "community_support": true}', 1),
    ('starter', 'Starter', 'For small teams getting started', 2900, 29000, 10000, 5, 10, 20,
     '{"social_login": true, "basic_mfa": true, "email_support": true, "custom_branding": true}', 2),
    ('pro', 'Pro', 'For growing businesses', 14900, 149000, 50000, NULL, NULL, NULL,
     '{"social_login": true, "advanced_mfa": true, "priority_support": true, "custom_branding": true, "webhooks": true, "scim": true}', 3),
    ('enterprise', 'Enterprise', 'For large organizations', 0, 0, 1000000, NULL, NULL, NULL,
     '{"social_login": true, "advanced_mfa": true, "dedicated_support": true, "custom_branding": true, "webhooks": true, "scim": true, "sla": true, "dedicated_db": true}', 4)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- BILLING: Subscriptions
-- ============================================================

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    plan_id VARCHAR(50) NOT NULL REFERENCES plans(id),

    status VARCHAR(20) NOT NULL DEFAULT 'trialing'
        CHECK (status IN ('trialing', 'active', 'past_due', 'canceled', 'paused')),
    billing_cycle VARCHAR(10) DEFAULT 'monthly'
        CHECK (billing_cycle IN ('monthly', 'yearly')),

    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    trial_ends_at TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN DEFAULT false,

    stripe_customer_id VARCHAR(255),
    stripe_subscription_id VARCHAR(255),
    metadata JSONB DEFAULT '{}',

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id)
);

CREATE INDEX idx_subscriptions_tenant ON subscriptions(tenant_id);
CREATE INDEX idx_subscriptions_stripe ON subscriptions(stripe_subscription_id);

CREATE TRIGGER subscriptions_updated_at
    BEFORE UPDATE ON subscriptions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- BILLING: Usage Records
-- ============================================================

CREATE TABLE usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    period_start DATE NOT NULL,
    period_end DATE NOT NULL,

    mau_count INTEGER DEFAULT 0,
    login_count INTEGER DEFAULT 0,
    failed_login_count INTEGER DEFAULT 0,
    signup_count INTEGER DEFAULT 0,
    api_calls INTEGER DEFAULT 0,
    webhook_deliveries INTEGER DEFAULT 0,
    scim_operations INTEGER DEFAULT 0,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, period_start)
);

CREATE INDEX idx_usage_records_tenant ON usage_records(tenant_id);
CREATE INDEX idx_usage_records_period ON usage_records(period_start, period_end);

CREATE TRIGGER usage_records_updated_at
    BEFORE UPDATE ON usage_records
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- BILLING: Active Users (MAU Tracking)
-- ============================================================

CREATE TABLE active_users (
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    period DATE NOT NULL,

    first_active_at TIMESTAMPTZ DEFAULT NOW(),
    last_active_at TIMESTAMPTZ DEFAULT NOW(),
    login_count INTEGER DEFAULT 1,

    PRIMARY KEY (tenant_id, user_id, period)
);

CREATE INDEX idx_active_users_period ON active_users(period);

-- Helper function for MAU tracking
CREATE OR REPLACE FUNCTION record_user_activity(p_tenant_id UUID, p_user_id UUID)
RETURNS void AS $$
BEGIN
    INSERT INTO active_users (tenant_id, user_id, period, first_active_at, last_active_at, login_count)
    VALUES (p_tenant_id, p_user_id, date_trunc('month', CURRENT_DATE)::date, NOW(), NOW(), 1)
    ON CONFLICT (tenant_id, user_id, period)
    DO UPDATE SET last_active_at = NOW(), login_count = active_users.login_count + 1;
END;
$$ LANGUAGE plpgsql;

-- ============================================================
-- BILLING: Invoices
-- ============================================================

CREATE TABLE invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    subscription_id UUID REFERENCES subscriptions(id) ON DELETE SET NULL,

    stripe_invoice_id VARCHAR(255) UNIQUE,
    stripe_payment_intent_id VARCHAR(255),

    amount_cents INTEGER NOT NULL,
    amount_paid_cents INTEGER DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'usd',

    status VARCHAR(20) NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'open', 'paid', 'void', 'uncollectible')),

    invoice_pdf_url TEXT,
    hosted_invoice_url TEXT,

    period_start TIMESTAMPTZ,
    period_end TIMESTAMPTZ,
    due_date TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    voided_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_invoices_tenant ON invoices(tenant_id);
CREATE INDEX idx_invoices_status ON invoices(status);

CREATE TRIGGER invoices_updated_at
    BEFORE UPDATE ON invoices
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- WEBHOOKS: Configurations
-- ============================================================

CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    secret VARCHAR(255) NOT NULL,
    events TEXT[] NOT NULL,

    is_enabled BOOLEAN DEFAULT true,
    retry_policy JSONB DEFAULT '{"max_retries": 3, "initial_delay_ms": 1000}',
    custom_headers JSONB DEFAULT '{}',

    -- Statistics
    total_deliveries INTEGER DEFAULT 0,
    successful_deliveries INTEGER DEFAULT 0,
    failed_deliveries INTEGER DEFAULT 0,
    last_triggered_at TIMESTAMPTZ,
    last_success_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_webhooks_tenant ON webhooks(tenant_id);
CREATE INDEX idx_webhooks_enabled ON webhooks(tenant_id, is_enabled) WHERE is_enabled = true;

CREATE TRIGGER webhooks_updated_at
    BEFORE UPDATE ON webhooks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- WEBHOOKS: Deliveries
-- ============================================================

CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,

    event_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,

    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'success', 'failed', 'retrying')),

    request_headers JSONB,
    request_body TEXT,
    response_status INTEGER,
    response_headers JSONB,
    response_body TEXT,

    attempt_count INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    next_retry_at TIMESTAMPTZ,
    last_error TEXT,

    delivered_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_webhook_deliveries_webhook ON webhook_deliveries(webhook_id);
CREATE INDEX idx_webhook_deliveries_status ON webhook_deliveries(status);
CREATE INDEX idx_webhook_deliveries_retry ON webhook_deliveries(next_retry_at) WHERE status = 'retrying';

-- Update webhook stats on delivery
CREATE OR REPLACE FUNCTION update_webhook_stats()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE webhooks SET
        total_deliveries = total_deliveries + 1,
        successful_deliveries = successful_deliveries + CASE WHEN NEW.status = 'success' THEN 1 ELSE 0 END,
        failed_deliveries = failed_deliveries + CASE WHEN NEW.status = 'failed' THEN 1 ELSE 0 END,
        last_triggered_at = NOW(),
        last_success_at = CASE WHEN NEW.status = 'success' THEN NOW() ELSE last_success_at END,
        last_failure_at = CASE WHEN NEW.status = 'failed' THEN NOW() ELSE last_failure_at END,
        updated_at = NOW()
    WHERE id = NEW.webhook_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER webhook_delivery_stats_trigger
    AFTER INSERT OR UPDATE OF status ON webhook_deliveries
    FOR EACH ROW
    WHEN (NEW.status IN ('success', 'failed'))
    EXECUTE FUNCTION update_webhook_stats();

-- ============================================================
-- WEBHOOKS: Event Types Reference
-- ============================================================

CREATE TABLE webhook_event_types (
    id VARCHAR(100) PRIMARY KEY,
    category VARCHAR(50) NOT NULL,
    description TEXT,
    payload_schema JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

INSERT INTO webhook_event_types (id, category, description) VALUES
    ('user.created', 'user', 'A new user was created'),
    ('user.updated', 'user', 'User profile was updated'),
    ('user.deleted', 'user', 'User was deleted'),
    ('user.login', 'user', 'User logged in successfully'),
    ('user.logout', 'user', 'User logged out'),
    ('user.password_changed', 'user', 'User changed their password'),
    ('user.email_verified', 'user', 'User verified their email'),
    ('user.mfa_enabled', 'user', 'User enabled MFA'),
    ('user.mfa_disabled', 'user', 'User disabled MFA'),
    ('tenant.created', 'tenant', 'A new tenant was created'),
    ('tenant.updated', 'tenant', 'Tenant settings were updated'),
    ('application.created', 'application', 'A new application was registered'),
    ('application.updated', 'application', 'Application was updated'),
    ('application.deleted', 'application', 'Application was deleted'),
    ('connection.created', 'connection', 'A new SSO connection was added'),
    ('connection.updated', 'connection', 'SSO connection was updated'),
    ('session.created', 'session', 'New session was created'),
    ('session.revoked', 'session', 'Session was revoked')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- SCIM: Tokens
-- ============================================================

CREATE TABLE scim_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) UNIQUE NOT NULL,
    token_prefix VARCHAR(20) NOT NULL,

    last_used_at TIMESTAMPTZ,
    request_count INTEGER DEFAULT 0,

    expires_at TIMESTAMPTZ,
    is_active BOOLEAN DEFAULT true,
    revoked_at TIMESTAMPTZ,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_scim_tokens_tenant ON scim_tokens(tenant_id);
CREATE INDEX idx_scim_tokens_prefix ON scim_tokens(token_prefix);

-- ============================================================
-- SCIM: Groups
-- ============================================================

CREATE TABLE scim_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    display_name VARCHAR(255) NOT NULL,
    external_id VARCHAR(255),
    role_id UUID REFERENCES roles(id) ON DELETE SET NULL,
    description TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(tenant_id, display_name)
);

CREATE INDEX idx_scim_groups_tenant ON scim_groups(tenant_id);
CREATE INDEX idx_scim_groups_external ON scim_groups(tenant_id, external_id);

CREATE TRIGGER scim_groups_updated_at
    BEFORE UPDATE ON scim_groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- SCIM: Group Members
-- ============================================================

CREATE TABLE scim_group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES scim_groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    created_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(group_id, user_id)
);

CREATE INDEX idx_scim_group_members_group ON scim_group_members(group_id);
CREATE INDEX idx_scim_group_members_user ON scim_group_members(user_id);

-- ============================================================
-- SCIM: Configurations
-- ============================================================

CREATE TABLE scim_configurations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID UNIQUE NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    is_enabled BOOLEAN DEFAULT false,
    auto_create_users BOOLEAN DEFAULT true,
    auto_update_users BOOLEAN DEFAULT true,
    auto_deactivate_users BOOLEAN DEFAULT false,
    sync_groups BOOLEAN DEFAULT true,

    attribute_mapping JSONB DEFAULT '{}',
    default_role VARCHAR(100) DEFAULT 'member',

    total_users_synced INTEGER DEFAULT 0,
    total_groups_synced INTEGER DEFAULT 0,
    last_sync_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TRIGGER scim_configurations_updated_at
    BEFORE UPDATE ON scim_configurations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- ACTIONS: JavaScript Hooks
-- ============================================================

CREATE TABLE actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    name VARCHAR(255) NOT NULL,
    description TEXT,
    trigger_type VARCHAR(50) NOT NULL CHECK (trigger_type IN (
        'pre_login', 'post_login',
        'pre_registration', 'post_registration',
        'pre_token_issue', 'post_token_issue',
        'pre_user_update', 'post_user_update',
        'pre_password_reset', 'post_password_reset'
    )),

    code TEXT NOT NULL,
    runtime VARCHAR(20) DEFAULT 'javascript',
    timeout_seconds INTEGER DEFAULT 10,
    secrets JSONB DEFAULT '{}',
    execution_order INTEGER DEFAULT 0,

    is_enabled BOOLEAN DEFAULT true,

    -- Statistics
    total_executions BIGINT DEFAULT 0,
    successful_executions BIGINT DEFAULT 0,
    failed_executions BIGINT DEFAULT 0,
    avg_execution_time_ms DOUBLE PRECISION DEFAULT 0,
    last_executed_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_actions_tenant ON actions(tenant_id);
CREATE INDEX idx_actions_trigger ON actions(trigger_type, is_enabled) WHERE is_enabled = true;
CREATE INDEX idx_actions_order ON actions(tenant_id, trigger_type, execution_order);

CREATE TRIGGER actions_updated_at
    BEFORE UPDATE ON actions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- ============================================================
-- ACTIONS: Execution Logs
-- ============================================================

CREATE TABLE action_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id UUID NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

    trigger_type VARCHAR(50) NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,

    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'failure', 'timeout', 'error')),
    execution_time_ms INTEGER,

    input_data JSONB,
    output_data JSONB,
    error_message TEXT,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_action_executions_action ON action_executions(action_id);
CREATE INDEX idx_action_executions_tenant ON action_executions(tenant_id);
CREATE INDEX idx_action_executions_created ON action_executions(created_at DESC);

-- Update action stats on execution
CREATE OR REPLACE FUNCTION update_action_stats()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE actions SET
        total_executions = total_executions + 1,
        successful_executions = successful_executions + CASE WHEN NEW.status = 'success' THEN 1 ELSE 0 END,
        failed_executions = failed_executions + CASE WHEN NEW.status != 'success' THEN 1 ELSE 0 END,
        avg_execution_time_ms = (avg_execution_time_ms * total_executions + COALESCE(NEW.execution_time_ms, 0)) / (total_executions + 1),
        last_executed_at = NOW(),
        updated_at = NOW()
    WHERE id = NEW.action_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER action_execution_stats_trigger
    AFTER INSERT ON action_executions
    FOR EACH ROW EXECUTE FUNCTION update_action_stats();
