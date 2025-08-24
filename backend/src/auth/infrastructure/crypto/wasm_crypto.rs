// WASM-compatible crypto services using sha2 crate
use async_trait::async_trait;
use uuid::Uuid;
use std::error::Error;
use base64::{Engine as _, engine::general_purpose};
use getrandom::getrandom;
use sha2::{Sha256, Digest};

use crate::auth::domain::user::{HashedPassword, JwtClaims};
use crate::auth::domain::ports::{PasswordService, TokenService};

pub struct WasmPasswordService;

impl WasmPasswordService {
    pub fn new() -> Self {
        Self
    }

    // Generate a random salt using getrandom (WASM compatible)
    fn generate_salt() -> Result<String, Box<dyn Error>> {
        let mut salt_bytes = [0u8; 32];
        getrandom(&mut salt_bytes).map_err(|e| format!("Failed to generate random bytes: {}", e))?;
        Ok(general_purpose::STANDARD.encode(salt_bytes))
    }

    // Simple hash using SHA-256 via sha2 crate (WASM compatible)
    fn hash_password_internal(password: &str, salt: &str) -> Result<HashedPassword, Box<dyn Error>> {
        // Combine password and salt
        let combined = format!("{}{}", password, salt);
        
        // Hash using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let result = hasher.finalize();
        let hash = general_purpose::STANDARD.encode(result);
        
        Ok(HashedPassword {
            hash,
            salt: salt.to_string(),
        })
    }

    // Verify password against hash
    fn verify_password_internal(password: &str, stored: &HashedPassword) -> Result<bool, Box<dyn Error>> {
        let computed = Self::hash_password_internal(password, &stored.salt)?;
        Ok(computed.hash == stored.hash)
    }
}

#[async_trait]
impl PasswordService for WasmPasswordService {
    async fn hash_password(&self, password: &str) -> Result<HashedPassword, Box<dyn Error>> {
        let salt = Self::generate_salt()?;
        Self::hash_password_internal(password, &salt)
    }

    async fn verify_password(&self, password: &str, stored: &HashedPassword) -> Result<bool, Box<dyn Error>> {
        Self::verify_password_internal(password, stored)
    }
}

pub struct WasmTokenService {
    secret: String,
}

impl WasmTokenService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    // Simple HMAC-SHA256 using sha2 crate (WASM compatible)
    fn hmac_sha256(data: &str, key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let key_bytes = key.as_bytes();
        let mut hasher = Sha256::new();
        hasher.update(key_bytes);
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        Ok(result.to_vec())
    }

    // Generate JWT token using HMAC-SHA256
    fn generate_jwt_internal(claims: &JwtClaims, secret: &str) -> Result<String, Box<dyn Error>> {
        let header = r#"{"alg":"HS256","typ":"JWT"}"#;
        let payload = serde_json::to_string(claims)?;
        
        // Base64 encode header and payload
        let encoded_header = general_purpose::URL_SAFE_NO_PAD.encode(header);
        let encoded_payload = general_purpose::URL_SAFE_NO_PAD.encode(&payload);
        
        // Create signature base
        let signature_base = format!("{}.{}", encoded_header, encoded_payload);
        
        // Generate signature
        let signature_bytes = Self::hmac_sha256(&signature_base, secret)?;
        let encoded_signature = general_purpose::URL_SAFE_NO_PAD.encode(signature_bytes);
        
        // Combine all parts
        Ok(format!("{}.{}.{}", encoded_header, encoded_payload, encoded_signature))
    }

    // Get current timestamp in seconds (WASM compatible)
    fn current_timestamp() -> u64 {
        // Use JavaScript Date.now() for better WASM compatibility
        (js_sys::Date::now() / 1000.0) as u64
    }
}

#[async_trait]
impl TokenService for WasmTokenService {
    async fn generate_token(&self, user_id: &Uuid, username: &str) -> Result<String, Box<dyn Error>> {
        let now = Self::current_timestamp();
        let exp = now + (24 * 60 * 60); // 24 hours from now
        
        let claims = JwtClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            iat: now,
            exp,
        };
        
        Self::generate_jwt_internal(&claims, &self.secret)
    }

    async fn validate_token(&self, _token: &str) -> Result<JwtClaims, Box<dyn Error>> {
        // TODO: Implement JWT validation
        Err("JWT validation not implemented yet".into())
    }
}
