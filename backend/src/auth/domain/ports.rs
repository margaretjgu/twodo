use async_trait::async_trait;
use uuid::Uuid;
use super::user::{User, HashedPassword, JwtClaims};
use std::error::Error;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<(), Box<dyn Error>>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Box<dyn Error>>;
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, Box<dyn Error>>;
    async fn username_exists(&self, username: &str) -> Result<bool, Box<dyn Error>>;
}

#[async_trait]
pub trait PasswordService: Send + Sync {
    async fn hash_password(&self, password: &str) -> Result<HashedPassword, Box<dyn Error>>;
    async fn verify_password(&self, password: &str, hash: &HashedPassword) -> Result<bool, Box<dyn Error>>;
}

#[async_trait] 
pub trait TokenService: Send + Sync {
    async fn generate_token(&self, user_id: &Uuid, username: &str) -> Result<String, Box<dyn Error>>;
    async fn validate_token(&self, token: &str) -> Result<JwtClaims, Box<dyn Error>>;
}
