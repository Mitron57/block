use std::sync::Arc;

use anyhow::Context;
use axum::http::{header::HeaderName, HeaderValue, Method};
use axum::Router;
use board_server::application::{AuthService, BoardService};
use board_server::config::AppConfig;
use board_server::domain::{BoardRepository, UserRepository};
use board_server::infrastructure::{JwtConfig, PgBoardRepository, PgUserRepository};
use board_server::presentation::{build_router, AppState};
use board_server::presentation::rooms::RoomRegistry;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("board_server=info,tower_http=info")),
        )
        .init();

    let cfg = AppConfig::from_env().context("load config")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await
        .context("connect database")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("run migrations")?;

    let users: Arc<dyn UserRepository> = Arc::new(PgUserRepository::new(pool.clone()));
    let boards_repo: Arc<dyn BoardRepository> = Arc::new(PgBoardRepository::new(pool.clone()));
    let jwt = JwtConfig::new(cfg.jwt_secret.clone(), 72);
    let auth = Arc::new(AuthService::new(users.clone(), jwt));
    let boards = Arc::new(BoardService::new(boards_repo, users));
    let rooms = Arc::new(RoomRegistry::new());

    let state = AppState { auth, boards, rooms };

    let origin = HeaderValue::from_str(&cfg.frontend_origin)
        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:5173"));
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(origin))
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
        ])
        .allow_credentials(false);

    let app = Router::new()
        .merge(build_router(state))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", cfg.host, cfg.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app).await.context("serve")?;
    Ok(())
}
