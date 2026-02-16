// Core modules
pub mod organization;
pub mod tenant;  // Kept for backward compatibility (re-exports organization types)
pub mod user;
pub mod role;
pub mod permission;
pub mod session;
pub mod oauth;
pub mod oidc_provider;
pub mod mfa;
pub mod audit;
pub mod organization_member;
pub mod connection;

// New modules for multi-tenant hierarchy
pub mod application;
pub mod action;

// Billing module
pub mod billing;

// OAuth2/OIDC Authorization Server module
pub mod oauth2;

// Webhooks module
pub mod webhook;

// SCIM provisioning module
pub mod scim;

// Groups module
pub mod group;

// Passwordless authentication module
pub mod passwordless;

// Self-service flow module
pub mod self_service;

// Re-export commonly used types
pub use organization::{
    Organization, CreateOrganization, UpdateOrganization,
    OrganizationSettings, IsolationMode, BrandingSettings,
    SecuritySettings, FeatureFlags,
};
pub use tenant::{Tenant, NewTenant, TenantSettings};  // Backward compatibility aliases
pub use user::{User, NewUser, UserMetadata};
pub use role::{Role, NewRole};
pub use permission::{Permission, NewPermission};
pub use session::{Session, NewSession};
pub use oauth::{OAuthClient, NewOAuthClient};
pub use oidc_provider::{OidcProvider, NewOidcProvider, ClaimMappings, OAuthState, IdTokenClaims};
pub use mfa::{MfaMethod, MfaMethodType};
pub use audit::{AuditLog, CreateAuditLog, AuditEventCategory, AuditStatus, AuditLogBuilder, AuditLogQuery};
pub use organization_member::{OrganizationMember, AddOrganizationMember, UpdateMemberRole, OrganizationMemberWithUser};
pub use connection::{
    Connection, CreateConnection, UpdateConnection, ConnectionScope,
    OidcConnectionConfig, SamlConnectionConfig, DatabaseConnectionConfig,
    SocialConnectionConfig, SocialProvider, SocialUserInfo,
};
pub use application::{
    Application, ApplicationWithSecret, CreateApplication, UpdateApplication,
    ApplicationType,
};
pub use action::{
    Action, CreateAction, UpdateAction, ActionTrigger,
    ActionExecution, ExecutionStatus, ActionContext, ActionResult,
};
pub use billing::{
    Plan, PlanFeatures, Subscription, SubscriptionWithPlan, SubscriptionStatus,
    BillingCycle, CreateSubscription, UpdateSubscription, UsageRecord, ActiveUser,
    UsageSummary, PlanLimits, Invoice, InvoiceStatus, PaymentMethod, BillingEvent,
    CreateCheckoutRequest, CheckoutResponse, BillingPortalResponse, BillingOverview,
    ChangePlanRequest,
};
pub use oauth2::{
    AuthorizationCode, CreateAuthorizationCode, RefreshToken, CreateRefreshToken,
    AccessTokenRecord, OAuthConsent, LoginSession, CreateLoginSession,
    AuthorizationRequest, CreateAuthorizationRequest, SigningKey,
    OidcDiscovery, Jwks, Jwk, TokenRequest, TokenResponse, TokenError,
    UserInfoResponse, AuthorizeRequest, IntrospectionRequest, IntrospectionResponse,
    RevocationRequest,
};
pub use webhook::{
    Webhook, CreateWebhook, UpdateWebhook, WebhookResponse, WebhookWithSecretResponse,
    WebhookDelivery, DeliveryStatus, WebhookEvent, WebhookEventType, WebhookPayload,
    RetryPolicy, TestWebhookRequest, TestWebhookResponse, DeliverySummary, DeliveryQuery,
};
pub use scim::{
    ScimUser, ScimGroup, ScimName, ScimEmail, ScimPhoneNumber, ScimMember, ScimGroupRef,
    ScimMeta, ScimListResponse, ScimError, ScimPatchRequest, ScimPatchOp,
    ServiceProviderConfig, ResourceType, ScimToken, CreateScimToken, ScimTokenResponse,
    ScimGroupRecord, ScimGroupMember, ScimConfiguration, ScimProvisioningLog,
    ScimListQuery, ScimFilter, ScimFilterOp, CreateScimUser, CreateScimGroup,
};
pub use group::{
    Group, GroupMember, GroupMemberWithUser, GroupWithMemberCount,
    CreateGroup, UpdateGroup, AddGroupMember, UpdateGroupMember, GroupRole,
};
pub use passwordless::{
    PasswordlessToken, PasswordlessTokenType, PasswordlessStartRequest, PasswordlessStartResponse,
    PasswordlessVerifyRequest, PasswordlessVerifyResponse, PasswordlessUserInfo,
    WebAuthnCredential, WebAuthnChallenge, WebAuthnRegisterStartRequest, WebAuthnRegisterStartResponse,
    WebAuthnRegisterFinishRequest, WebAuthnAuthStartRequest, WebAuthnAuthStartResponse,
    WebAuthnAuthFinishRequest, TenantRateLimit, UpdateRateLimitRequest, RateLimitsResponse,
};
pub use self_service::{
    SelfServiceFlow, FlowType, DeliveryMethod, FlowState,
    FlowUi, UiNode, UiNodeAttributes, UiMessage, UiNodeMeta, UiLabel,
    LoginFlowSubmit, RegistrationFlowSubmit,
    FlowResponse, SessionResponse, IdentityResponse, AuthMethodRef,
    FlowQuery, CreateFlowQuery,
};
