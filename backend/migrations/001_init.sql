-- ============================================================
-- CIAM Hierarchical Authentication System - Initial Schema
-- ============================================================
-- 
-- This migration creates the complete schema for the CIAM system
-- with hierarchical organization support (Auth0-style model).
--
-- Key Concepts:
--   - Platform: Root level (single instance for on-prem deployment)
--   - Organizations: Customer workspaces (formerly "tenants")
--   - Users: Global user pool that can belong to multiple organizations
--   - Organization Members: Junction table for user-org relationships
--   - Connections: SSO/OIDC identity providers per organization
--   - Applications: OAuth clients for authentication
--
-- ============================================================


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: application_type; Type: TYPE; Schema: public; Owner: -
--

DO $$ BEGIN
    CREATE TYPE public.application_type AS ENUM (
        'service',
        'webapp',
        'spa',
        'native'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;


--
-- Name: audit_event_category; Type: TYPE; Schema: public; Owner: -
--

DO $$ BEGIN
    CREATE TYPE public.audit_event_category AS ENUM (
        'authentication',
        'authorization',
        'user_management',
        'tenant_management',
        'security',
        'admin',
        'system'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;


--
-- Name: subject_type; Type: TYPE; Schema: public; Owner: -
--

DO $$ BEGIN
    CREATE TYPE public.subject_type AS ENUM (
        'user',
        'application',
        'group',
        'userset'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;


--
-- Name: update_updated_at(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE OR REPLACE FUNCTION public.update_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$;


--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: public; Owner: -
--

CREATE OR REPLACE FUNCTION public.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: account_lockouts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.account_lockouts (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid,
    tenant_id uuid,  -- Nullable for platform admin lockouts without organization context
    locked_until timestamp with time zone NOT NULL,
    reason character varying(255) NOT NULL,
    locked_by uuid,
    unlock_token character varying(255),
    unlocked_at timestamp with time zone,
    unlocked_by uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: applications; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.applications (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    slug text NOT NULL,
    description text,
    type text NOT NULL,
    client_id text NOT NULL,
    client_secret text,
    callback_urls jsonb DEFAULT '[]'::jsonb,
    logout_urls jsonb DEFAULT '[]'::jsonb,
    web_origins jsonb DEFAULT '[]'::jsonb,
    allowed_connections jsonb DEFAULT '[]'::jsonb,
    require_organization boolean DEFAULT false NOT NULL,
    platform_admin_only boolean DEFAULT false NOT NULL,
    access_token_lifetime_seconds integer DEFAULT 3600 NOT NULL,
    refresh_token_lifetime_seconds integer DEFAULT 2592000 NOT NULL,
    is_enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT applications_type_check CHECK ((type = ANY (ARRAY['web'::text, 'spa'::text, 'native'::text, 'api'::text])))
);


--
-- Name: audit_logs; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.audit_logs (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    event_type character varying(100) NOT NULL,
    event_category public.audit_event_category NOT NULL,
    event_action character varying(50) NOT NULL,
    actor_type character varying(50),
    actor_id character varying(255),
    actor_name character varying(255),
    actor_ip_address inet,
    actor_user_agent text,
    target_type character varying(50),
    target_id character varying(255),
    target_name character varying(255),
    description text,
    metadata jsonb DEFAULT '{}'::jsonb,
    status character varying(20) NOT NULL,
    error_message text,
    request_id character varying(255),
    session_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    organization_id uuid
)
PARTITION BY RANGE (created_at);


--
-- Name: audit_logs_2026_02; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.audit_logs_2026_02 (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    event_type character varying(100) NOT NULL,
    event_category public.audit_event_category NOT NULL,
    event_action character varying(50) NOT NULL,
    actor_type character varying(50),
    actor_id character varying(255),
    actor_name character varying(255),
    actor_ip_address inet,
    actor_user_agent text,
    target_type character varying(50),
    target_id character varying(255),
    target_name character varying(255),
    description text,
    metadata jsonb DEFAULT '{}'::jsonb,
    status character varying(20) NOT NULL,
    error_message text,
    request_id character varying(255),
    session_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    organization_id uuid
);


--
-- Name: audit_logs_2026_03; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.audit_logs_2026_03 (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    event_type character varying(100) NOT NULL,
    event_category public.audit_event_category NOT NULL,
    event_action character varying(50) NOT NULL,
    actor_type character varying(50),
    actor_id character varying(255),
    actor_name character varying(255),
    actor_ip_address inet,
    actor_user_agent text,
    target_type character varying(50),
    target_id character varying(255),
    target_name character varying(255),
    description text,
    metadata jsonb DEFAULT '{}'::jsonb,
    status character varying(20) NOT NULL,
    error_message text,
    request_id character varying(255),
    session_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    organization_id uuid
);


--
-- Name: audit_logs_2026_04; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.audit_logs_2026_04 (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    event_type character varying(100) NOT NULL,
    event_category public.audit_event_category NOT NULL,
    event_action character varying(50) NOT NULL,
    actor_type character varying(50),
    actor_id character varying(255),
    actor_name character varying(255),
    actor_ip_address inet,
    actor_user_agent text,
    target_type character varying(50),
    target_id character varying(255),
    target_name character varying(255),
    description text,
    metadata jsonb DEFAULT '{}'::jsonb,
    status character varying(20) NOT NULL,
    error_message text,
    request_id character varying(255),
    session_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    organization_id uuid
);


--
-- Name: blocked_email_domains; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.blocked_email_domains (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid,
    domain character varying(255) NOT NULL,
    reason text,
    added_by uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: connections; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.connections (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    type text NOT NULL,
    scope text NOT NULL,
    organization_id uuid,
    config jsonb DEFAULT '{}'::jsonb NOT NULL,
    is_enabled boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT connections_scope_check CHECK ((scope = ANY (ARRAY['platform'::text, 'organization'::text]))),
    CONSTRAINT connections_scope_org_check CHECK ((((scope = 'platform'::text) AND (organization_id IS NULL)) OR ((scope = 'organization'::text) AND (organization_id IS NOT NULL)))),
    CONSTRAINT connections_type_check CHECK ((type = ANY (ARRAY['database'::text, 'oidc'::text, 'saml'::text, 'oauth2'::text])))
);


--
-- Name: email_verification_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.email_verification_tokens (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    tenant_id uuid NOT NULL,
    email character varying(255) NOT NULL,
    token_hash character varying(255) NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: invitations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.invitations (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    email character varying(255) NOT NULL,
    token_hash character varying(255) NOT NULL,
    invited_by uuid NOT NULL,
    role_id uuid,
    metadata jsonb,
    accepted_at timestamp with time zone,
    accepted_by uuid,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: login_attempts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.login_attempts (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid,
    tenant_id uuid,  -- Nullable for platform admin logins without organization context
    email character varying(255) NOT NULL,
    ip_address text NOT NULL,
    successful boolean NOT NULL,
    failure_reason character varying(100),
    attempted_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    user_agent text
);


--
-- Name: magic_link_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.magic_link_tokens (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid,
    tenant_id uuid NOT NULL,
    email character varying(255) NOT NULL,
    token_hash character varying(255) NOT NULL,
    device_fingerprint text,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone,
    ip_address text,
    user_agent text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: mfa_backup_codes; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.mfa_backup_codes (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    code_hash character varying(255) NOT NULL,
    used_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: mfa_challenges; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.mfa_challenges (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    challenge_token character varying(255) NOT NULL,
    method_id uuid,
    code_hash character varying(255),
    verified boolean DEFAULT false,
    ip_address text,
    user_agent text,
    expires_at timestamp with time zone NOT NULL,
    verified_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: mfa_methods; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.mfa_methods (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    method_type character varying(20) NOT NULL,
    secret text,
    verified boolean DEFAULT false,
    name character varying(100),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    last_used_at timestamp with time zone,
    CONSTRAINT mfa_methods_method_type_check CHECK (((method_type)::text = ANY (ARRAY[('totp'::character varying)::text, ('sms'::character varying)::text, ('email'::character varying)::text, ('webauthn'::character varying)::text])))
);


--
-- Name: oauth_clients; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.oauth_clients (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    client_id character varying(255) NOT NULL,
    client_secret_hash text NOT NULL,
    name character varying(255) NOT NULL,
    redirect_uris text[] NOT NULL,
    grant_types character varying(50)[] DEFAULT ARRAY['authorization_code'::text],
    scopes character varying(50)[] DEFAULT ARRAY['openid'::text, 'profile'::text, 'email'::text],
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: oidc_providers; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.oidc_providers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    provider_type character varying(50) NOT NULL,
    issuer text NOT NULL,
    client_id text NOT NULL,
    client_secret text NOT NULL,
    authorization_endpoint text NOT NULL,
    token_endpoint text NOT NULL,
    userinfo_endpoint text,
    jwks_uri text NOT NULL,
    scopes character varying(100)[] DEFAULT ARRAY['openid'::text, 'profile'::text, 'email'::text],
    claim_mappings jsonb DEFAULT '{"email": "email", "phone": "phone_number", "last_name": "family_name", "first_name": "given_name"}'::jsonb NOT NULL,
    is_active boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: organization_members; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.organization_members (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    organization_id uuid NOT NULL,
    role text DEFAULT 'member'::text NOT NULL,
    joined_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: organizations; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.organizations (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    slug character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    isolation_mode character varying(20) DEFAULT 'pool'::character varying,
    custom_domain character varying(255),
    settings jsonb DEFAULT '{}'::jsonb,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT tenants_isolation_mode_check CHECK (((isolation_mode)::text = ANY (ARRAY[('pool'::character varying)::text, ('silo'::character varying)::text])))
);


--
-- Name: password_reset_tokens; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.password_reset_tokens (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    tenant_id uuid NOT NULL,
    token_hash character varying(255) NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    used_at timestamp with time zone,
    ip_address text,
    user_agent text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.permissions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    resource character varying(100) NOT NULL,
    action character varying(100) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: platform_config; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.platform_config (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name text NOT NULL,
    slug text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


--
-- Name: relation_tuples; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.relation_tuples (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    tenant_id uuid NOT NULL,
    namespace character varying(100) NOT NULL,
    object_id character varying(255) NOT NULL,
    relation character varying(100) NOT NULL,
    subject_type public.subject_type NOT NULL,
    subject_id character varying(255) NOT NULL,
    subject_relation character varying(100),
    created_at timestamp with time zone DEFAULT now(),
    organization_id uuid
);


--
-- Name: role_permissions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.role_permissions (
    role_id uuid NOT NULL,
    permission_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.roles (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    parent_role_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: sessions; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.sessions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    token_hash text NOT NULL,
    refresh_token_hash text,
    device_fingerprint text,
    ip_address text,  -- Changed from inet to text for compatibility with code
    user_agent text,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


--
-- Name: tenants; Type: VIEW; Schema: public; Owner: -
--

CREATE OR REPLACE VIEW public.tenants AS
 SELECT id,
    slug,
    name,
    isolation_mode,
    custom_domain,
    settings,
    created_at,
    updated_at
   FROM public.organizations;


--
-- Name: user_bans; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.user_bans (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    tenant_id uuid NOT NULL,
    user_id uuid,
    ip_address text,
    email character varying(255),
    reason text NOT NULL,
    banned_by uuid NOT NULL,
    unbanned_at timestamp with time zone,
    unbanned_by uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT user_bans_identifier_check CHECK (((user_id IS NOT NULL) OR (ip_address IS NOT NULL) OR (email IS NOT NULL)))
);


--
-- Name: user_roles; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.user_roles (
    user_id uuid NOT NULL,
    role_id uuid NOT NULL,
    granted_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    granted_by uuid
);


--
-- Name: users; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE IF NOT EXISTS public.users (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    default_organization_id uuid,
    email character varying(255) NOT NULL,
    email_verified boolean DEFAULT false,
    phone character varying(50),
    phone_verified boolean DEFAULT false,
    password_hash text,
    metadata jsonb DEFAULT '{}'::jsonb,
    is_active boolean DEFAULT true,
    last_login_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    provider_user_id text,
    provider_id uuid,
    mfa_enabled boolean DEFAULT false,
    mfa_enforced_at timestamp with time zone,
    is_platform_admin boolean DEFAULT false NOT NULL
);


--
-- Name: audit_logs_2026_02; Type: TABLE ATTACH; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_inherits
        WHERE inhrelid = 'public.audit_logs_2026_02'::regclass
        AND inhparent = 'public.audit_logs'::regclass
    ) THEN
        ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_2026_02 FOR VALUES FROM ('2026-02-01 00:00:00+00') TO ('2026-03-01 00:00:00+00');
    END IF;
END $$;


--
-- Name: audit_logs_2026_03; Type: TABLE ATTACH; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_inherits
        WHERE inhrelid = 'public.audit_logs_2026_03'::regclass
        AND inhparent = 'public.audit_logs'::regclass
    ) THEN
        ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_2026_03 FOR VALUES FROM ('2026-03-01 00:00:00+00') TO ('2026-04-01 00:00:00+00');
    END IF;
END $$;


--
-- Name: audit_logs_2026_04; Type: TABLE ATTACH; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_inherits
        WHERE inhrelid = 'public.audit_logs_2026_04'::regclass
        AND inhparent = 'public.audit_logs'::regclass
    ) THEN
        ALTER TABLE ONLY public.audit_logs ATTACH PARTITION public.audit_logs_2026_04 FOR VALUES FROM ('2026-04-01 00:00:00+00') TO ('2026-05-01 00:00:00+00');
    END IF;
END $$;


--
-- Name: account_lockouts account_lockouts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'account_lockouts_pkey'
        AND conrelid = 'public.account_lockouts'::regclass
    ) THEN
        ALTER TABLE ONLY public.account_lockouts
            ADD CONSTRAINT account_lockouts_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: account_lockouts account_lockouts_unlock_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'account_lockouts_unlock_token_key'
        AND conrelid = 'public.account_lockouts'::regclass
    ) THEN
        ALTER TABLE ONLY public.account_lockouts
            ADD CONSTRAINT account_lockouts_unlock_token_key UNIQUE (unlock_token);
    END IF;
END $$;


--
-- Name: applications applications_client_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'applications_client_id_key'
        AND conrelid = 'public.applications'::regclass
    ) THEN
        ALTER TABLE ONLY public.applications
            ADD CONSTRAINT applications_client_id_key UNIQUE (client_id);
    END IF;
END $$;


--
-- Name: applications applications_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'applications_pkey'
        AND conrelid = 'public.applications'::regclass
    ) THEN
        ALTER TABLE ONLY public.applications
            ADD CONSTRAINT applications_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: applications applications_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'applications_slug_key'
        AND conrelid = 'public.applications'::regclass
    ) THEN
        ALTER TABLE ONLY public.applications
            ADD CONSTRAINT applications_slug_key UNIQUE (slug);
    END IF;
END $$;


--
-- Name: audit_logs audit_logs_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'audit_logs_pkey'
        AND conrelid = 'public.audit_logs'::regclass
    ) THEN
        ALTER TABLE ONLY public.audit_logs
            ADD CONSTRAINT audit_logs_pkey PRIMARY KEY (created_at, id);
    END IF;
END $$;


--
-- Name: audit_logs_2026_02 audit_logs_2026_02_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'audit_logs_2026_02_pkey'
        AND conrelid = 'public.audit_logs_2026_02'::regclass
    ) THEN
        ALTER TABLE ONLY public.audit_logs_2026_02
            ADD CONSTRAINT audit_logs_2026_02_pkey PRIMARY KEY (created_at, id);
    END IF;
END $$;


--
-- Name: audit_logs_2026_03 audit_logs_2026_03_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'audit_logs_2026_03_pkey'
        AND conrelid = 'public.audit_logs_2026_03'::regclass
    ) THEN
        ALTER TABLE ONLY public.audit_logs_2026_03
            ADD CONSTRAINT audit_logs_2026_03_pkey PRIMARY KEY (created_at, id);
    END IF;
END $$;


--
-- Name: audit_logs_2026_04 audit_logs_2026_04_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'audit_logs_2026_04_pkey'
        AND conrelid = 'public.audit_logs_2026_04'::regclass
    ) THEN
        ALTER TABLE ONLY public.audit_logs_2026_04
            ADD CONSTRAINT audit_logs_2026_04_pkey PRIMARY KEY (created_at, id);
    END IF;
END $$;


--
-- Name: blocked_email_domains blocked_email_domains_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'blocked_email_domains_pkey'
        AND conrelid = 'public.blocked_email_domains'::regclass
    ) THEN
        ALTER TABLE ONLY public.blocked_email_domains
            ADD CONSTRAINT blocked_email_domains_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: blocked_email_domains blocked_email_domains_tenant_id_domain_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'blocked_email_domains_tenant_id_domain_key'
        AND conrelid = 'public.blocked_email_domains'::regclass
    ) THEN
        ALTER TABLE ONLY public.blocked_email_domains
            ADD CONSTRAINT blocked_email_domains_tenant_id_domain_key UNIQUE (tenant_id, domain);
    END IF;
END $$;


--
-- Name: connections connections_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'connections_pkey'
        AND conrelid = 'public.connections'::regclass
    ) THEN
        ALTER TABLE ONLY public.connections
            ADD CONSTRAINT connections_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: email_verification_tokens email_verification_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'email_verification_tokens_pkey'
        AND conrelid = 'public.email_verification_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.email_verification_tokens
            ADD CONSTRAINT email_verification_tokens_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: email_verification_tokens email_verification_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'email_verification_tokens_token_hash_key'
        AND conrelid = 'public.email_verification_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.email_verification_tokens
            ADD CONSTRAINT email_verification_tokens_token_hash_key UNIQUE (token_hash);
    END IF;
END $$;


--
-- Name: invitations invitations_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'invitations_pkey'
        AND conrelid = 'public.invitations'::regclass
    ) THEN
        ALTER TABLE ONLY public.invitations
            ADD CONSTRAINT invitations_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: invitations invitations_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'invitations_token_hash_key'
        AND conrelid = 'public.invitations'::regclass
    ) THEN
        ALTER TABLE ONLY public.invitations
            ADD CONSTRAINT invitations_token_hash_key UNIQUE (token_hash);
    END IF;
END $$;


--
-- Name: login_attempts login_attempts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'login_attempts_pkey'
        AND conrelid = 'public.login_attempts'::regclass
    ) THEN
        ALTER TABLE ONLY public.login_attempts
            ADD CONSTRAINT login_attempts_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: magic_link_tokens magic_link_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'magic_link_tokens_pkey'
        AND conrelid = 'public.magic_link_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.magic_link_tokens
            ADD CONSTRAINT magic_link_tokens_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: magic_link_tokens magic_link_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'magic_link_tokens_token_hash_key'
        AND conrelid = 'public.magic_link_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.magic_link_tokens
            ADD CONSTRAINT magic_link_tokens_token_hash_key UNIQUE (token_hash);
    END IF;
END $$;


--
-- Name: mfa_backup_codes mfa_backup_codes_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'mfa_backup_codes_pkey'
        AND conrelid = 'public.mfa_backup_codes'::regclass
    ) THEN
        ALTER TABLE ONLY public.mfa_backup_codes
            ADD CONSTRAINT mfa_backup_codes_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: mfa_challenges mfa_challenges_challenge_token_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'mfa_challenges_challenge_token_key'
        AND conrelid = 'public.mfa_challenges'::regclass
    ) THEN
        ALTER TABLE ONLY public.mfa_challenges
            ADD CONSTRAINT mfa_challenges_challenge_token_key UNIQUE (challenge_token);
    END IF;
END $$;


--
-- Name: mfa_challenges mfa_challenges_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'mfa_challenges_pkey'
        AND conrelid = 'public.mfa_challenges'::regclass
    ) THEN
        ALTER TABLE ONLY public.mfa_challenges
            ADD CONSTRAINT mfa_challenges_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: mfa_methods mfa_methods_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'mfa_methods_pkey'
        AND conrelid = 'public.mfa_methods'::regclass
    ) THEN
        ALTER TABLE ONLY public.mfa_methods
            ADD CONSTRAINT mfa_methods_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: oauth_clients oauth_clients_client_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'oauth_clients_client_id_key'
        AND conrelid = 'public.oauth_clients'::regclass
    ) THEN
        ALTER TABLE ONLY public.oauth_clients
            ADD CONSTRAINT oauth_clients_client_id_key UNIQUE (client_id);
    END IF;
END $$;


--
-- Name: oauth_clients oauth_clients_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'oauth_clients_pkey'
        AND conrelid = 'public.oauth_clients'::regclass
    ) THEN
        ALTER TABLE ONLY public.oauth_clients
            ADD CONSTRAINT oauth_clients_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: oidc_providers oidc_providers_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'oidc_providers_pkey'
        AND conrelid = 'public.oidc_providers'::regclass
    ) THEN
        ALTER TABLE ONLY public.oidc_providers
            ADD CONSTRAINT oidc_providers_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: organization_members organization_members_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'organization_members_pkey'
        AND conrelid = 'public.organization_members'::regclass
    ) THEN
        ALTER TABLE ONLY public.organization_members
            ADD CONSTRAINT organization_members_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: organization_members organization_members_user_id_organization_id_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'organization_members_user_id_organization_id_key'
        AND conrelid = 'public.organization_members'::regclass
    ) THEN
        ALTER TABLE ONLY public.organization_members
            ADD CONSTRAINT organization_members_user_id_organization_id_key UNIQUE (user_id, organization_id);
    END IF;
END $$;


--
-- Name: password_reset_tokens password_reset_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'password_reset_tokens_pkey'
        AND conrelid = 'public.password_reset_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.password_reset_tokens
            ADD CONSTRAINT password_reset_tokens_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: password_reset_tokens password_reset_tokens_token_hash_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'password_reset_tokens_token_hash_key'
        AND conrelid = 'public.password_reset_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.password_reset_tokens
            ADD CONSTRAINT password_reset_tokens_token_hash_key UNIQUE (token_hash);
    END IF;
END $$;


--
-- Name: permissions permissions_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'permissions_name_key'
        AND conrelid = 'public.permissions'::regclass
    ) THEN
        ALTER TABLE ONLY public.permissions
            ADD CONSTRAINT permissions_name_key UNIQUE (name);
    END IF;
END $$;


--
-- Name: permissions permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'permissions_pkey'
        AND conrelid = 'public.permissions'::regclass
    ) THEN
        ALTER TABLE ONLY public.permissions
            ADD CONSTRAINT permissions_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: platform_config platform_config_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'platform_config_pkey'
        AND conrelid = 'public.platform_config'::regclass
    ) THEN
        ALTER TABLE ONLY public.platform_config
            ADD CONSTRAINT platform_config_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: platform_config platform_config_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'platform_config_slug_key'
        AND conrelid = 'public.platform_config'::regclass
    ) THEN
        ALTER TABLE ONLY public.platform_config
            ADD CONSTRAINT platform_config_slug_key UNIQUE (slug);
    END IF;
END $$;


--
-- Name: relation_tuples relation_tuples_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'relation_tuples_pkey'
        AND conrelid = 'public.relation_tuples'::regclass
    ) THEN
        ALTER TABLE ONLY public.relation_tuples
            ADD CONSTRAINT relation_tuples_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: role_permissions role_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'role_permissions_pkey'
        AND conrelid = 'public.role_permissions'::regclass
    ) THEN
        ALTER TABLE ONLY public.role_permissions
            ADD CONSTRAINT role_permissions_pkey PRIMARY KEY (role_id, permission_id);
    END IF;
END $$;


--
-- Name: roles roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'roles_pkey'
        AND conrelid = 'public.roles'::regclass
    ) THEN
        ALTER TABLE ONLY public.roles
            ADD CONSTRAINT roles_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: roles roles_tenant_id_name_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'roles_tenant_id_name_key'
        AND conrelid = 'public.roles'::regclass
    ) THEN
        ALTER TABLE ONLY public.roles
            ADD CONSTRAINT roles_tenant_id_name_key UNIQUE (tenant_id, name);
    END IF;
END $$;


--
-- Name: sessions sessions_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'sessions_pkey'
        AND conrelid = 'public.sessions'::regclass
    ) THEN
        ALTER TABLE ONLY public.sessions
            ADD CONSTRAINT sessions_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: organizations tenants_custom_domain_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'tenants_custom_domain_key'
        AND conrelid = 'public.organizations'::regclass
    ) THEN
        ALTER TABLE ONLY public.organizations
            ADD CONSTRAINT tenants_custom_domain_key UNIQUE (custom_domain);
    END IF;
END $$;


--
-- Name: organizations tenants_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'tenants_pkey'
        AND conrelid = 'public.organizations'::regclass
    ) THEN
        ALTER TABLE ONLY public.organizations
            ADD CONSTRAINT tenants_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: organizations tenants_slug_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'tenants_slug_key'
        AND conrelid = 'public.organizations'::regclass
    ) THEN
        ALTER TABLE ONLY public.organizations
            ADD CONSTRAINT tenants_slug_key UNIQUE (slug);
    END IF;
END $$;


--
-- Name: user_bans user_bans_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_bans_pkey'
        AND conrelid = 'public.user_bans'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_bans
            ADD CONSTRAINT user_bans_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: user_roles user_roles_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_roles_pkey'
        AND conrelid = 'public.user_roles'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_roles
            ADD CONSTRAINT user_roles_pkey PRIMARY KEY (user_id, role_id);
    END IF;
END $$;


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'users_pkey'
        AND conrelid = 'public.users'::regclass
    ) THEN
        ALTER TABLE ONLY public.users
            ADD CONSTRAINT users_pkey PRIMARY KEY (id);
    END IF;
END $$;


--
-- Name: users users_tenant_id_email_key; Type: CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'users_tenant_id_email_key'
        AND conrelid = 'public.users'::regclass
    ) THEN
        ALTER TABLE ONLY public.users
            ADD CONSTRAINT users_tenant_id_email_key UNIQUE (default_organization_id, email);
    END IF;
END $$;


--
-- Name: idx_audit_logs_category; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_audit_logs_category ON ONLY public.audit_logs USING btree (event_category, created_at DESC);


--
-- Name: audit_logs_2026_02_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_02_event_category_created_at_idx ON public.audit_logs_2026_02 USING btree (event_category, created_at DESC);


--
-- Name: idx_audit_logs_event; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_audit_logs_event ON ONLY public.audit_logs USING btree (event_type, created_at DESC);


--
-- Name: audit_logs_2026_02_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_02_event_type_created_at_idx ON public.audit_logs_2026_02 USING btree (event_type, created_at DESC);


--
-- Name: idx_audit_logs_org; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_audit_logs_org ON ONLY public.audit_logs USING btree (organization_id) WHERE (organization_id IS NOT NULL);


--
-- Name: audit_logs_2026_02_organization_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_02_organization_id_idx ON public.audit_logs_2026_02 USING btree (organization_id) WHERE (organization_id IS NOT NULL);


--
-- Name: idx_audit_logs_request; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_audit_logs_request ON ONLY public.audit_logs USING btree (request_id) WHERE (request_id IS NOT NULL);


--
-- Name: audit_logs_2026_02_request_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_02_request_id_idx ON public.audit_logs_2026_02 USING btree (request_id) WHERE (request_id IS NOT NULL);


--
-- Name: idx_audit_logs_status; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_audit_logs_status ON ONLY public.audit_logs USING btree (status, created_at DESC);


--
-- Name: audit_logs_2026_02_status_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_02_status_created_at_idx ON public.audit_logs_2026_02 USING btree (status, created_at DESC);


--
-- Name: idx_audit_logs_actor; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON ONLY public.audit_logs USING btree (tenant_id, actor_type, actor_id, created_at DESC);


--
-- Name: audit_logs_2026_02_tenant_id_actor_type_actor_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_02_tenant_id_actor_type_actor_id_created_at_idx ON public.audit_logs_2026_02 USING btree (tenant_id, actor_type, actor_id, created_at DESC);


--
-- Name: idx_audit_logs_tenant; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_audit_logs_tenant ON ONLY public.audit_logs USING btree (tenant_id, created_at DESC);


--
-- Name: audit_logs_2026_02_tenant_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_02_tenant_id_created_at_idx ON public.audit_logs_2026_02 USING btree (tenant_id, created_at DESC);


--
-- Name: idx_audit_logs_target; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_audit_logs_target ON ONLY public.audit_logs USING btree (tenant_id, target_type, target_id, created_at DESC);


--
-- Name: audit_logs_2026_02_tenant_id_target_type_target_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_02_tenant_id_target_type_target_id_created__idx ON public.audit_logs_2026_02 USING btree (tenant_id, target_type, target_id, created_at DESC);


--
-- Name: audit_logs_2026_03_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_03_event_category_created_at_idx ON public.audit_logs_2026_03 USING btree (event_category, created_at DESC);


--
-- Name: audit_logs_2026_03_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_03_event_type_created_at_idx ON public.audit_logs_2026_03 USING btree (event_type, created_at DESC);


--
-- Name: audit_logs_2026_03_organization_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_03_organization_id_idx ON public.audit_logs_2026_03 USING btree (organization_id) WHERE (organization_id IS NOT NULL);


--
-- Name: audit_logs_2026_03_request_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_03_request_id_idx ON public.audit_logs_2026_03 USING btree (request_id) WHERE (request_id IS NOT NULL);


--
-- Name: audit_logs_2026_03_status_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_03_status_created_at_idx ON public.audit_logs_2026_03 USING btree (status, created_at DESC);


--
-- Name: audit_logs_2026_03_tenant_id_actor_type_actor_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_03_tenant_id_actor_type_actor_id_created_at_idx ON public.audit_logs_2026_03 USING btree (tenant_id, actor_type, actor_id, created_at DESC);


--
-- Name: audit_logs_2026_03_tenant_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_03_tenant_id_created_at_idx ON public.audit_logs_2026_03 USING btree (tenant_id, created_at DESC);


--
-- Name: audit_logs_2026_03_tenant_id_target_type_target_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_03_tenant_id_target_type_target_id_created__idx ON public.audit_logs_2026_03 USING btree (tenant_id, target_type, target_id, created_at DESC);


--
-- Name: audit_logs_2026_04_event_category_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_04_event_category_created_at_idx ON public.audit_logs_2026_04 USING btree (event_category, created_at DESC);


--
-- Name: audit_logs_2026_04_event_type_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_04_event_type_created_at_idx ON public.audit_logs_2026_04 USING btree (event_type, created_at DESC);


--
-- Name: audit_logs_2026_04_organization_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_04_organization_id_idx ON public.audit_logs_2026_04 USING btree (organization_id) WHERE (organization_id IS NOT NULL);


--
-- Name: audit_logs_2026_04_request_id_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_04_request_id_idx ON public.audit_logs_2026_04 USING btree (request_id) WHERE (request_id IS NOT NULL);


--
-- Name: audit_logs_2026_04_status_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_04_status_created_at_idx ON public.audit_logs_2026_04 USING btree (status, created_at DESC);


--
-- Name: audit_logs_2026_04_tenant_id_actor_type_actor_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_04_tenant_id_actor_type_actor_id_created_at_idx ON public.audit_logs_2026_04 USING btree (tenant_id, actor_type, actor_id, created_at DESC);


--
-- Name: audit_logs_2026_04_tenant_id_created_at_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_04_tenant_id_created_at_idx ON public.audit_logs_2026_04 USING btree (tenant_id, created_at DESC);


--
-- Name: audit_logs_2026_04_tenant_id_target_type_target_id_created__idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS audit_logs_2026_04_tenant_id_target_type_target_id_created__idx ON public.audit_logs_2026_04 USING btree (tenant_id, target_type, target_id, created_at DESC);


--
-- Name: idx_account_lockouts_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_account_lockouts_active ON public.account_lockouts USING btree (user_id, locked_until) WHERE (unlocked_at IS NULL);


--
-- Name: idx_account_lockouts_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_account_lockouts_user ON public.account_lockouts USING btree (user_id, locked_until);


--
-- Name: idx_applications_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_applications_client_id ON public.applications USING btree (client_id);


--
-- Name: idx_applications_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_applications_enabled ON public.applications USING btree (is_enabled) WHERE (is_enabled = true);


--
-- Name: idx_applications_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_applications_slug ON public.applications USING btree (slug);


--
-- Name: idx_blocked_domains_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_blocked_domains_lookup ON public.blocked_email_domains USING btree (domain);


--
-- Name: idx_blocked_domains_tenant; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_blocked_domains_tenant ON public.blocked_email_domains USING btree (tenant_id);


--
-- Name: idx_connections_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_connections_enabled ON public.connections USING btree (is_enabled) WHERE (is_enabled = true);


--
-- Name: idx_connections_org; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_connections_org ON public.connections USING btree (organization_id) WHERE (organization_id IS NOT NULL);


--
-- Name: idx_connections_scope; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_connections_scope ON public.connections USING btree (scope);


--
-- Name: idx_connections_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_connections_type ON public.connections USING btree (type);


--
-- Name: idx_email_verification_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_email_verification_expires ON public.email_verification_tokens USING btree (expires_at) WHERE (used_at IS NULL);


--
-- Name: idx_email_verification_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_email_verification_user ON public.email_verification_tokens USING btree (user_id);


--
-- Name: idx_invitations_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_invitations_email ON public.invitations USING btree (tenant_id, email);


--
-- Name: idx_invitations_pending; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_invitations_pending ON public.invitations USING btree (tenant_id, expires_at) WHERE (accepted_at IS NULL);


--
-- Name: idx_invitations_tenant; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_invitations_tenant ON public.invitations USING btree (tenant_id);


--
-- Name: idx_login_attempts_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_login_attempts_email ON public.login_attempts USING btree (tenant_id, email, attempted_at DESC);


--
-- Name: idx_login_attempts_ip; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_login_attempts_ip ON public.login_attempts USING btree (ip_address, attempted_at DESC);


--
-- Name: idx_login_attempts_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_login_attempts_user ON public.login_attempts USING btree (user_id, attempted_at DESC);


--
-- Name: idx_magic_link_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_magic_link_email ON public.magic_link_tokens USING btree (tenant_id, email);


--
-- Name: idx_magic_link_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_magic_link_expires ON public.magic_link_tokens USING btree (expires_at) WHERE (used_at IS NULL);


--
-- Name: idx_mfa_backup_codes_unused; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_mfa_backup_codes_unused ON public.mfa_backup_codes USING btree (user_id, used_at) WHERE (used_at IS NULL);


--
-- Name: idx_mfa_backup_codes_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_mfa_backup_codes_user ON public.mfa_backup_codes USING btree (user_id);


--
-- Name: idx_mfa_challenges_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_mfa_challenges_expires ON public.mfa_challenges USING btree (expires_at) WHERE (verified = false);


--
-- Name: idx_mfa_challenges_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_mfa_challenges_token ON public.mfa_challenges USING btree (challenge_token);


--
-- Name: idx_mfa_challenges_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_mfa_challenges_user ON public.mfa_challenges USING btree (user_id);


--
-- Name: idx_mfa_methods_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_mfa_methods_type ON public.mfa_methods USING btree (user_id, method_type);


--
-- Name: idx_mfa_methods_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_mfa_methods_user ON public.mfa_methods USING btree (user_id);


--
-- Name: idx_oauth_clients_client_id; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_oauth_clients_client_id ON public.oauth_clients USING btree (client_id);


--
-- Name: idx_oauth_clients_tenant; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_oauth_clients_tenant ON public.oauth_clients USING btree (tenant_id);


--
-- Name: idx_oidc_providers_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_oidc_providers_active ON public.oidc_providers USING btree (is_active) WHERE (is_active = true);


--
-- Name: idx_oidc_providers_tenant; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_oidc_providers_tenant ON public.oidc_providers USING btree (tenant_id);


--
-- Name: idx_oidc_providers_type; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_oidc_providers_type ON public.oidc_providers USING btree (provider_type);


--
-- Name: idx_org_members_org; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_org_members_org ON public.organization_members USING btree (organization_id);


--
-- Name: idx_org_members_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_org_members_role ON public.organization_members USING btree (organization_id, role);


--
-- Name: idx_org_members_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_org_members_user ON public.organization_members USING btree (user_id);


--
-- Name: idx_organizations_slug; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_organizations_slug ON public.organizations USING btree (slug);


--
-- Name: idx_password_reset_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_password_reset_expires ON public.password_reset_tokens USING btree (expires_at) WHERE (used_at IS NULL);


--
-- Name: idx_password_reset_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_password_reset_user ON public.password_reset_tokens USING btree (user_id);


--
-- Name: idx_permissions_resource_action; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_permissions_resource_action ON public.permissions USING btree (resource, action);


--
-- Name: idx_relation_tuples_org; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_relation_tuples_org ON public.relation_tuples USING btree (organization_id) WHERE (organization_id IS NOT NULL);


--
-- Name: idx_roles_parent; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_roles_parent ON public.roles USING btree (parent_role_id) WHERE (parent_role_id IS NOT NULL);


--
-- Name: idx_roles_tenant; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_roles_tenant ON public.roles USING btree (tenant_id);


--
-- Name: idx_sessions_expires; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_sessions_expires ON public.sessions USING btree (expires_at);


--
-- Name: idx_sessions_token; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_sessions_token ON public.sessions USING btree (token_hash);


--
-- Name: idx_sessions_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_sessions_user ON public.sessions USING btree (user_id);


--
-- Name: idx_tenants_custom_domain; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_tenants_custom_domain ON public.organizations USING btree (custom_domain) WHERE (custom_domain IS NOT NULL);


--
-- Name: idx_tuples_lookup; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_tuples_lookup ON public.relation_tuples USING btree (tenant_id, namespace, object_id, relation);


--
-- Name: idx_tuples_ns_rel; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_tuples_ns_rel ON public.relation_tuples USING btree (tenant_id, namespace, relation);


--
-- Name: idx_tuples_object; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_tuples_object ON public.relation_tuples USING btree (tenant_id, namespace, object_id);


--
-- Name: idx_tuples_subject; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_tuples_subject ON public.relation_tuples USING btree (tenant_id, subject_type, subject_id);


--
-- Name: idx_tuples_unique; Type: INDEX; Schema: public; Owner: -
--

CREATE UNIQUE INDEX idx_tuples_unique ON public.relation_tuples USING btree (tenant_id, namespace, object_id, relation, subject_type, subject_id, COALESCE(subject_relation, ''::character varying));


--
-- Name: idx_user_bans_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_user_bans_email ON public.user_bans USING btree (tenant_id, email) WHERE (unbanned_at IS NULL);


--
-- Name: idx_user_bans_ip; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_user_bans_ip ON public.user_bans USING btree (ip_address) WHERE (unbanned_at IS NULL);


--
-- Name: idx_user_bans_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_user_bans_user ON public.user_bans USING btree (user_id) WHERE (unbanned_at IS NULL);


--
-- Name: idx_user_roles_role; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_user_roles_role ON public.user_roles USING btree (role_id);


--
-- Name: idx_user_roles_user; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_user_roles_user ON public.user_roles USING btree (user_id);


--
-- Name: idx_users_mfa_enabled; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_users_mfa_enabled ON public.users USING btree (default_organization_id, mfa_enabled) WHERE (mfa_enabled = true);


--
-- Name: idx_users_phone; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_users_phone ON public.users USING btree (phone) WHERE (phone IS NOT NULL);


--
-- Name: idx_users_platform_admin; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_users_platform_admin ON public.users USING btree (is_platform_admin) WHERE (is_platform_admin = true);


--
-- Name: idx_users_provider; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_users_provider ON public.users USING btree (provider_id, provider_user_id);


--
-- Name: idx_users_tenant_active; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_users_tenant_active ON public.users USING btree (default_organization_id, is_active);


--
-- Name: idx_users_tenant_email; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX IF NOT EXISTS idx_users_tenant_email ON public.users USING btree (default_organization_id, email);


--
-- Name: audit_logs_2026_02_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_category ATTACH PARTITION public.audit_logs_2026_02_event_category_created_at_idx;


--
-- Name: audit_logs_2026_02_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_event ATTACH PARTITION public.audit_logs_2026_02_event_type_created_at_idx;


--
-- Name: audit_logs_2026_02_organization_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_org ATTACH PARTITION public.audit_logs_2026_02_organization_id_idx;


--
-- Name: audit_logs_2026_02_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_2026_02_pkey;


--
-- Name: audit_logs_2026_02_request_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_request ATTACH PARTITION public.audit_logs_2026_02_request_id_idx;


--
-- Name: audit_logs_2026_02_status_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_status ATTACH PARTITION public.audit_logs_2026_02_status_created_at_idx;


--
-- Name: audit_logs_2026_02_tenant_id_actor_type_actor_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_actor ATTACH PARTITION public.audit_logs_2026_02_tenant_id_actor_type_actor_id_created_at_idx;


--
-- Name: audit_logs_2026_02_tenant_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_tenant ATTACH PARTITION public.audit_logs_2026_02_tenant_id_created_at_idx;


--
-- Name: audit_logs_2026_02_tenant_id_target_type_target_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target ATTACH PARTITION public.audit_logs_2026_02_tenant_id_target_type_target_id_created__idx;


--
-- Name: audit_logs_2026_03_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_category ATTACH PARTITION public.audit_logs_2026_03_event_category_created_at_idx;


--
-- Name: audit_logs_2026_03_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_event ATTACH PARTITION public.audit_logs_2026_03_event_type_created_at_idx;


--
-- Name: audit_logs_2026_03_organization_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_org ATTACH PARTITION public.audit_logs_2026_03_organization_id_idx;


--
-- Name: audit_logs_2026_03_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_2026_03_pkey;


--
-- Name: audit_logs_2026_03_request_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_request ATTACH PARTITION public.audit_logs_2026_03_request_id_idx;


--
-- Name: audit_logs_2026_03_status_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_status ATTACH PARTITION public.audit_logs_2026_03_status_created_at_idx;


--
-- Name: audit_logs_2026_03_tenant_id_actor_type_actor_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_actor ATTACH PARTITION public.audit_logs_2026_03_tenant_id_actor_type_actor_id_created_at_idx;


--
-- Name: audit_logs_2026_03_tenant_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_tenant ATTACH PARTITION public.audit_logs_2026_03_tenant_id_created_at_idx;


--
-- Name: audit_logs_2026_03_tenant_id_target_type_target_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target ATTACH PARTITION public.audit_logs_2026_03_tenant_id_target_type_target_id_created__idx;


--
-- Name: audit_logs_2026_04_event_category_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_category ATTACH PARTITION public.audit_logs_2026_04_event_category_created_at_idx;


--
-- Name: audit_logs_2026_04_event_type_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_event ATTACH PARTITION public.audit_logs_2026_04_event_type_created_at_idx;


--
-- Name: audit_logs_2026_04_organization_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_org ATTACH PARTITION public.audit_logs_2026_04_organization_id_idx;


--
-- Name: audit_logs_2026_04_pkey; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.audit_logs_pkey ATTACH PARTITION public.audit_logs_2026_04_pkey;


--
-- Name: audit_logs_2026_04_request_id_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_request ATTACH PARTITION public.audit_logs_2026_04_request_id_idx;


--
-- Name: audit_logs_2026_04_status_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_status ATTACH PARTITION public.audit_logs_2026_04_status_created_at_idx;


--
-- Name: audit_logs_2026_04_tenant_id_actor_type_actor_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_actor ATTACH PARTITION public.audit_logs_2026_04_tenant_id_actor_type_actor_id_created_at_idx;


--
-- Name: audit_logs_2026_04_tenant_id_created_at_idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_tenant ATTACH PARTITION public.audit_logs_2026_04_tenant_id_created_at_idx;


--
-- Name: audit_logs_2026_04_tenant_id_target_type_target_id_created__idx; Type: INDEX ATTACH; Schema: public; Owner: -
--

ALTER INDEX public.idx_audit_logs_target ATTACH PARTITION public.audit_logs_2026_04_tenant_id_target_type_target_id_created__idx;


--
-- Name: applications update_applications_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_applications_updated_at BEFORE UPDATE ON public.applications FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: connections update_connections_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_connections_updated_at BEFORE UPDATE ON public.connections FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: oauth_clients update_oauth_clients_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_oauth_clients_updated_at BEFORE UPDATE ON public.oauth_clients FOR EACH ROW EXECUTE FUNCTION public.update_updated_at();


--
-- Name: oidc_providers update_oidc_providers_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_oidc_providers_updated_at BEFORE UPDATE ON public.oidc_providers FOR EACH ROW EXECUTE FUNCTION public.update_updated_at();


--
-- Name: roles update_roles_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON public.roles FOR EACH ROW EXECUTE FUNCTION public.update_updated_at();


--
-- Name: organizations update_tenants_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON public.organizations FOR EACH ROW EXECUTE FUNCTION public.update_updated_at();


--
-- Name: users update_users_updated_at; Type: TRIGGER; Schema: public; Owner: -
--

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.update_updated_at();


--
-- Name: account_lockouts account_lockouts_locked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'account_lockouts_locked_by_fkey'
        AND conrelid = 'public.account_lockouts'::regclass
    ) THEN
        ALTER TABLE ONLY public.account_lockouts
            ADD CONSTRAINT account_lockouts_locked_by_fkey FOREIGN KEY (locked_by) REFERENCES public.users(id);
    END IF;
END $$;


--
-- Name: account_lockouts account_lockouts_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'account_lockouts_tenant_id_fkey'
        AND conrelid = 'public.account_lockouts'::regclass
    ) THEN
        ALTER TABLE ONLY public.account_lockouts
            ADD CONSTRAINT account_lockouts_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: account_lockouts account_lockouts_unlocked_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'account_lockouts_unlocked_by_fkey'
        AND conrelid = 'public.account_lockouts'::regclass
    ) THEN
        ALTER TABLE ONLY public.account_lockouts
            ADD CONSTRAINT account_lockouts_unlocked_by_fkey FOREIGN KEY (unlocked_by) REFERENCES public.users(id);
    END IF;
END $$;


--
-- Name: account_lockouts account_lockouts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'account_lockouts_user_id_fkey'
        AND conrelid = 'public.account_lockouts'::regclass
    ) THEN
        ALTER TABLE ONLY public.account_lockouts
            ADD CONSTRAINT account_lockouts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: audit_logs audit_logs_organization_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE public.audit_logs
    ADD CONSTRAINT audit_logs_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES public.organizations(id) ON DELETE CASCADE;


--
-- Name: audit_logs audit_logs_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE public.audit_logs
    ADD CONSTRAINT audit_logs_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;


--
-- Name: blocked_email_domains blocked_email_domains_added_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'blocked_email_domains_added_by_fkey'
        AND conrelid = 'public.blocked_email_domains'::regclass
    ) THEN
        ALTER TABLE ONLY public.blocked_email_domains
            ADD CONSTRAINT blocked_email_domains_added_by_fkey FOREIGN KEY (added_by) REFERENCES public.users(id);
    END IF;
END $$;


--
-- Name: blocked_email_domains blocked_email_domains_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'blocked_email_domains_tenant_id_fkey'
        AND conrelid = 'public.blocked_email_domains'::regclass
    ) THEN
        ALTER TABLE ONLY public.blocked_email_domains
            ADD CONSTRAINT blocked_email_domains_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: connections connections_organization_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'connections_organization_id_fkey'
        AND conrelid = 'public.connections'::regclass
    ) THEN
        ALTER TABLE ONLY public.connections
            ADD CONSTRAINT connections_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: email_verification_tokens email_verification_tokens_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'email_verification_tokens_tenant_id_fkey'
        AND conrelid = 'public.email_verification_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.email_verification_tokens
            ADD CONSTRAINT email_verification_tokens_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: email_verification_tokens email_verification_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'email_verification_tokens_user_id_fkey'
        AND conrelid = 'public.email_verification_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.email_verification_tokens
            ADD CONSTRAINT email_verification_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: invitations invitations_accepted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'invitations_accepted_by_fkey'
        AND conrelid = 'public.invitations'::regclass
    ) THEN
        ALTER TABLE ONLY public.invitations
            ADD CONSTRAINT invitations_accepted_by_fkey FOREIGN KEY (accepted_by) REFERENCES public.users(id);
    END IF;
END $$;


--
-- Name: invitations invitations_invited_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'invitations_invited_by_fkey'
        AND conrelid = 'public.invitations'::regclass
    ) THEN
        ALTER TABLE ONLY public.invitations
            ADD CONSTRAINT invitations_invited_by_fkey FOREIGN KEY (invited_by) REFERENCES public.users(id);
    END IF;
END $$;


--
-- Name: invitations invitations_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'invitations_role_id_fkey'
        AND conrelid = 'public.invitations'::regclass
    ) THEN
        ALTER TABLE ONLY public.invitations
            ADD CONSTRAINT invitations_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id);
    END IF;
END $$;


--
-- Name: invitations invitations_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'invitations_tenant_id_fkey'
        AND conrelid = 'public.invitations'::regclass
    ) THEN
        ALTER TABLE ONLY public.invitations
            ADD CONSTRAINT invitations_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: login_attempts login_attempts_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'login_attempts_tenant_id_fkey'
        AND conrelid = 'public.login_attempts'::regclass
    ) THEN
        ALTER TABLE ONLY public.login_attempts
            ADD CONSTRAINT login_attempts_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: login_attempts login_attempts_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'login_attempts_user_id_fkey'
        AND conrelid = 'public.login_attempts'::regclass
    ) THEN
        ALTER TABLE ONLY public.login_attempts
            ADD CONSTRAINT login_attempts_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: magic_link_tokens magic_link_tokens_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'magic_link_tokens_tenant_id_fkey'
        AND conrelid = 'public.magic_link_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.magic_link_tokens
            ADD CONSTRAINT magic_link_tokens_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: magic_link_tokens magic_link_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'magic_link_tokens_user_id_fkey'
        AND conrelid = 'public.magic_link_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.magic_link_tokens
            ADD CONSTRAINT magic_link_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: mfa_backup_codes mfa_backup_codes_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'mfa_backup_codes_user_id_fkey'
        AND conrelid = 'public.mfa_backup_codes'::regclass
    ) THEN
        ALTER TABLE ONLY public.mfa_backup_codes
            ADD CONSTRAINT mfa_backup_codes_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: mfa_challenges mfa_challenges_method_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'mfa_challenges_method_id_fkey'
        AND conrelid = 'public.mfa_challenges'::regclass
    ) THEN
        ALTER TABLE ONLY public.mfa_challenges
            ADD CONSTRAINT mfa_challenges_method_id_fkey FOREIGN KEY (method_id) REFERENCES public.mfa_methods(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: mfa_challenges mfa_challenges_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'mfa_challenges_user_id_fkey'
        AND conrelid = 'public.mfa_challenges'::regclass
    ) THEN
        ALTER TABLE ONLY public.mfa_challenges
            ADD CONSTRAINT mfa_challenges_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: mfa_methods mfa_methods_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'mfa_methods_user_id_fkey'
        AND conrelid = 'public.mfa_methods'::regclass
    ) THEN
        ALTER TABLE ONLY public.mfa_methods
            ADD CONSTRAINT mfa_methods_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: oauth_clients oauth_clients_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'oauth_clients_tenant_id_fkey'
        AND conrelid = 'public.oauth_clients'::regclass
    ) THEN
        ALTER TABLE ONLY public.oauth_clients
            ADD CONSTRAINT oauth_clients_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: oidc_providers oidc_providers_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'oidc_providers_tenant_id_fkey'
        AND conrelid = 'public.oidc_providers'::regclass
    ) THEN
        ALTER TABLE ONLY public.oidc_providers
            ADD CONSTRAINT oidc_providers_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: organization_members organization_members_organization_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'organization_members_organization_id_fkey'
        AND conrelid = 'public.organization_members'::regclass
    ) THEN
        ALTER TABLE ONLY public.organization_members
            ADD CONSTRAINT organization_members_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: organization_members organization_members_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'organization_members_user_id_fkey'
        AND conrelid = 'public.organization_members'::regclass
    ) THEN
        ALTER TABLE ONLY public.organization_members
            ADD CONSTRAINT organization_members_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: password_reset_tokens password_reset_tokens_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'password_reset_tokens_tenant_id_fkey'
        AND conrelid = 'public.password_reset_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.password_reset_tokens
            ADD CONSTRAINT password_reset_tokens_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: password_reset_tokens password_reset_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'password_reset_tokens_user_id_fkey'
        AND conrelid = 'public.password_reset_tokens'::regclass
    ) THEN
        ALTER TABLE ONLY public.password_reset_tokens
            ADD CONSTRAINT password_reset_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: relation_tuples relation_tuples_organization_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'relation_tuples_organization_id_fkey'
        AND conrelid = 'public.relation_tuples'::regclass
    ) THEN
        ALTER TABLE ONLY public.relation_tuples
            ADD CONSTRAINT relation_tuples_organization_id_fkey FOREIGN KEY (organization_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: relation_tuples relation_tuples_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'relation_tuples_tenant_id_fkey'
        AND conrelid = 'public.relation_tuples'::regclass
    ) THEN
        ALTER TABLE ONLY public.relation_tuples
            ADD CONSTRAINT relation_tuples_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: role_permissions role_permissions_permission_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'role_permissions_permission_id_fkey'
        AND conrelid = 'public.role_permissions'::regclass
    ) THEN
        ALTER TABLE ONLY public.role_permissions
            ADD CONSTRAINT role_permissions_permission_id_fkey FOREIGN KEY (permission_id) REFERENCES public.permissions(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: role_permissions role_permissions_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'role_permissions_role_id_fkey'
        AND conrelid = 'public.role_permissions'::regclass
    ) THEN
        ALTER TABLE ONLY public.role_permissions
            ADD CONSTRAINT role_permissions_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: roles roles_parent_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'roles_parent_role_id_fkey'
        AND conrelid = 'public.roles'::regclass
    ) THEN
        ALTER TABLE ONLY public.roles
            ADD CONSTRAINT roles_parent_role_id_fkey FOREIGN KEY (parent_role_id) REFERENCES public.roles(id);
    END IF;
END $$;


--
-- Name: roles roles_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'roles_tenant_id_fkey'
        AND conrelid = 'public.roles'::regclass
    ) THEN
        ALTER TABLE ONLY public.roles
            ADD CONSTRAINT roles_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: sessions sessions_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'sessions_user_id_fkey'
        AND conrelid = 'public.sessions'::regclass
    ) THEN
        ALTER TABLE ONLY public.sessions
            ADD CONSTRAINT sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: user_bans user_bans_banned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_bans_banned_by_fkey'
        AND conrelid = 'public.user_bans'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_bans
            ADD CONSTRAINT user_bans_banned_by_fkey FOREIGN KEY (banned_by) REFERENCES public.users(id);
    END IF;
END $$;


--
-- Name: user_bans user_bans_tenant_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_bans_tenant_id_fkey'
        AND conrelid = 'public.user_bans'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_bans
            ADD CONSTRAINT user_bans_tenant_id_fkey FOREIGN KEY (tenant_id) REFERENCES public.organizations(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: user_bans user_bans_unbanned_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_bans_unbanned_by_fkey'
        AND conrelid = 'public.user_bans'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_bans
            ADD CONSTRAINT user_bans_unbanned_by_fkey FOREIGN KEY (unbanned_by) REFERENCES public.users(id);
    END IF;
END $$;


--
-- Name: user_bans user_bans_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_bans_user_id_fkey'
        AND conrelid = 'public.user_bans'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_bans
            ADD CONSTRAINT user_bans_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: user_roles user_roles_granted_by_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_roles_granted_by_fkey'
        AND conrelid = 'public.user_roles'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_roles
            ADD CONSTRAINT user_roles_granted_by_fkey FOREIGN KEY (granted_by) REFERENCES public.users(id);
    END IF;
END $$;


--
-- Name: user_roles user_roles_role_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_roles_role_id_fkey'
        AND conrelid = 'public.user_roles'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_roles
            ADD CONSTRAINT user_roles_role_id_fkey FOREIGN KEY (role_id) REFERENCES public.roles(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- Name: user_roles user_roles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: -
--

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'user_roles_user_id_fkey'
        AND conrelid = 'public.user_roles'::regclass
    ) THEN
        ALTER TABLE ONLY public.user_roles
            ADD CONSTRAINT user_roles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;
    END IF;
END $$;


--
-- PostgreSQL database dump complete
--



-- ============================================================
-- Default Data
-- ============================================================

-- Insert platform configuration for on-prem deployment
INSERT INTO platform_config (name, slug)
VALUES ('OnPrem CIAM Platform', 'onprem')
ON CONFLICT (slug) DO NOTHING;

-- Insert default permissions
INSERT INTO permissions (name, description, resource, action) VALUES
    ('users:read', 'Read user information', 'users', 'read'),
    ('users:write', 'Create and update users', 'users', 'write'),
    ('users:delete', 'Delete users', 'users', 'delete'),
    ('roles:read', 'Read role information', 'roles', 'read'),
    ('roles:write', 'Create and update roles', 'roles', 'write'),
    ('roles:delete', 'Delete roles', 'roles', 'delete'),
    ('tenants:read', 'Read tenant information', 'tenants', 'read'),
    ('tenants:write', 'Create and update tenants', 'tenants', 'write'),
    ('tenants:delete', 'Delete tenants', 'tenants', 'delete'),
    ('audit:read', 'Read audit logs', 'audit', 'read')
ON CONFLICT (name) DO NOTHING;

-- ============================================================

-- ============================================================
