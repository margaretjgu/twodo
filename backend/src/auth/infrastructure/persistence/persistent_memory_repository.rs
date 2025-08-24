use async_trait::async_trait;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

use crate::auth::domain::user::User;
use crate::auth::domain::ports::UserRepository;
use std::error::Error;

// Global shared storage that persists across requests
static GLOBAL_USER_STORE: Lazy<Arc<Mutex<HashMap<String, User>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub struct PersistentMemoryUserRepository;

impl PersistentMemoryUserRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UserRepository for PersistentMemoryUserRepository {
    async fn create_user(&self, user: &User) -> Result<(), Box<dyn Error>> {
        let mut users = GLOBAL_USER_STORE.lock().unwrap();
        users.insert(user.username.clone(), user.clone());
        Ok(())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Box<dyn Error>> {
        let users = GLOBAL_USER_STORE.lock().unwrap();
        Ok(users.get(username).cloned())
    }

    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>, Box<dyn Error>> {
        let users = GLOBAL_USER_STORE.lock().unwrap();
        Ok(users.values().find(|u| &u.id == user_id).cloned())
    }

    async fn username_exists(&self, username: &str) -> Result<bool, Box<dyn Error>> {
        let users = GLOBAL_USER_STORE.lock().unwrap();
        Ok(users.contains_key(username))
    }
}
