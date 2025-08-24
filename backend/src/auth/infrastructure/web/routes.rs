use worker::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::application::use_cases::AuthService;
use crate::auth::domain::user::{UserRegistration, UserLogin};

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

// Standalone handlers for direct integration with Workers router
pub async fn handle_register(
    mut req: Request,
    auth_service: Arc<AuthService>,
) -> Result<Response> {
    // Parse request body
    let payload: AuthPayload = match req.json().await {
        Ok(p) => p,
        Err(_) => return Response::from_json(&ErrorResponse {
            error: "Invalid JSON".to_string(),
            details: Some("Request body must be valid JSON with username and password".to_string()),
        }),
    };

    // Create registration request
    let registration = UserRegistration {
        username: payload.username,
        password: payload.password,
    };

    // Register user
    match auth_service.register(registration).await {
        Ok(user_info) => Response::from_json(&user_info),
        Err(e) => {
            let status = if e.to_string().contains("already exists") { 409 } else { 400 };
            let response = Response::from_json(&ErrorResponse {
                error: e.to_string(),
                details: None,
            })?;
            Ok(response.with_status(status))
        }
    }
}

pub async fn handle_login(
    mut req: Request,
    auth_service: Arc<AuthService>,
) -> Result<Response> {
    // Parse request body
    let payload: AuthPayload = match req.json().await {
        Ok(p) => p,
        Err(_) => return Response::from_json(&ErrorResponse {
            error: "Invalid JSON".to_string(),
            details: Some("Request body must be valid JSON with username and password".to_string()),
        }),
    };

    // Create login request
    let login = UserLogin {
        username: payload.username,
        password: payload.password,
    };

    // Authenticate user
    match auth_service.login(login).await {
        Ok(auth_result) => Response::from_json(&auth_result),
        Err(e) => {
            let status = if e.to_string().contains("Invalid credentials") { 401 } else { 400 };
            let response = Response::from_json(&ErrorResponse {
                error: e.to_string(),
                details: None,
            })?;
            Ok(response.with_status(status))
        }
    }
}
