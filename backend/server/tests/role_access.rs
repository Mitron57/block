//! Integration checks for role enforcement. Run with Postgres:
//! `DATABASE_URL=postgres://board:board@localhost:5432/board JWT_SECRET=integration-test-secret-32chars cargo test --test role_access -- --ignored`

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use board_server::application::{AuthService, BoardService};
use board_server::domain::{BoardRepository, UserRepository};
use board_server::infrastructure::{JwtConfig, PgBoardRepository, PgUserRepository};
use board_server::presentation::{build_router, AppState};
use board_server::presentation::rooms::RoomRegistry;
use http_body_util::BodyExt;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

async fn app_router() -> axum::Router {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL for integration tests");
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "integration-test-secret-32chars!!".into());
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("db connect");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrate");
    sqlx::query(
        "TRUNCATE board_elements, board_members, boards, users RESTART IDENTITY CASCADE",
    )
    .execute(&pool)
    .await
    .expect("truncate");

    let users: Arc<dyn UserRepository> = Arc::new(PgUserRepository::new(pool.clone()));
    let boards_repo: Arc<dyn BoardRepository> = Arc::new(PgBoardRepository::new(pool.clone()));
    let jwt = JwtConfig::new(jwt_secret, 24);
    let auth = Arc::new(AuthService::new(users.clone(), jwt));
    let boards = Arc::new(BoardService::new(boards_repo, users));
    let rooms = Arc::new(RoomRegistry::new());
    let state = AppState { auth, boards, rooms };
    build_router(state)
}

async fn read_body(res: axum::response::Response) -> String {
    let b = res.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8_lossy(&b).into_owned()
}

#[tokio::test]
#[ignore]
async fn viewer_cannot_patch_board() {
    let app = app_router().await;

    let alice_reg = r#"{"email":"alice-int@test.local","password":"password123","display_name":"Alice"}"#;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(alice_reg))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&read_body(res).await).unwrap();
    let alice_token = body["token"].as_str().unwrap();

    let bob_reg = r#"{"email":"bob-int@test.local","password":"password123","display_name":"Bob"}"#;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(bob_reg))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body: serde_json::Value = serde_json::from_str(&read_body(res).await).unwrap();
    let bob_token = body["token"].as_str().unwrap();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/boards")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {alice_token}"))
                .body(Body::from(r#"{"title":"Shared"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let board: serde_json::Value = serde_json::from_str(&read_body(res).await).unwrap();
    let board_id = board["id"].as_str().unwrap();

    let invite = r#"{"email":"bob-int@test.local","role":"viewer"}"#;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/boards/{board_id}/members"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {alice_token}"))
                .body(Body::from(invite))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/boards/{board_id}"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {bob_token}"))
                .body(Body::from(r#"{"title":"Hacked"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[ignore]
async fn viewer_cannot_add_element_via_rest() {
    let app = app_router().await;

    let alice_reg = r#"{"email":"alice2-int@test.local","password":"password123","display_name":"Alice"}"#;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(alice_reg))
                .unwrap(),
        )
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&read_body(res).await).unwrap();
    let alice_token = body["token"].as_str().unwrap();

    let bob_reg = r#"{"email":"bob2-int@test.local","password":"password123","display_name":"Bob"}"#;
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(bob_reg))
                .unwrap(),
        )
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&read_body(res).await).unwrap();
    let bob_token = body["token"].as_str().unwrap();

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/boards")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {alice_token}"))
                .body(Body::from(r#"{"title":"REST"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    let board: serde_json::Value = serde_json::from_str(&read_body(res).await).unwrap();
    let board_id = board["id"].as_str().unwrap();

    let invite = r#"{"email":"bob2-int@test.local","role":"viewer"}"#;
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/boards/{board_id}/members"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {alice_token}"))
                .body(Body::from(invite))
                .unwrap(),
        )
        .await
        .unwrap();

    let payload = r#"{"element_type":"stroke","payload":{"points":[[0,0],[1,1]]},"z_index":0}"#;
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/boards/{board_id}/elements"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {bob_token}"))
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
