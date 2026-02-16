-- ============================================================
-- CoreAuth CIAM - OAuth2/OIDC Authorization Server
-- ============================================================
-- Authorization codes, tokens, JWKS, consents
-- ============================================================

-- ============================================================
-- OAuth Authorization Codes (PKCE Support)
-- ============================================================

CREATE TABLE oauth_authorization_codes (
    code VARCHAR(255) PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    redirect_uri TEXT NOT NULL,
    scope TEXT,
    audience TEXT,
    response_type VARCHAR(50) DEFAULT 'code',

    -- PKCE
    code_challenge TEXT,
    code_challenge_method VARCHAR(10) CHECK (code_challenge_method IN ('S256', 'plain')),

    -- OIDC
    nonce TEXT,
    state TEXT,

    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_oauth_codes_client ON oauth_authorization_codes(client_id);
CREATE INDEX idx_oauth_codes_user ON oauth_authorization_codes(user_id);
CREATE INDEX idx_oauth_codes_expires ON oauth_authorization_codes(expires_at);

-- ============================================================
-- OAuth Refresh Tokens
-- ============================================================

CREATE TABLE oauth_refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash VARCHAR(255) UNIQUE NOT NULL,

    client_id VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    scope TEXT,
    audience TEXT,

    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_oauth_refresh_client ON oauth_refresh_tokens(client_id);
CREATE INDEX idx_oauth_refresh_user ON oauth_refresh_tokens(user_id);
CREATE INDEX idx_oauth_refresh_expires ON oauth_refresh_tokens(expires_at);

-- ============================================================
-- OAuth Access Tokens (Audit/Revocation)
-- ============================================================

CREATE TABLE oauth_access_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    jti VARCHAR(255) UNIQUE NOT NULL,

    client_id VARCHAR(255) NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    scope TEXT,
    audience TEXT,

    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_oauth_access_jti ON oauth_access_tokens(jti);
CREATE INDEX idx_oauth_access_user ON oauth_access_tokens(user_id);
CREATE INDEX idx_oauth_access_expires ON oauth_access_tokens(expires_at);

-- ============================================================
-- OAuth Consents (User Grants)
-- ============================================================

CREATE TABLE oauth_consents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id VARCHAR(255) NOT NULL,
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,

    scopes TEXT[] NOT NULL,

    granted_at TIMESTAMPTZ DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,

    UNIQUE(user_id, client_id, tenant_id)
);

CREATE INDEX idx_oauth_consents_user ON oauth_consents(user_id);
CREATE INDEX idx_oauth_consents_client ON oauth_consents(client_id);

-- ============================================================
-- OAuth Authorization Requests (In-Progress)
-- ============================================================

CREATE TABLE oauth_authorization_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id VARCHAR(255) UNIQUE NOT NULL,

    client_id VARCHAR(255) NOT NULL,
    redirect_uri TEXT NOT NULL,
    response_type VARCHAR(50) NOT NULL,
    scope TEXT,
    state TEXT,

    -- PKCE
    code_challenge TEXT,
    code_challenge_method VARCHAR(10),

    -- OIDC
    nonce TEXT,

    -- Context
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    connection_hint VARCHAR(255),
    login_hint VARCHAR(255),
    prompt VARCHAR(50),
    max_age INTEGER,
    ui_locales TEXT,

    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_oauth_auth_requests_expires ON oauth_authorization_requests(expires_at);

-- ============================================================
-- JWT Signing Keys
-- ============================================================

CREATE TABLE signing_keys (
    id VARCHAR(255) PRIMARY KEY,
    algorithm VARCHAR(10) NOT NULL DEFAULT 'RS256',

    public_key_pem TEXT NOT NULL,
    private_key_pem TEXT NOT NULL,

    is_current BOOLEAN DEFAULT false,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    rotated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_signing_keys_current ON signing_keys(is_current) WHERE is_current = true;

-- Insert default development signing key (valid RSA 2048-bit key pair)
INSERT INTO signing_keys (id, algorithm, public_key_pem, private_key_pem, is_current)
VALUES (
    'dev-key-001',
    'RS256',
    '-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA1xaI7PEKEp7gHQubq9SE
9GmsIAhym38oiI10dvGlrifpv13T2xMuUZFMIdIa1ed1KeR05JuAnRCOPoU7ADQ9
sn2QvtLy206Ji1xCRyjuP6TS8hZPOG7XMQ4J1iIknThuLRgQlZaSmy/7/L40eT+x
jjr8jjLAMKLCfi0EJXo/imBnpCMT0su8iUi2ztN1uMGS7BLKB73jHaPAKxWHv0e5
dPmQEmYxLKSfEkDqb7BqskGStv2DL0+YDJY8016OLkMmkMZOOpztiTBnKKpH0m9/
A2sO6PJYNdFzS8VzPixYqQDRia3UH81c9Yp0TsU4rLNnB1Plns1E7GoiwCD7zFuZ
ZQIDAQAB
-----END PUBLIC KEY-----',
    '-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDXFojs8QoSnuAd
C5ur1IT0aawgCHKbfyiIjXR28aWuJ+m/XdPbEy5RkUwh0hrV53Up5HTkm4CdEI4+
hTsAND2yfZC+0vLbTomLXEJHKO4/pNLyFk84btcxDgnWIiSdOG4tGBCVlpKbL/v8
vjR5P7GOOvyOMsAwosJ+LQQlej+KYGekIxPSy7yJSLbO03W4wZLsEsoHveMdo8Ar
FYe/R7l0+ZASZjEspJ8SQOpvsGqyQZK2/YMvT5gMljzTXo4uQyaQxk46nO2JMGco
qkfSb38Daw7o8lg10XNLxXM+LFipANGJrdQfzVz1inROxTiss2cHU+WezUTsaiLA
IPvMW5llAgMBAAECggEAMlc7IW+x6pVGQW4N14v0OUtBRrcLceLzTvCCnXMl0qrT
Td+NPfVRjjQ6VCEbeEyFstIeXMIbekddb3sZycywUwYJ+muffR422Yf0mDiXeIxg
ddosXPouQfQ176kKlkSWntXGzegRtKIosWkoGWfDrBUYtsRgkJGLgWISWh4wYHeN
M2uIpJX9ZZe3DDNEz7SgLb+uydDa5ebTAqrgmh2mkoy0eAkXE8qL0j+tqRsEIfWn
vh83f96Ohq4tC1Ns2YyiSp83tuYzsjHfX0SpIMNDxeygPBduzzP7CQ+pP0M4OXoX
aW4kqeQ+n6qykhBxN9xkp4eUzs1zUQnEYaeslIB8UQKBgQD7+k6CalIA1Rh29+6I
Kon4TyS67wpbh0icLaWCE5OsR2JwTczMXuhy0swHscKj5rI/SMLQKbNI85H9sTIt
SyYPWVGYVZk+wEJueOt/gHamkE2jvr3s16w3w+ma7G1R9ra9jDvs5FUs7bhIgk7r
Lv93bpwqaDsb6PFHr2uStMcbsQKBgQDahXr1q3ltzZR/i69XU8mUI7TIhQtsNBh1
SCFayvJOKAP3TcaGcI3YKPNQ+RdPJ7DcT6iUFgUUpoE6cIDIsqe7IXvStjBBXUie
5+F1CzSBsXLb8kmADn4vWcRIQkAUdUIMSY7CHi0jQHkhpq9Eul9ZCh/1FjrwA5mc
V6BsY7rp9QKBgGi2rZeu3WMxK1iNUhhOLUX+hdIVcqV0w+z7XzN+NTk79SGcg0ZP
DqRhC58K9UstnNeFwkfFfJcNeZcG40ZFW4y3XpxCvkuAlFrab2tuFGDFyS4KH11k
h1IpXVQfepK8R1bgBys7/FWOeK3RUUCVKF8WnVlWNXI0zMgwkzDFZZURAoGAVpox
6e0EWczwvj+oxO4y5F/mRNXdeguHaeulNGtb7jeTos4TELLItFM+YuflIfyz7gwv
3kh/yPhYHMX7dA7BxLr3bMHBjBYhMsDpLE69h2zA/YQfNv4HalKkJJME/FagT2hT
iIEXGHJzzy9VJOLL6OjHU4V05Cw5E6nyrSI1MIUCgYEAuWBjMDlAqnvyATVyQnJJ
DjkZ4To+vcN8I4NqUoCYs7N5NphIlMv5OSZ/XEoJJVnLrWGTpTSIzMxCELTnzcXQ
lmeu+QPQ4vXgrQbAEaeWw9/W1FDaq1Piaf3U3CrJuX5KsymaVugg53xIHG7xIO/y
OTMlvG8sAD+Z5oDxPgogpso=
-----END PRIVATE KEY-----',
    true
) ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- Cleanup Functions
-- ============================================================

CREATE OR REPLACE FUNCTION cleanup_expired_oauth_codes()
RETURNS void AS $$
BEGIN
    DELETE FROM oauth_authorization_codes
    WHERE expires_at < NOW() OR used_at IS NOT NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION cleanup_expired_auth_requests()
RETURNS void AS $$
BEGIN
    DELETE FROM oauth_authorization_requests
    WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION cleanup_expired_refresh_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM oauth_refresh_tokens
    WHERE (expires_at IS NOT NULL AND expires_at < NOW())
       OR revoked_at IS NOT NULL;
END;
$$ LANGUAGE plpgsql;
