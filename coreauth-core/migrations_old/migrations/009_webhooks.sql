-- Webhooks System Migration
-- Adds support for real-time event notifications to external systems

-- ============================================================================
-- WEBHOOKS TABLE
-- ============================================================================
-- Stores webhook configurations for organizations

CREATE TABLE IF NOT EXISTS webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Configuration
    name TEXT NOT NULL,
    url TEXT NOT NULL,                    -- https://example.com/webhook
    secret TEXT NOT NULL,                 -- For HMAC signature verification

    -- Event filtering
    events TEXT[] NOT NULL,               -- ['user.created', 'user.login']

    -- Status
    is_enabled BOOLEAN DEFAULT true,

    -- Retry policy (JSON for flexibility)
    retry_policy JSONB DEFAULT '{"max_retries": 3, "initial_delay_ms": 1000, "max_delay_ms": 60000}',

    -- Headers to include in webhook requests
    custom_headers JSONB DEFAULT '{}',

    -- Statistics
    total_deliveries INTEGER DEFAULT 0,
    successful_deliveries INTEGER DEFAULT 0,
    failed_deliveries INTEGER DEFAULT 0,
    last_triggered_at TIMESTAMPTZ,
    last_success_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_webhooks_organization_id ON webhooks(organization_id);
CREATE INDEX IF NOT EXISTS idx_webhooks_enabled ON webhooks(is_enabled) WHERE is_enabled = true;

-- ============================================================================
-- WEBHOOK DELIVERIES TABLE
-- ============================================================================
-- Tracks individual webhook delivery attempts

CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    webhook_id UUID NOT NULL REFERENCES webhooks(id) ON DELETE CASCADE,

    -- Event details
    event_id TEXT NOT NULL,               -- Unique event identifier (evt_xxx)
    event_type TEXT NOT NULL,             -- 'user.created', 'user.login', etc.
    payload JSONB NOT NULL,               -- Full event payload

    -- Delivery status
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'success', 'failed', 'retrying'

    -- Request details
    request_headers JSONB,
    request_body TEXT,

    -- Response details
    response_status INTEGER,
    response_headers JSONB,
    response_body TEXT,
    response_time_ms INTEGER,

    -- Retry tracking
    attempt_count INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 4,        -- 1 initial + 3 retries
    next_retry_at TIMESTAMPTZ,
    last_error TEXT,

    -- Timestamps
    delivered_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_webhook_id ON webhook_deliveries(webhook_id);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_event_type ON webhook_deliveries(event_type);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_status ON webhook_deliveries(status);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_next_retry ON webhook_deliveries(next_retry_at)
    WHERE status = 'retrying' AND next_retry_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_created_at ON webhook_deliveries(created_at);

-- ============================================================================
-- WEBHOOK EVENTS TABLE
-- ============================================================================
-- Stores events before they are dispatched to webhooks (for reliability)

CREATE TABLE IF NOT EXISTS webhook_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id TEXT UNIQUE NOT NULL,        -- Public event ID (evt_xxx)
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Event details
    event_type TEXT NOT NULL,
    payload JSONB NOT NULL,

    -- Processing status
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'processing', 'completed', 'failed'
    processed_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_webhook_events_organization_id ON webhook_events(organization_id);
CREATE INDEX IF NOT EXISTS idx_webhook_events_status ON webhook_events(status);
CREATE INDEX IF NOT EXISTS idx_webhook_events_created_at ON webhook_events(created_at);

-- ============================================================================
-- SUPPORTED WEBHOOK EVENTS
-- ============================================================================
-- Reference table for available webhook event types

CREATE TABLE IF NOT EXISTS webhook_event_types (
    id TEXT PRIMARY KEY,                   -- 'user.created'
    category TEXT NOT NULL,                -- 'user', 'organization', 'application'
    description TEXT NOT NULL,
    payload_schema JSONB,                  -- JSON Schema for payload
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Insert supported event types
INSERT INTO webhook_event_types (id, category, description) VALUES
    -- User events
    ('user.created', 'user', 'A new user was created'),
    ('user.updated', 'user', 'User profile was updated'),
    ('user.deleted', 'user', 'User was deleted'),
    ('user.login', 'user', 'User logged in successfully'),
    ('user.login_failed', 'user', 'User login attempt failed'),
    ('user.logout', 'user', 'User logged out'),
    ('user.password_changed', 'user', 'User password was changed'),
    ('user.password_reset_requested', 'user', 'Password reset was requested'),
    ('user.email_verified', 'user', 'User email was verified'),
    ('user.mfa_enrolled', 'user', 'User enrolled in MFA'),
    ('user.mfa_disabled', 'user', 'User disabled MFA'),
    ('user.blocked', 'user', 'User was blocked'),
    ('user.unblocked', 'user', 'User was unblocked'),

    -- Organization events
    ('organization.created', 'organization', 'A new organization was created'),
    ('organization.updated', 'organization', 'Organization settings were updated'),
    ('organization.deleted', 'organization', 'Organization was deleted'),
    ('organization.member_added', 'organization', 'A member was added to the organization'),
    ('organization.member_removed', 'organization', 'A member was removed from the organization'),
    ('organization.member_role_changed', 'organization', 'A member role was changed'),

    -- Application events
    ('application.created', 'application', 'A new application was registered'),
    ('application.updated', 'application', 'Application settings were updated'),
    ('application.deleted', 'application', 'Application was deleted'),
    ('application.secret_rotated', 'application', 'Application secret was rotated'),

    -- Connection events
    ('connection.created', 'connection', 'A new connection was created'),
    ('connection.updated', 'connection', 'Connection settings were updated'),
    ('connection.deleted', 'connection', 'Connection was deleted'),

    -- Session events
    ('session.created', 'session', 'A new session was created'),
    ('session.revoked', 'session', 'A session was revoked')
ON CONFLICT (id) DO UPDATE SET
    category = EXCLUDED.category,
    description = EXCLUDED.description;

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to update webhook statistics
CREATE OR REPLACE FUNCTION update_webhook_stats()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'success' AND (OLD.status IS NULL OR OLD.status != 'success') THEN
        UPDATE webhooks
        SET
            total_deliveries = total_deliveries + 1,
            successful_deliveries = successful_deliveries + 1,
            last_triggered_at = NOW(),
            last_success_at = NOW(),
            updated_at = NOW()
        WHERE id = NEW.webhook_id;
    ELSIF NEW.status = 'failed' AND (OLD.status IS NULL OR OLD.status != 'failed') THEN
        UPDATE webhooks
        SET
            total_deliveries = total_deliveries + 1,
            failed_deliveries = failed_deliveries + 1,
            last_triggered_at = NOW(),
            last_failure_at = NOW(),
            updated_at = NOW()
        WHERE id = NEW.webhook_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for webhook statistics
DROP TRIGGER IF EXISTS webhook_delivery_stats_trigger ON webhook_deliveries;
CREATE TRIGGER webhook_delivery_stats_trigger
    AFTER INSERT OR UPDATE OF status ON webhook_deliveries
    FOR EACH ROW
    EXECUTE FUNCTION update_webhook_stats();

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE webhooks IS 'Webhook configurations for real-time event notifications';
COMMENT ON TABLE webhook_deliveries IS 'Individual webhook delivery attempts and their results';
COMMENT ON TABLE webhook_events IS 'Event queue for reliable webhook delivery';
COMMENT ON TABLE webhook_event_types IS 'Supported webhook event types reference';

COMMENT ON COLUMN webhooks.secret IS 'Secret key for HMAC-SHA256 signature verification';
COMMENT ON COLUMN webhooks.events IS 'Array of event types this webhook subscribes to';
COMMENT ON COLUMN webhooks.retry_policy IS 'JSON config: max_retries, initial_delay_ms, max_delay_ms';
COMMENT ON COLUMN webhook_deliveries.event_id IS 'Public event identifier (evt_xxx format)';
COMMENT ON COLUMN webhook_deliveries.next_retry_at IS 'When to retry failed delivery (exponential backoff)';
