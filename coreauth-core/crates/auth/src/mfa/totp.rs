use crate::error::{AuthError, Result};
use base32::Alphabet;
use image::Luma;
use qrcode::QrCode;
use rand::Rng;
use totp_lite::{totp_custom, Sha1};

const TOTP_DIGITS: u32 = 6;
const TOTP_STEP: u64 = 30; // 30 seconds

/// Generate a random secret for TOTP
pub fn generate_secret() -> String {
    let mut rng = rand::thread_rng();
    let secret_bytes: Vec<u8> = (0..20).map(|_| rng.gen()).collect();
    base32::encode(Alphabet::Rfc4648 { padding: false }, &secret_bytes)
}

/// Generate the current TOTP code for a given secret
pub fn generate_totp(secret: &str) -> Result<String> {
    let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
        .ok_or_else(|| AuthError::ValidationError("Invalid secret format".to_string()))?;

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AuthError::Internal(format!("Time error: {}", e)))?
        .as_secs();

    let totp_value = totp_custom::<Sha1>(TOTP_STEP, TOTP_DIGITS, &secret_bytes, time);
    Ok(format!("{:0width$}", totp_value, width = TOTP_DIGITS as usize))
}

/// Verify a TOTP code against a secret
/// Allows a time window of ±1 period (30 seconds) to account for clock drift
pub fn verify_totp(secret: &str, code: &str) -> Result<bool> {
    let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret)
        .ok_or_else(|| AuthError::ValidationError("Invalid secret format".to_string()))?;

    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AuthError::Internal(format!("Time error: {}", e)))?
        .as_secs();

    // Check current time and ±1 period (total 3 windows)
    for time_offset in [-1i64, 0, 1] {
        let check_time = (time as i64 + (time_offset * TOTP_STEP as i64)) as u64;
        let totp_value = totp_custom::<Sha1>(TOTP_STEP, TOTP_DIGITS, &secret_bytes, check_time);
        let expected_code = format!("{:0width$}", totp_value, width = TOTP_DIGITS as usize);

        if constant_time_compare(&expected_code, code) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Generate a TOTP URI for QR code generation (otpauth:// format)
/// This is the format that authenticator apps expect
pub fn generate_totp_uri(secret: &str, account_name: &str, issuer: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits={}&period={}",
        urlencoding::encode(issuer),
        urlencoding::encode(account_name),
        secret,
        urlencoding::encode(issuer),
        TOTP_DIGITS,
        TOTP_STEP
    )
}

/// Generate a QR code image from a TOTP URI
/// Returns PNG image bytes
pub fn generate_qr_code(totp_uri: &str) -> Result<Vec<u8>> {
    let qr = QrCode::new(totp_uri.as_bytes())
        .map_err(|e| AuthError::Internal(format!("QR code generation failed: {}", e)))?;

    // Generate a large QR code (scale of 10)
    let image = qr.render::<Luma<u8>>()
        .min_dimensions(256, 256)
        .build();

    // Convert to PNG bytes
    let mut png_bytes = Vec::new();
    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| AuthError::Internal(format!("PNG encoding failed: {}", e)))?;

    Ok(png_bytes)
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let mut result = 0u8;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = generate_secret();
        assert!(!secret.is_empty());
        assert!(secret.len() >= 32); // Base32 encoded 20 bytes
    }

    #[test]
    fn test_totp_generation() {
        let secret = generate_secret();
        let code = generate_totp(&secret).unwrap();
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_totp_verification() {
        let secret = generate_secret();
        let code = generate_totp(&secret).unwrap();
        assert!(verify_totp(&secret, &code).unwrap());
    }

    #[test]
    fn test_totp_uri_generation() {
        let secret = "JBSWY3DPEHPK3PXP";
        let uri = generate_totp_uri(secret, "user@example.com", "CIAM");
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("secret=JBSWY3DPEHPK3PXP"));
        assert!(uri.contains("issuer=CIAM"));
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("123456", "123456"));
        assert!(!constant_time_compare("123456", "123457"));
        assert!(!constant_time_compare("123456", "12345"));
    }
}
