-- Migration 005: Actions/Hooks Extensibility System
-- Add support for JavaScript actions triggered on specific events

-- Create actions table
CREATE TABLE IF NOT EXISTS actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Action metadata
    name TEXT NOT NULL,
    description TEXT,

    -- Trigger point
    trigger_type TEXT NOT NULL CHECK (trigger_type IN (
        'pre_login',
        'post_login',
        'pre_registration',
        'post_registration',
        'pre_token_issue',
        'post_token_issue',
        'pre_user_update',
        'post_user_update',
        'pre_password_reset',
        'post_password_reset'
    )),

    -- Action code (JavaScript)
    code TEXT NOT NULL,

    -- Runtime settings
    runtime TEXT DEFAULT 'nodejs18' NOT NULL,
    timeout_seconds INTEGER DEFAULT 10 NOT NULL CHECK (timeout_seconds > 0 AND timeout_seconds <= 30),

    -- Secrets (encrypted credentials for API calls in JSON format)
    secrets JSONB DEFAULT '{}'::jsonb,

    -- Execution order (multiple actions can run on same trigger)
    execution_order INTEGER DEFAULT 0 NOT NULL,

    -- Status
    is_enabled BOOLEAN DEFAULT true NOT NULL,

    -- Execution stats
    last_executed_at TIMESTAMPTZ,
    total_executions BIGINT DEFAULT 0 NOT NULL,
    total_failures BIGINT DEFAULT 0 NOT NULL,

    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- Unique name per organization
    CONSTRAINT actions_org_name_unique UNIQUE(organization_id, name)
);

-- Indexes for actions
CREATE INDEX IF NOT EXISTS idx_actions_org ON actions(organization_id);
CREATE INDEX IF NOT EXISTS idx_actions_trigger ON actions(trigger_type, is_enabled) WHERE is_enabled = true;
CREATE INDEX IF NOT EXISTS idx_actions_order ON actions(organization_id, trigger_type, execution_order);

-- Create action_executions table (partitioned by month for performance)
CREATE TABLE IF NOT EXISTS action_executions (
    id UUID DEFAULT gen_random_uuid() NOT NULL,
    action_id UUID NOT NULL REFERENCES actions(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Execution context
    trigger_type TEXT NOT NULL,
    user_id UUID,  -- Nullable for non-user-related triggers

    -- Execution details
    status TEXT NOT NULL CHECK (status IN ('success', 'failure', 'timeout')),
    execution_time_ms INTEGER NOT NULL,

    -- Input/output data
    input_data JSONB,
    output_data JSONB,
    error_message TEXT,

    -- Metadata
    executed_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,

    -- Composite primary key including partition key
    PRIMARY KEY (id, executed_at)
) PARTITION BY RANGE (executed_at);

-- Create partitions for current and next 3 months
CREATE TABLE IF NOT EXISTS action_executions_2026_02
PARTITION OF action_executions
FOR VALUES FROM ('2026-02-01') TO ('2026-03-01');

CREATE TABLE IF NOT EXISTS action_executions_2026_03
PARTITION OF action_executions
FOR VALUES FROM ('2026-03-01') TO ('2026-04-01');

CREATE TABLE IF NOT EXISTS action_executions_2026_04
PARTITION OF action_executions
FOR VALUES FROM ('2026-04-01') TO ('2026-05-01');

CREATE TABLE IF NOT EXISTS action_executions_2026_05
PARTITION OF action_executions
FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');

-- Indexes on the partitioned table
CREATE INDEX IF NOT EXISTS idx_action_executions_action ON action_executions(action_id);
CREATE INDEX IF NOT EXISTS idx_action_executions_org ON action_executions(organization_id);
CREATE INDEX IF NOT EXISTS idx_action_executions_status ON action_executions(status);
CREATE INDEX IF NOT EXISTS idx_action_executions_user ON action_executions(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_action_executions_executed_at ON action_executions(executed_at DESC);

-- Function to update action stats
CREATE OR REPLACE FUNCTION update_action_stats()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE actions
    SET
        last_executed_at = NEW.executed_at,
        total_executions = total_executions + 1,
        total_failures = CASE
            WHEN NEW.status = 'failure' OR NEW.status = 'timeout'
            THEN total_failures + 1
            ELSE total_failures
        END
    WHERE id = NEW.action_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update stats after each execution
DROP TRIGGER IF EXISTS action_execution_stats_trigger ON action_executions;
CREATE TRIGGER action_execution_stats_trigger
    AFTER INSERT ON action_executions
    FOR EACH ROW
    EXECUTE FUNCTION update_action_stats();

-- Function to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_actions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger for updated_at
DROP TRIGGER IF EXISTS actions_updated_at_trigger ON actions;
CREATE TRIGGER actions_updated_at_trigger
    BEFORE UPDATE ON actions
    FOR EACH ROW
    EXECUTE FUNCTION update_actions_updated_at();

-- Add comments
COMMENT ON TABLE actions IS 'Extensible JavaScript actions triggered on specific events';
COMMENT ON COLUMN actions.trigger_type IS 'Event that triggers this action execution';
COMMENT ON COLUMN actions.code IS 'JavaScript code to execute (sandboxed)';
COMMENT ON COLUMN actions.secrets IS 'Encrypted credentials accessible in action code';
COMMENT ON COLUMN actions.execution_order IS 'Order of execution when multiple actions share same trigger (lower = earlier)';
COMMENT ON TABLE action_executions IS 'Historical log of action executions (partitioned by month)';
