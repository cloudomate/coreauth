pub mod service;
pub mod templates;

pub use service::{EmailService, EmailProvider, EmailMessage};
pub use templates::EmailBranding;
