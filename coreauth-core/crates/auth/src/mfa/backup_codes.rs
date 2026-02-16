use crate::error::Result;
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::{distributions::Alphanumeric, Rng};

const CODE_LENGTH: usize = 8;
const CODE_COUNT: usize = 10;

/// Generate a set of backup recovery codes
/// Returns plaintext codes that should be shown to the user once
pub fn generate_backup_codes() -> Vec<String> {
    let mut rng = rand::thread_rng();
    (0..CODE_COUNT)
        .map(|_| {
            let code: String = (0..CODE_LENGTH)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            // Format as XXXX-XXXX for readability
            format!("{}-{}", &code[..4], &code[4..])
        })
        .collect()
}

/// Hash a backup code for secure storage
/// Uses Argon2 with salt
pub fn hash_backup_code(code: &str) -> Result<String> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();

    // Remove hyphen before hashing
    let code_clean = code.replace('-', "");

    let password_hash = argon2
        .hash_password(code_clean.as_bytes(), &salt)
        .map_err(|e| crate::error::AuthError::from(e))?;

    Ok(password_hash.to_string())
}

/// Verify a backup code against its hash
pub fn verify_backup_code(code: &str, hash: &str) -> Result<bool> {
    use argon2::password_hash::{PasswordHash, PasswordVerifier};

    // Remove hyphen before verifying
    let code_clean = code.replace('-', "");

    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| crate::error::AuthError::PasswordHashError(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(code_clean.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_backup_codes() {
        let codes = generate_backup_codes();
        assert_eq!(codes.len(), CODE_COUNT);

        for code in &codes {
            // Format: XXXX-XXXX
            assert_eq!(code.len(), 9);
            assert_eq!(code.chars().nth(4), Some('-'));
            assert!(code.chars().all(|c| c.is_alphanumeric() || c == '-'));
        }
    }

    #[test]
    fn test_hash_and_verify_backup_code() {
        let code = "ABCD-EFGH";
        let hash = hash_backup_code(code).unwrap();

        assert!(verify_backup_code(code, &hash).unwrap());
        assert!(!verify_backup_code("ABCD-EFGI", &hash).unwrap());
    }

    #[test]
    fn test_code_without_hyphen() {
        let code = "ABCD-EFGH";
        let hash = hash_backup_code(code).unwrap();

        // Should work with or without hyphen
        assert!(verify_backup_code("ABCDEFGH", &hash).unwrap());
        assert!(verify_backup_code("ABCD-EFGH", &hash).unwrap());
    }
}
