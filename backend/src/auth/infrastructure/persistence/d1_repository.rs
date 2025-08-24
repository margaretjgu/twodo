use async_trait::async_trait;
use uuid::Uuid;
use worker::*;
use serde_json::Value;
use chrono::{DateTime, Utc};

use crate::auth::domain::user::User;
use crate::auth::domain::ports::UserRepository;

pub struct D1UserRepository {
    db: D1Database,
}

impl D1UserRepository {
    pub fn new(db: D1Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for D1UserRepository {
    async fn create_user(&self, user: &User) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let stmt = self.db.prepare("INSERT INTO users (id, username, password_hash, created_at) VALUES (?, ?, ?, ?)");
        
        stmt.bind(&[
            user.id.to_string().into(),
            user.username.clone().into(),
            user.password_hash.clone().into(),
            user.created_at.to_rfc3339().into(),
        ])
        .map_err(|e| format!("Bind error: {}", e))?
        .run()
        .await
        .map_err(|e| format!("Run error: {}", e))?;
        
        Ok(())
    }

    async fn get_user_by_username(&self, username: &str) -> std::result::Result<Option<User>, Box<dyn std::error::Error>> {
        let stmt = self.db.prepare("SELECT id, username, password_hash, created_at FROM users WHERE username = ?");
        
        let result = stmt.bind(&[username.into()])
            .map_err(|e| format!("Bind error: {}", e))?
            .first::<Value>(None)
            .await
            .map_err(|e| format!("Query error: {}", e))?;
        
        if let Some(row) = result {
            let user = User {
                id: Uuid::parse_str(row["id"].as_str().ok_or("Invalid user ID")?)
                    .map_err(|e| format!("UUID parse error: {}", e))?,
                username: row["username"].as_str().ok_or("Invalid username")?.to_string(),
                password_hash: row["password_hash"].as_str().ok_or("Invalid password hash")?.to_string(),
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().ok_or("Invalid created_at")?)
                    .map_err(|e| format!("DateTime parse error: {}", e))?
                    .with_timezone(&Utc),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn get_user_by_id(&self, user_id: &Uuid) -> std::result::Result<Option<User>, Box<dyn std::error::Error>> {
        let stmt = self.db.prepare("SELECT id, username, password_hash, created_at FROM users WHERE id = ?");
        
        let result = stmt.bind(&[user_id.to_string().into()])
            .map_err(|e| format!("Bind error: {}", e))?
            .first::<Value>(None)
            .await
            .map_err(|e| format!("Query error: {}", e))?;
        
        if let Some(row) = result {
            let user = User {
                id: Uuid::parse_str(row["id"].as_str().ok_or("Invalid user ID")?)
                    .map_err(|e| format!("UUID parse error: {}", e))?,
                username: row["username"].as_str().ok_or("Invalid username")?.to_string(),
                password_hash: row["password_hash"].as_str().ok_or("Invalid password hash")?.to_string(),
                created_at: DateTime::parse_from_rfc3339(row["created_at"].as_str().ok_or("Invalid created_at")?)
                    .map_err(|e| format!("DateTime parse error: {}", e))?
                    .with_timezone(&Utc),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    async fn username_exists(&self, username: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let stmt = self.db.prepare("SELECT 1 FROM users WHERE username = ? LIMIT 1");
        
        let result = stmt.bind(&[username.into()])
            .map_err(|e| format!("Bind error: {}", e))?
            .first::<Value>(None)
            .await
            .map_err(|e| format!("Query error: {}", e))?;
        
        Ok(result.is_some())
    }
}
