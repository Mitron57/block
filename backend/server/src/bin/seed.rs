//! Load test users/boards. Run with DATABASE_URL (and JWT_SECRET not required).
use std::sync::Arc;

use anyhow::Context;
use board_server::domain::{BoardRepository, BoardRole, UserRepository};
use board_server::infrastructure::{password, PgBoardRepository, PgUserRepository};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("connect")?;

    let users: Arc<dyn UserRepository> = Arc::new(PgUserRepository::new(pool.clone()));
    let boards: Arc<dyn BoardRepository> = Arc::new(PgBoardRepository::new(pool.clone()));

    let alice_pwd = password::hash_password("password123").map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let alice = users
        .create_user("alice@example.com", &alice_pwd, "Alice")
        .await
        .context("alice user")?;

    let bob_pwd = password::hash_password("password123").map_err(|e| anyhow::anyhow!("{e:?}"))?;
    let bob = users
        .create_user("bob@example.com", &bob_pwd, "Bob")
        .await
        .context("bob user")?;

    let board = boards
        .create_board(alice.id, "Demo board")
        .await
        .context("board")?;

    boards
        .upsert_member(board.id, bob.id, BoardRole::Viewer)
        .await
        .context("bob viewer")?;

    boards
        .insert_element(
            board.id,
            "stroke",
            serde_json::json!({"points":[[0,0],[100,50]],"color":"#000"}),
            0,
        )
        .await
        .context("element")?;

    tracing::info!(?alice.id, ?bob.id, ?board.id, "seed complete");
    Ok(())
}
