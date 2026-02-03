use crate::error::{AuthError, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2,
};

pub struct PasswordHasher;

impl PasswordHasher {
    /// Hash a password using Argon2id
    pub fn hash(password: &str) -> Result<String> {
        // Validate password strength
        Self::validate_password(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))?
            .to_string();

        Ok(password_hash)
    }

    /// Verify a password against a hash
    pub fn verify(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthError::PasswordHashError(e.to_string())),
        }
    }

    /// Validate password strength
    fn validate_password(password: &str) -> Result<()> {
        let min_length = std::env::var("PASSWORD_MIN_LENGTH")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8);

        if password.len() < min_length {
            return Err(AuthError::WeakPassword(format!(
                "Password must be at least {} characters",
                min_length
            )));
        }

        // Check for at least one uppercase letter
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        // Check for at least one lowercase letter
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        // Check for at least one digit
        if !password.chars().any(|c| c.is_numeric()) {
            return Err(AuthError::WeakPassword(
                "Password must contain at least one number".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if a password needs rehashing (algorithm params changed)
    pub fn needs_rehash(hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return true, // Invalid hash format, needs rehash
        };

        // Check if algorithm identifier is argon2id
        parsed_hash.algorithm != argon2::Algorithm::Argon2id.ident()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "MySecureP@ssw0rd";
        let hash = PasswordHasher::hash(password).expect("Failed to hash password");

        assert!(PasswordHasher::verify(password, &hash).unwrap());
        assert!(!PasswordHasher::verify("WrongPassword1!", &hash).unwrap());
    }

    #[test]
    fn test_password_validation() {
        // Too short
        assert!(PasswordHasher::hash("Short1!").is_err());

        // No uppercase
        assert!(PasswordHasher::hash("nouppercase1!").is_err());

        // No lowercase
        assert!(PasswordHasher::hash("NOLOWERCASE1!").is_err());

        // No number
        assert!(PasswordHasher::hash("NoNumbers!").is_err());

        // Valid password
        assert!(PasswordHasher::hash("ValidP@ssw0rd").is_ok());
    }
}
