use crate::error::{AuthError, Result};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone)]
pub enum SmsProvider {
    /// SMPP SMS provider
    Smpp {
        host: String,
        port: u16,
        system_id: String,
        password: String,
    },
    /// Twilio SMS provider
    Twilio {
        account_sid: String,
        auth_token: String,
        from_number: String,
    },
    /// AWS SNS provider
    AwsSns {
        region: String,
        access_key_id: String,
        secret_access_key: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub to: String,
    pub message: String,
}

pub struct SmsService {
    provider: SmsProvider,
}

impl SmsService {
    pub fn new(provider: SmsProvider) -> Self {
        Self { provider }
    }

    pub fn from_env() -> Result<Self> {
        let sms_provider = std::env::var("SMS_PROVIDER").unwrap_or_else(|_| "smpp".to_string());

        let provider = match sms_provider.as_str() {
            "smpp" => SmsProvider::Smpp {
                host: std::env::var("SMPP_HOST")
                    .map_err(|_| AuthError::Internal("SMPP_HOST not configured".to_string()))?,
                port: std::env::var("SMPP_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(2775),
                system_id: std::env::var("SMPP_SYSTEM_ID")
                    .map_err(|_| AuthError::Internal("SMPP_SYSTEM_ID not configured".to_string()))?,
                password: std::env::var("SMPP_PASSWORD")
                    .map_err(|_| AuthError::Internal("SMPP_PASSWORD not configured".to_string()))?,
            },
            "twilio" => SmsProvider::Twilio {
                account_sid: std::env::var("TWILIO_ACCOUNT_SID")
                    .map_err(|_| AuthError::Internal("TWILIO_ACCOUNT_SID not configured".to_string()))?,
                auth_token: std::env::var("TWILIO_AUTH_TOKEN")
                    .map_err(|_| AuthError::Internal("TWILIO_AUTH_TOKEN not configured".to_string()))?,
                from_number: std::env::var("TWILIO_FROM_NUMBER")
                    .map_err(|_| AuthError::Internal("TWILIO_FROM_NUMBER not configured".to_string()))?,
            },
            "aws_sns" => SmsProvider::AwsSns {
                region: std::env::var("AWS_REGION")
                    .map_err(|_| AuthError::Internal("AWS_REGION not configured".to_string()))?,
                access_key_id: std::env::var("AWS_ACCESS_KEY_ID")
                    .map_err(|_| AuthError::Internal("AWS_ACCESS_KEY_ID not configured".to_string()))?,
                secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY")
                    .map_err(|_| AuthError::Internal("AWS_SECRET_ACCESS_KEY not configured".to_string()))?,
            },
            _ => return Err(AuthError::Internal(format!("Unknown SMS provider: {}", sms_provider))),
        };

        Ok(Self { provider })
    }

    pub async fn send(&self, sms: SmsMessage) -> Result<()> {
        match &self.provider {
            SmsProvider::Smpp {
                host,
                port,
                system_id,
                password,
            } => {
                // Connect to SMPP gateway
                let addr = format!("{}:{}", host, port);
                tracing::debug!("Connecting to SMPP gateway at {}", addr);

                let mut stream = TcpStream::connect(&addr)
                    .await
                    .map_err(|e| AuthError::Internal(format!("Failed to connect to SMPP gateway: {}", e)))?;

                // SMPP Command IDs
                const BIND_TRANSMITTER: u32 = 0x00000002;
                const BIND_TRANSMITTER_RESP: u32 = 0x80000002;
                const SUBMIT_SM: u32 = 0x00000004;
                const SUBMIT_SM_RESP: u32 = 0x80000004;
                const UNBIND: u32 = 0x00000006;

                // Build and send BIND_TRANSMITTER PDU
                let mut bind_pdu = Vec::new();
                bind_pdu.extend_from_slice(&[0u8; 4]); // Length placeholder
                bind_pdu.extend_from_slice(&BIND_TRANSMITTER.to_be_bytes());
                bind_pdu.extend_from_slice(&[0u8; 4]); // Status = 0
                bind_pdu.extend_from_slice(&1u32.to_be_bytes()); // Sequence number
                bind_pdu.extend_from_slice(system_id.as_bytes());
                bind_pdu.push(0); // Null terminator
                bind_pdu.extend_from_slice(password.as_bytes());
                bind_pdu.push(0); // Null terminator
                bind_pdu.extend_from_slice(b"CIAM\0"); // System type
                bind_pdu.push(0x34); // Interface version
                bind_pdu.push(0); // TON
                bind_pdu.push(0); // NPI
                bind_pdu.push(0); // Address range (null)

                // Update length
                let length = bind_pdu.len() as u32;
                bind_pdu[0..4].copy_from_slice(&length.to_be_bytes());

                stream.write_all(&bind_pdu).await
                    .map_err(|e| AuthError::Internal(format!("Failed to send BIND: {}", e)))?;

                // Read bind response
                let mut header = [0u8; 16];
                stream.read_exact(&mut header).await
                    .map_err(|e| AuthError::Internal(format!("Failed to read BIND response: {}", e)))?;

                let resp_length = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
                let resp_cmd = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
                let resp_status = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);

                if resp_cmd != BIND_TRANSMITTER_RESP || resp_status != 0 {
                    return Err(AuthError::Internal(format!("SMPP bind failed: status={}", resp_status)));
                }

                // Read rest of bind response body
                if resp_length > 16 {
                    let mut body = vec![0u8; (resp_length - 16) as usize];
                    stream.read_exact(&mut body).await.ok();
                }

                tracing::debug!("Successfully bound to SMPP gateway");

                // Build and send SUBMIT_SM PDU
                let destination = sms.to.trim_start_matches('+');
                let message_bytes = sms.message.as_bytes();

                let mut submit_pdu = Vec::new();
                submit_pdu.extend_from_slice(&[0u8; 4]); // Length placeholder
                submit_pdu.extend_from_slice(&SUBMIT_SM.to_be_bytes());
                submit_pdu.extend_from_slice(&[0u8; 4]); // Status
                submit_pdu.extend_from_slice(&2u32.to_be_bytes()); // Sequence number
                submit_pdu.push(0); // Service type (null)
                submit_pdu.push(0); // Source TON
                submit_pdu.push(0); // Source NPI
                submit_pdu.push(0); // Source address (null)
                submit_pdu.push(1); // Dest TON (international)
                submit_pdu.push(1); // Dest NPI (ISDN)
                submit_pdu.extend_from_slice(destination.as_bytes());
                submit_pdu.push(0); // Null terminator
                submit_pdu.push(0); // ESM class
                submit_pdu.push(0); // Protocol ID
                submit_pdu.push(0); // Priority
                submit_pdu.push(0); // Schedule delivery time (null)
                submit_pdu.push(0); // Validity period (null)
                submit_pdu.push(0); // Registered delivery
                submit_pdu.push(0); // Replace if present
                submit_pdu.push(0); // Data coding (default)
                submit_pdu.push(0); // SM default msg ID
                submit_pdu.push(message_bytes.len() as u8); // SM length
                submit_pdu.extend_from_slice(message_bytes);

                // Update length
                let length = submit_pdu.len() as u32;
                submit_pdu[0..4].copy_from_slice(&length.to_be_bytes());

                stream.write_all(&submit_pdu).await
                    .map_err(|e| AuthError::Internal(format!("Failed to send SUBMIT_SM: {}", e)))?;

                // Read submit response
                let mut submit_header = [0u8; 16];
                stream.read_exact(&mut submit_header).await
                    .map_err(|e| AuthError::Internal(format!("Failed to read SUBMIT_SM response: {}", e)))?;

                let submit_length = u32::from_be_bytes([submit_header[0], submit_header[1], submit_header[2], submit_header[3]]);
                let submit_cmd = u32::from_be_bytes([submit_header[4], submit_header[5], submit_header[6], submit_header[7]]);
                let submit_status = u32::from_be_bytes([submit_header[8], submit_header[9], submit_header[10], submit_header[11]]);

                if submit_cmd != SUBMIT_SM_RESP || submit_status != 0 {
                    return Err(AuthError::Internal(format!("SMPP submit failed: status={}", submit_status)));
                }

                // Read rest of submit response body (message ID)
                if submit_length > 16 {
                    let mut body = vec![0u8; (submit_length - 16) as usize];
                    stream.read_exact(&mut body).await.ok();
                }

                // Send UNBIND
                let mut unbind_pdu = Vec::new();
                unbind_pdu.extend_from_slice(&16u32.to_be_bytes()); // Length
                unbind_pdu.extend_from_slice(&UNBIND.to_be_bytes());
                unbind_pdu.extend_from_slice(&[0u8; 4]); // Status
                unbind_pdu.extend_from_slice(&3u32.to_be_bytes()); // Sequence

                stream.write_all(&unbind_pdu).await.ok();

                tracing::info!("SMS sent to {} via SMPP gateway", sms.to);
                Ok(())
            }
            SmsProvider::Twilio {
                account_sid,
                auth_token,
                from_number,
            } => {
                // Send via Twilio API
                let client = reqwest::Client::new();
                let url = format!(
                    "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
                    account_sid
                );

                let response = client
                    .post(&url)
                    .basic_auth(account_sid, Some(auth_token))
                    .form(&[
                        ("To", sms.to.as_str()),
                        ("From", from_number.as_str()),
                        ("Body", sms.message.as_str()),
                    ])
                    .send()
                    .await
                    .map_err(|e| AuthError::Internal(format!("Failed to send SMS via Twilio: {}", e)))?;

                if !response.status().is_success() {
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(AuthError::Internal(format!(
                        "Twilio API error: {}",
                        error_text
                    )));
                }

                tracing::info!("SMS sent to {} via Twilio", sms.to);
                Ok(())
            }
            SmsProvider::AwsSns { .. } => {
                // TODO: Implement AWS SNS integration
                Err(AuthError::Internal(
                    "AWS SNS SMS provider not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Send an OTP code via SMS
    pub async fn send_otp(&self, phone_number: &str, code: &str) -> Result<()> {
        let message = format!(
            "Your verification code is: {}\n\nThis code will expire in 10 minutes.\n\nIf you didn't request this code, please ignore this message.",
            code
        );

        self.send(SmsMessage {
            to: phone_number.to_string(),
            message,
        })
        .await
    }

    /// Test connection to SMS service
    pub async fn test_connection(&self) -> Result<()> {
        match &self.provider {
            SmsProvider::Smpp {
                host,
                port,
                system_id,
                password,
            } => {
                // Test connection by attempting to bind
                let addr = format!("{}:{}", host, port);
                tracing::debug!("Testing SMPP connection to {}", addr);

                let mut stream = TcpStream::connect(&addr)
                    .await
                    .map_err(|e| AuthError::Internal(format!("SMPP connection test failed: {}", e)))?;

                const BIND_TRANSMITTER: u32 = 0x00000002;
                const BIND_TRANSMITTER_RESP: u32 = 0x80000002;
                const UNBIND: u32 = 0x00000006;

                // Build BIND_TRANSMITTER PDU
                let mut bind_pdu = Vec::new();
                bind_pdu.extend_from_slice(&[0u8; 4]); // Length placeholder
                bind_pdu.extend_from_slice(&BIND_TRANSMITTER.to_be_bytes());
                bind_pdu.extend_from_slice(&[0u8; 4]); // Status
                bind_pdu.extend_from_slice(&1u32.to_be_bytes()); // Sequence
                bind_pdu.extend_from_slice(system_id.as_bytes());
                bind_pdu.push(0);
                bind_pdu.extend_from_slice(password.as_bytes());
                bind_pdu.push(0);
                bind_pdu.extend_from_slice(b"CIAM\0");
                bind_pdu.push(0x34);
                bind_pdu.push(0);
                bind_pdu.push(0);
                bind_pdu.push(0);

                let length = bind_pdu.len() as u32;
                bind_pdu[0..4].copy_from_slice(&length.to_be_bytes());

                stream.write_all(&bind_pdu).await
                    .map_err(|e| AuthError::Internal(format!("SMPP bind test failed: {}", e)))?;

                // Read response
                let mut header = [0u8; 16];
                stream.read_exact(&mut header).await
                    .map_err(|e| AuthError::Internal(format!("SMPP bind response failed: {}", e)))?;

                let resp_cmd = u32::from_be_bytes([header[4], header[5], header[6], header[7]]);
                let resp_status = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);

                if resp_cmd != BIND_TRANSMITTER_RESP || resp_status != 0 {
                    return Err(AuthError::Internal(format!("SMPP bind test failed: status={}", resp_status)));
                }

                // Send UNBIND
                let mut unbind_pdu = Vec::new();
                unbind_pdu.extend_from_slice(&16u32.to_be_bytes());
                unbind_pdu.extend_from_slice(&UNBIND.to_be_bytes());
                unbind_pdu.extend_from_slice(&[0u8; 4]);
                unbind_pdu.extend_from_slice(&2u32.to_be_bytes());

                stream.write_all(&unbind_pdu).await.ok();

                tracing::info!("SMPP connection test successful");
                Ok(())
            }
            SmsProvider::Twilio {
                account_sid,
                auth_token,
                ..
            } => {
                // Test Twilio credentials by fetching account info
                let client = reqwest::Client::new();
                let url = format!(
                    "https://api.twilio.com/2010-04-01/Accounts/{}.json",
                    account_sid
                );

                let response = client
                    .get(&url)
                    .basic_auth(account_sid, Some(auth_token))
                    .send()
                    .await
                    .map_err(|e| AuthError::Internal(format!("Twilio connection test failed: {}", e)))?;

                if !response.status().is_success() {
                    return Err(AuthError::Internal("Twilio authentication failed".to_string()));
                }

                tracing::info!("Twilio SMS service connection test successful");
                Ok(())
            }
            SmsProvider::AwsSns { .. } => {
                Err(AuthError::Internal(
                    "AWS SNS connection test not yet implemented".to_string(),
                ))
            }
        }
    }
}
