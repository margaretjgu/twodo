use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::auth::domain::user::{User, UserRegistration, UserLogin, AuthResult, UserInfo};
use crate::auth::domain::ports::{UserRepository, PasswordService, TokenService};
use std::error::Error;

pub struct AuthService {
    user_repository: Arc<dyn UserRepository>,
    password_service: Arc<dyn PasswordService>,
    token_service: Arc<dyn TokenService>,
}

impl AuthService {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        password_service: Arc<dyn PasswordService>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            user_repository,
            password_service,
            token_service,
        }
    }

    pub async fn register(&self, registration: UserRegistration) -> Result<UserInfo, Box<dyn Error>> {
        // Validate input
        if registration.username.len() < 3 || registration.username.len() > 50 {
            return Err("Username must be between 3 and 50 characters".into());
        }
        if registration.password.len() < 8 {
            return Err("Password must be at least 8 characters".into());
        }

        // Check if user already exists
        if self.user_repository.username_exists(&registration.username).await? {
            return Err("User already exists".into());
        }

        // Hash password
        let hashed_password = self.password_service.hash_password(&registration.password).await?;
        let password_hash = serde_json::to_string(&hashed_password)?;

        // Create user
        let user = User {
            id: Uuid::new_v4(),
            username: registration.username.clone(),
            password_hash,
            created_at: Utc::now(),
        };

        // Save user
        self.user_repository.create_user(&user).await?;

        Ok(UserInfo {
            id: user.id.to_string(),
            username: user.username,
        })
    }

    pub async fn login(&self, login: UserLogin) -> Result<AuthResult, Box<dyn Error>> {
        // Find user
        let user = self
            .user_repository
            .get_user_by_username(&login.username)
            .await?
            .ok_or("Invalid credentials")?;

        // Parse stored password hash
        let stored_password = serde_json::from_str(&user.password_hash)
            .map_err(|_| "Invalid password data")?;

        // Verify password
        if !self.password_service.verify_password(&login.password, &stored_password).await? {
            return Err("Invalid credentials".into());
        }

        // Generate token
        let token = self.token_service.generate_token(&user.id, &user.username).await?;

        Ok(AuthResult {
            user: UserInfo {
                id: user.id.to_string(),
                username: user.username,
            },
            token,
        })
    }

    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<UserInfo>, Box<dyn Error>> {
        if let Some(user) = self.user_repository.get_user_by_id(user_id).await? {
            Ok(Some(UserInfo {
                id: user.id.to_string(),
                username: user.username,
            }))
        } else {
            Ok(None)
        }
    }
}
