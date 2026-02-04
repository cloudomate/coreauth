use crate::handlers;
use crate::middleware;
use crate::AppState;
use axum::{
    middleware::from_fn_with_state,
    routing::{delete, get, patch, post, put},
    Router,
};
use std::sync::Arc;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health::health_check))
        // OAuth2/OIDC Authorization Server - Public endpoints
        .route("/.well-known/openid-configuration", get(handlers::oauth2::openid_configuration))
        .route("/.well-known/jwks.json", get(handlers::oauth2::jwks))
        .route("/authorize", get(handlers::oauth2::authorize))
        .route("/oauth/token", post(handlers::oauth2::token))
        .route("/userinfo", get(handlers::oauth2::userinfo).post(handlers::oauth2::userinfo))
        .route("/oauth/revoke", post(handlers::oauth2::revoke))
        .route("/oauth/introspect", post(handlers::oauth2::introspect))
        .route("/logout", get(handlers::oauth2::logout))
        // Universal Login - Public endpoints
        .route("/login", get(handlers::universal_login::login_page).post(handlers::universal_login::login_submit))
        .route("/signup", get(handlers::universal_login::signup_page).post(handlers::universal_login::signup_submit))
        .route("/mfa", get(handlers::universal_login::mfa_page))
        .route("/mfa/verify", post(handlers::universal_login::mfa_verify))
        .route("/consent", get(handlers::universal_login::consent_page).post(handlers::universal_login::consent_submit))
        .route("/logged-out", get(handlers::universal_login::logged_out_page))
        // Social Login - Public endpoints
        .route("/login/social/:connection_id", get(handlers::social_login::social_login))
        .route("/login/social/callback", get(handlers::social_login::social_callback))
        // Test endpoints (development only)
        .route("/api/test/email", post(handlers::test::test_email))
        .route("/api/test/sms", post(handlers::test::test_sms))
        .route("/api/test/connectivity", get(handlers::test::test_connectivity))
        // Tenant onboarding (public)
        .route("/api/tenants", post(handlers::tenant::create_tenant))
        // Organization lookup by slug (public - for login page)
        .route("/api/organizations/by-slug/:slug", get(handlers::tenant::get_organization_by_slug))
        // Auth routes
        .route(
            "/api/auth/register",
            post(handlers::auth::register)
                .layer(axum::middleware::from_fn_with_state(
                    state.cache.clone(),
                    middleware::rate_limit_registration,
                ))
        )
        .route(
            "/api/auth/login",
            post(handlers::auth::login)
                .layer(axum::middleware::from_fn_with_state(
                    state.cache.clone(),
                    middleware::rate_limit_login,
                ))
        )
        .route(
            "/api/auth/login-hierarchical",
            post(handlers::auth::login_hierarchical)
                .layer(axum::middleware::from_fn_with_state(
                    state.cache.clone(),
                    middleware::rate_limit_login,
                ))
        )
        .route("/api/auth/refresh", post(handlers::auth::refresh_token))
        .route("/api/auth/logout", post(handlers::auth::logout))
        .route("/api/auth/me", get(handlers::auth::me))
        // Email verification routes
        .route("/api/verify-email", get(handlers::verification::verify_email))
        .route(
            "/api/auth/resend-verification",
            post(handlers::verification::resend_verification)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        // Password reset routes
        .route(
            "/api/auth/forgot-password",
            post(handlers::password_reset::request_password_reset)
                .layer(axum::middleware::from_fn_with_state(
                    state.cache.clone(),
                    middleware::rate_limit_password_reset,
                ))
        )
        .route("/api/auth/verify-reset-token", get(handlers::password_reset::verify_reset_token))
        .route("/api/auth/reset-password", post(handlers::password_reset::reset_password))
        // Tenant users routes - Protected (tenant admin)
        .route(
            "/api/tenants/:tenant_id/users",
            get(handlers::tenant::list_tenant_users)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/tenants/:tenant_id/users/:user_id/role",
            put(handlers::tenant::update_user_role)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // Invitation routes - Public endpoints
        .route("/api/invitations/verify", get(handlers::invitation::verify_invitation))
        .route("/api/invitations/accept", post(handlers::invitation::accept_invitation))
        // Invitation routes - Protected (tenant admin)
        .route(
            "/api/tenants/:tenant_id/invitations",
            post(handlers::invitation::create_invitation)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/tenants/:tenant_id/invitations",
            get(handlers::invitation::list_invitations)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/tenants/:tenant_id/invitations/:invitation_id",
            delete(handlers::invitation::revoke_invitation)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/tenants/:tenant_id/invitations/:invitation_id/resend",
            post(handlers::invitation::resend_invitation)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // OIDC routes - Public endpoints
        .route("/api/oidc/login", get(handlers::oidc::oidc_login))
        .route("/api/oidc/callback", get(handlers::oidc::oidc_callback))
        // OIDC provider templates - Public (anyone can view templates)
        .route("/api/oidc/templates", get(handlers::oidc::list_provider_templates))
        .route("/api/oidc/templates/:provider_type", get(handlers::oidc::get_provider_template))
        // OIDC routes - Protected: List providers (requires auth)
        .route(
            "/api/oidc/providers",
            get(handlers::oidc::list_providers)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        // OIDC routes - Protected: Create provider (requires tenant admin)
        .route(
            "/api/oidc/providers",
            post(handlers::oidc::create_provider)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // OIDC routes - Protected: Update provider (requires tenant admin)
        .route(
            "/api/oidc/providers/:id",
            patch(handlers::oidc::update_provider)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // OIDC routes - Protected: Delete provider (requires tenant admin)
        .route(
            "/api/oidc/providers/:id",
            delete(handlers::oidc::delete_provider)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // MFA enrollment with enrollment token (unauthenticated)
        .route("/api/mfa/enroll-with-token/totp", post(handlers::mfa::enroll_totp_with_token))
        .route("/api/mfa/verify-with-token/totp/:method_id", post(handlers::mfa::verify_totp_with_token))
        // MFA routes - All protected (require auth)
        .route(
            "/api/mfa/enroll/totp",
            post(handlers::mfa::enroll_totp)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/mfa/totp/:method_id/verify",
            post(handlers::mfa::verify_totp)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/mfa/methods",
            get(handlers::mfa::list_mfa_methods)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/mfa/methods/:method_id",
            delete(handlers::mfa::delete_mfa_method)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/mfa/backup-codes/regenerate",
            post(handlers::mfa::regenerate_backup_codes)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        // Organization/Tenant security settings routes - Protected (require tenant admin)
        .route(
            "/api/organizations/:org_id/security",
            get(handlers::tenant::get_security_settings)
                .put(handlers::tenant::update_security_settings)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // Application Management - Protected (require tenant admin)
        .route(
            "/api/applications",
            post(handlers::authz::create_application)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/tenants/:tenant_id/applications",
            get(handlers::authz::list_applications)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/applications/:app_id/tenants/:tenant_id",
            get(handlers::authz::get_application)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/applications/:app_id/tenants/:tenant_id",
            post(handlers::authz::update_application)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/applications/:app_id/tenants/:tenant_id/rotate-secret",
            post(handlers::authz::rotate_secret)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/applications/:app_id/tenants/:tenant_id",
            delete(handlers::authz::delete_application)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // Application Authentication - Public (for client credentials flow)
        .route("/api/applications/authenticate", post(handlers::authz::authenticate_application))
        // Relation Tuple Management - Protected (require auth)
        .route(
            "/api/authz/tuples",
            post(handlers::authz::create_tuple)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/authz/tuples",
            delete(handlers::authz::delete_tuple)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/authz/tuples/query",
            post(handlers::authz::query_tuples)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/authz/tuples/by-object/:tenant_id/:namespace/:object_id",
            get(handlers::authz::get_object_tuples)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/authz/tuples/by-subject/:tenant_id/:subject_type/:subject_id",
            get(handlers::authz::get_subject_tuples)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        // Authorization Checks - Protected (require auth)
        .route(
            "/api/authz/check",
            post(handlers::authz::check_permission)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/authz/expand/:tenant_id/:namespace/:object_id/:relation",
            get(handlers::authz::expand_relation)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        // Forward Auth - Public (for downstream apps)
        .route("/authz/forward-auth", post(handlers::authz::forward_auth))
        .route("/authz/forward-auth", get(handlers::authz::forward_auth_get))
        // Audit Log routes - Protected (require auth)
        .route(
            "/api/audit/logs",
            get(handlers::audit::query_audit_logs)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/audit/logs/:id",
            get(handlers::audit::get_audit_log)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/audit/security-events",
            get(handlers::audit::get_security_events)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/audit/failed-logins/:user_id",
            get(handlers::audit::get_failed_logins)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/audit/export",
            get(handlers::audit::export_audit_logs)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        .route(
            "/api/audit/stats",
            get(handlers::audit::get_audit_stats)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_auth))
        )
        // OAuth Application Management - Protected (require tenant admin)
        .route(
            "/api/organizations/:org_id/applications",
            post(handlers::application::create_application)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/applications",
            get(handlers::application::list_applications)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/applications/:app_id",
            get(handlers::application::get_application)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/applications/:app_id",
            axum::routing::put(handlers::application::update_application)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/applications/:app_id/rotate-secret",
            post(handlers::application::rotate_secret)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/applications/:app_id",
            delete(handlers::application::delete_application)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // Actions/Hooks Management - Protected (require tenant admin)
        .route(
            "/api/organizations/:org_id/actions",
            post(handlers::action::create_action)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/actions",
            get(handlers::action::list_actions)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/actions/:action_id",
            get(handlers::action::get_action)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/actions/:action_id",
            axum::routing::put(handlers::action::update_action)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/actions/:action_id",
            delete(handlers::action::delete_action)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/actions/:action_id/test",
            post(handlers::action::test_action)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/actions/:action_id/executions",
            get(handlers::action::get_action_executions)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/actions/executions",
            get(handlers::action::get_organization_executions)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // Webhooks Management - Protected (require tenant admin)
        .route(
            "/api/organizations/:org_id/webhooks",
            get(handlers::webhook::list_webhooks)
                .post(handlers::webhook::create_webhook)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/webhooks/:webhook_id",
            get(handlers::webhook::get_webhook)
                .put(handlers::webhook::update_webhook)
                .delete(handlers::webhook::delete_webhook)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/webhooks/:webhook_id/rotate-secret",
            post(handlers::webhook::rotate_secret)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/webhooks/:webhook_id/test",
            post(handlers::webhook::test_webhook)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/webhooks/:webhook_id/deliveries",
            get(handlers::webhook::list_deliveries)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/webhooks/:webhook_id/deliveries/:delivery_id",
            get(handlers::webhook::get_delivery)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/webhooks/:webhook_id/deliveries/:delivery_id/retry",
            post(handlers::webhook::retry_delivery)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        // Webhook Event Types - Public (anyone can see available events)
        .route("/api/webhooks/event-types", get(handlers::webhook::list_event_types))
        // SCIM 2.0 Provisioning - Public endpoints (authenticated via bearer token)
        .route("/scim/v2/ServiceProviderConfig", get(handlers::scim::get_service_provider_config))
        .route("/scim/v2/ResourceTypes", get(handlers::scim::get_resource_types))
        .route("/scim/v2/Schemas", get(handlers::scim::get_schemas))
        // SCIM Users
        .route("/scim/v2/Users", get(handlers::scim::list_users).post(handlers::scim::create_user))
        .route(
            "/scim/v2/Users/:user_id",
            get(handlers::scim::get_user)
                .put(handlers::scim::replace_user)
                .patch(handlers::scim::patch_user)
                .delete(handlers::scim::delete_user)
        )
        // SCIM Groups
        .route("/scim/v2/Groups", get(handlers::scim::list_groups).post(handlers::scim::create_group))
        .route(
            "/scim/v2/Groups/:group_id",
            get(handlers::scim::get_group)
                .patch(handlers::scim::patch_group)
                .delete(handlers::scim::delete_group)
        )
        // SCIM Token Management - Protected (require tenant admin)
        .route(
            "/api/organizations/:org_id/scim/tokens",
            get(handlers::scim::list_scim_tokens)
                .post(handlers::scim::create_scim_token)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .route(
            "/api/organizations/:org_id/scim/tokens/:token_id",
            delete(handlers::scim::revoke_scim_token)
                .route_layer(from_fn_with_state(state.clone(), middleware::require_tenant_admin))
        )
        .with_state(state)
}
