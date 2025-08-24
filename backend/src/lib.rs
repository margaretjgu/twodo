// TwoDo Cloudflare Workers Entry Point
use worker::*;
use serde::{Deserialize, Serialize};

// Domain modules with proper hexagonal architecture
pub mod auth;
pub mod groups;
pub mod expenses;
pub mod chores;
pub mod calendar;

// Simple endpoint handlers that create services on-demand
async fn handle_register_endpoint(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    use std::sync::Arc;
    use crate::auth::infrastructure::{WasmPasswordService, WasmTokenService};
    use crate::auth::application::use_cases::AuthService;
    use crate::auth::domain::user::UserRegistration;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    struct AuthPayload {
        username: String,
        password: String,
    }

    #[derive(Serialize)]
    struct ErrorResponse {
        error: String,
    }

    // Create auth service with persistent memory storage
    let user_repository = Arc::new(crate::auth::infrastructure::persistence::persistent_memory_repository::PersistentMemoryUserRepository::new());
    let password_service = Arc::new(WasmPasswordService::new());
    let token_service = Arc::new(WasmTokenService::new("demo-secret".to_string()));
    let auth_service = AuthService::new(user_repository, password_service, token_service);

    // Parse request
    let payload: AuthPayload = match req.json().await {
        Ok(p) => p,
        Err(_) => return Response::from_json(&ErrorResponse {
            error: "Invalid JSON".to_string(),
        }),
    };

    let registration = UserRegistration {
        username: payload.username,
        password: payload.password,
    };

    // Register user
    match auth_service.register(registration).await {
        Ok(user_info) => Response::from_json(&user_info),
        Err(e) => {
            let response = Response::from_json(&ErrorResponse {
                error: e.to_string(),
            })?;
            Ok(response.with_status(400))
        }
    }
}

async fn handle_login_endpoint(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    use std::sync::Arc;
    use crate::auth::infrastructure::{WasmPasswordService, WasmTokenService};
    use crate::auth::application::use_cases::AuthService;
    use crate::auth::domain::user::UserLogin;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    struct AuthPayload {
        username: String,
        password: String,
    }

    #[derive(Serialize)]
    struct ErrorResponse {
        error: String,
    }

    // Create auth service with persistent memory storage
    let user_repository = Arc::new(crate::auth::infrastructure::persistence::persistent_memory_repository::PersistentMemoryUserRepository::new());
    let password_service = Arc::new(WasmPasswordService::new());
    let token_service = Arc::new(WasmTokenService::new("demo-secret".to_string()));
    let auth_service = AuthService::new(user_repository, password_service, token_service);

    // Parse request
    let payload: AuthPayload = match req.json().await {
        Ok(p) => p,
        Err(_) => return Response::from_json(&ErrorResponse {
            error: "Invalid JSON".to_string(),
        }),
    };

    let login = UserLogin {
        username: payload.username,
        password: payload.password,
    };

    // Login user
    match auth_service.login(login).await {
        Ok(auth_result) => Response::from_json(&auth_result),
        Err(e) => {
            let response = Response::from_json(&ErrorResponse {
                error: e.to_string(),
            })?;
            Ok(response.with_status(401))
        }
    }
}

// Future modules - properly structured following hexagonal architecture
// pub mod expenses;
// pub mod groups; 
// pub mod chores;
// pub mod calendar;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();
    
    router
        .get("/", |_, _| Response::ok("🎉 TwoDo API is running on Cloudflare Workers!"))
        .get("/health", handle_health)
        .get("/api/auth/status", handle_auth_status)
        .post_async("/api/auth/register", handle_register_endpoint)
        .post_async("/api/auth/login", handle_login_endpoint)
        .get("/api/expenses/balances/:group_id", handle_get_balances)
        .post("/api/expenses", handle_create_expense)
        .run(req, env)
        .await
}

// Types are now defined in the auth domain module

fn handle_health(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::from_json(&serde_json::json!({
        "status": "healthy",
        "timestamp": js_sys::Date::now(),
        "version": "1.0.0",
        "environment": "production"
    }))
}

fn handle_auth_status(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::from_json(&serde_json::json!({
        "status": "✅ Authentication refactored with proper Hexagonal Architecture",
        "architecture_implemented": {
            "pattern": "Hexagonal Architecture (Ports & Adapters)",
            "structure": "domain → application → infrastructure",
            "separation_of_concerns": "✅ Complete"
        },
        "modules_refactored": {
            "auth/domain/": {
                "user.rs": "✅ User entities, value objects, and DTOs",
                "ports.rs": "✅ Repository and service interfaces (traits)"
            },
            "auth/application/": {
                "use_cases.rs": "✅ AuthService business logic"
            },
            "auth/infrastructure/": {
                "crypto/": "✅ WASM-compatible crypto services",
                "persistence/": "✅ D1 and in-memory repositories", 
                "web/": "✅ HTTP route handlers"
            }
        },
        "benefits_achieved": [
            "🏗️ Modular and testable architecture",
            "🔄 Swappable implementations (D1 ↔ In-memory)",
            "🎯 Clear separation of concerns",
            "📦 Independent deployable modules",
            "🧪 Easy unit testing",
            "🔧 WASM-compatible design"
        ],
        "api_demonstration": {
            "description": "Real authentication APIs implemented following hexagonal principles",
            "endpoints": {
                "/api/auth/register": "POST - User registration with validation",
                "/api/auth/login": "POST - User login with JWT generation"
            },
            "technologies": "Rust + WASM + Cloudflare Workers + D1"
        },
        "version": "2.0.0",
        "live_endpoints": {
            "/api/auth/register": "✅ Working with in-memory storage",
            "/api/auth/login": "✅ Working with in-memory storage"
        }
    }))
}

fn handle_get_balances(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(group_id) = ctx.param("group_id") {
        Response::from_json(&serde_json::json!({
            "group_id": group_id,
            "balances": {},
            "message": "Balance endpoint - implementing next!",
            "status": "todo",
            "version": "1.0.0"
        }))
    } else {
        Response::error("Missing group_id", 400)
    }
}

fn handle_create_expense(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    Response::from_json(&serde_json::json!({
        "message": "Create expense endpoint - implementing next!",
        "status": "todo",
        "version": "1.0.0"
    }))
}
