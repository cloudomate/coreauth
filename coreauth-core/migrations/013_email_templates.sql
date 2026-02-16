-- ============================================================
-- CoreAuth CIAM - Customizable Email Templates
-- ============================================================
-- Per-tenant email template overrides. When a custom template
-- exists for a tenant+type, it's used instead of the built-in
-- default. Simple {{variable}} substitution is supported.

CREATE TABLE IF NOT EXISTS email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    template_type VARCHAR(50) NOT NULL,
    subject TEXT NOT NULL,
    html_body TEXT NOT NULL,
    text_body TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_email_templates_tenant_type UNIQUE (tenant_id, template_type),
    CONSTRAINT chk_template_type CHECK (template_type IN (
        'email_verification',
        'password_reset',
        'user_invitation',
        'magic_link',
        'account_locked',
        'mfa_enforcement'
    ))
);

CREATE INDEX idx_email_templates_tenant_id ON email_templates(tenant_id);
