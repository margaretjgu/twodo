use std::sync::Arc;
use axum::{Router, routing::get};
use tower_http::cors::{CorsLayer, Any};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod auth;
pub mod calendar;
pub mod chores;
pub mod config;
pub mod expenses;
pub mod groups;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Load configuration
    let config = config::Config::from_env();
    
    // Setup logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "twodo_backend=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Initialize repositories
    let user_repo = Arc::new(auth::infrastructure::persistence::in_memory_repository::InMemoryUserRepository::new());
    let calendar_repo = Arc::new(calendar::infrastructure::persistence::in_memory_repository::InMemoryCalendarRepository::new());
    let chore_repo = Arc::new(chores::infrastructure::persistence::in_memory_repository::InMemoryChoreRepository::new());
    let expense_repo = Arc::new(expenses::infrastructure::persistence::in_memory_repository::InMemoryExpenseRepository::new());
    let settlement_repo = Arc::new(expenses::infrastructure::persistence::in_memory_repository::InMemorySettlementRepository::new());
    let group_repo = Arc::new(groups::infrastructure::persistence::in_memory_repository::InMemoryGroupRepository::new());
    let group_invitation_repo = Arc::new(groups::infrastructure::persistence::in_memory_repository::InMemoryGroupInvitationRepository::new());

    // Create services and routers
    let group_service = Arc::new(groups::application::use_cases::GroupService::new(
        group_repo,
        group_invitation_repo,
    ));
    
    let expense_service = Arc::new(expenses::application::use_cases::ExpenseService::new(
        expense_repo,
        settlement_repo,
    ));
    
    let auth_router = auth::infrastructure::web::routes::create_router(
        user_repo,
        config.jwt_secret.clone(),
        config.jwt_expiration_hours,
    );
    let calendar_router = calendar::infrastructure::web::routes::create_router(calendar_repo);
    let chore_router = chores::infrastructure::web::routes::create_router(chore_repo);
    let expense_router = expenses::infrastructure::web::routes::create_router(expense_service);
    let group_router = groups::infrastructure::web::routes::create_router(group_service);

    // Combine routers
    let app = Router::new()
        .route("/", get(|| async { "Welcome to TwoDo API" }))
        .nest("/api/auth", auth_router)
        .nest("/api/calendar", calendar_router)
        .nest("/api/chores", chore_router)
        .nest("/api/expenses", expense_router)
        .nest("/api/groups", group_router)
        .layer(cors);

    // Run the server
    let addr = SocketAddr::from((
        config.host.parse::<std::net::IpAddr>().expect("Invalid host"),
        config.port,
    ));
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
