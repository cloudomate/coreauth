use crate::error::{AuthError, Result};
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum EmailProvider {
    Smtp {
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
        from_email: String,
        from_name: String,
    },
    MailHog {
        host: String,
        port: u16,
        from_email: String,
        from_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub text_body: String,
    pub html_body: Option<String>,
}

#[derive(Clone)]
pub struct EmailService {
    provider: EmailProvider,
}

impl EmailService {
    pub fn new(provider: EmailProvider) -> Self {
        Self { provider }
    }

    pub fn from_env() -> Result<Self> {
        let email_provider = std::env::var("EMAIL_PROVIDER").unwrap_or_else(|_| "mailhog".to_string());

        let provider = match email_provider.as_str() {
            "mailhog" => EmailProvider::MailHog {
                host: std::env::var("MAILHOG_HOST").unwrap_or_else(|_| "localhost".to_string()),
                port: std::env::var("MAILHOG_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(1025),
                from_email: std::env::var("EMAIL_FROM").unwrap_or_else(|_| "noreply@ciam.dev".to_string()),
                from_name: std::env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "CIAM System".to_string()),
            },
            "smtp" => EmailProvider::Smtp {
                host: std::env::var("SMTP_HOST")
                    .map_err(|_| AuthError::Internal("SMTP_HOST not configured".to_string()))?,
                port: std::env::var("SMTP_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(587),
                username: std::env::var("SMTP_USERNAME").ok(),
                password: std::env::var("SMTP_PASSWORD").ok(),
                from_email: std::env::var("EMAIL_FROM")
                    .map_err(|_| AuthError::Internal("EMAIL_FROM not configured".to_string()))?,
                from_name: std::env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "CIAM System".to_string()),
            },
            _ => return Err(AuthError::Internal(format!("Unknown email provider: {}", email_provider))),
        };

        Ok(Self { provider })
    }

    pub async fn send(&self, email: EmailMessage) -> Result<()> {
        match &self.provider {
            EmailProvider::Smtp {
                host,
                port,
                username,
                password,
                from_email,
                from_name,
            } => {
                self.send_smtp(
                    host,
                    *port,
                    username.as_deref(),
                    password.as_deref(),
                    from_email,
                    from_name,
                    email,
                )
                .await
            }
            EmailProvider::MailHog {
                host,
                port,
                from_email,
                from_name,
            } => {
                // MailHog doesn't require authentication
                self.send_smtp(host, *port, None, None, from_email, from_name, email)
                    .await
            }
        }
    }

    async fn send_smtp(
        &self,
        host: &str,
        port: u16,
        username: Option<&str>,
        password: Option<&str>,
        from_email: &str,
        from_name: &str,
        email: EmailMessage,
    ) -> Result<()> {
        // Build the email message
        let from = format!("{} <{}>", from_name, from_email)
            .parse::<Mailbox>()
            .map_err(|e| AuthError::Internal(format!("Invalid from address: {}", e)))?;

        let to = if let Some(name) = &email.to_name {
            format!("{} <{}>", name, email.to)
        } else {
            email.to.clone()
        }
        .parse::<Mailbox>()
        .map_err(|e| AuthError::Internal(format!("Invalid to address: {}", e)))?;

        let message_builder = Message::builder()
            .from(from)
            .to(to)
            .subject(&email.subject);

        // Build multipart message if HTML is provided
        let message = if let Some(html) = &email.html_body {
            message_builder
                .multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(email.text_body.clone()),
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html.clone()),
                        ),
                )
                .map_err(|e| AuthError::Internal(format!("Failed to build email: {}", e)))?
        } else {
            message_builder
                .body(email.text_body.clone())
                .map_err(|e| AuthError::Internal(format!("Failed to build email: {}", e)))?
        };

        // Build SMTP transport
        // For MailHog (port 1025 or 1027), use builder() without TLS
        // For production SMTP (port 587 or 465), use relay() with TLS
        let mailer = if port == 1025 || port == 1027 {
            // MailHog - no TLS, no authentication
            let mut builder = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                .port(port);

            // Add credentials if provided (though MailHog doesn't need them)
            if let (Some(user), Some(pass)) = (username, password) {
                builder = builder.credentials(Credentials::new(
                    user.to_string(),
                    pass.to_string(),
                ));
            }

            builder.build()
        } else {
            // Production SMTP with TLS
            let mut transport_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                .map_err(|e| AuthError::Internal(format!("Failed to create SMTP transport: {}", e)))?
                .port(port);

            // Add credentials if provided
            if let (Some(user), Some(pass)) = (username, password) {
                transport_builder = transport_builder.credentials(Credentials::new(
                    user.to_string(),
                    pass.to_string(),
                ));
            }

            transport_builder.build()
        };

        // Send the email
        mailer
            .send(message)
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to send email: {}", e)))?;

        tracing::info!(
            "Email sent successfully to {} with subject: {}",
            email.to,
            email.subject
        );

        Ok(())
    }

    /// Test connection to email service
    pub async fn test_connection(&self) -> Result<()> {
        match &self.provider {
            EmailProvider::Smtp {
                host,
                port,
                username,
                password,
                ..
            } => {
                let mut transport_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                    .map_err(|e| AuthError::Internal(format!("Failed to create SMTP transport: {}", e)))?
                    .port(*port);

                if let (Some(user), Some(pass)) = (username, password) {
                    transport_builder = transport_builder.credentials(Credentials::new(
                        user.to_string(),
                        pass.to_string(),
                    ));
                }

                let mailer: AsyncSmtpTransport<Tokio1Executor> = transport_builder.build();
                mailer
                    .test_connection()
                    .await
                    .map_err(|e| AuthError::Internal(format!("SMTP connection test failed: {}", e)))?;

                tracing::info!("Email service connection test successful");
                Ok(())
            }
            EmailProvider::MailHog { host, port, .. } => {
                // For MailHog, just check if it's reachable
                let transport_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                    .map_err(|e| AuthError::Internal(format!("Failed to create SMTP transport: {}", e)))?
                    .port(*port);

                let mailer: AsyncSmtpTransport<Tokio1Executor> = transport_builder.build();
                mailer
                    .test_connection()
                    .await
                    .map_err(|e| AuthError::Internal(format!("MailHog connection test failed: {}", e)))?;

                tracing::info!("MailHog connection test successful");
                Ok(())
            }
        }
    }
}
