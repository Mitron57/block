use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    Board, BoardElement, BoardMember, BoardRepository, BoardRole, DomainError, DomainResult, User,
    UserRepository,
};

fn map_sqlx(e: sqlx::Error) -> DomainError {
    if let Some(db) = e.as_database_error() {
        if db.code().as_deref() == Some("23505") {
            return DomainError::Conflict("unique violation".into());
        }
    }
    tracing::error!(?e, "sqlx error");
    DomainError::Internal
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    password_hash: String,
    display_name: String,
    created_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(r: UserRow) -> Self {
        User {
            id: r.id,
            email: r.email,
            password_hash: r.password_hash,
            display_name: r.display_name,
            created_at: r.created_at,
        }
    }
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> DomainResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"INSERT INTO users (email, password_hash, display_name)
               VALUES ($1, $2, $3)
               RETURNING id, email, password_hash, display_name, created_at"#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(row.into())
    }

    async fn find_by_email(&self, email: &str) -> DomainResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, display_name, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(row.map(Into::into))
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash, display_name, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(row.map(Into::into))
    }
}

#[derive(sqlx::FromRow)]
struct BoardRow {
    id: Uuid,
    owner_id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct BoardMemberRow {
    board_id: Uuid,
    user_id: Uuid,
    role: BoardRole,
}

#[derive(sqlx::FromRow)]
struct BoardElementRow {
    id: Uuid,
    board_id: Uuid,
    element_type: String,
    payload: serde_json::Value,
    z_index: i32,
    created_at: DateTime<Utc>,
}

pub struct PgBoardRepository {
    pool: PgPool,
}

impl PgBoardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BoardRepository for PgBoardRepository {
    async fn create_board(&self, owner_id: Uuid, title: &str) -> DomainResult<Board> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx)?;
        let row = sqlx::query_as::<_, BoardRow>(
            r#"INSERT INTO boards (owner_id, title) VALUES ($1, $2)
               RETURNING id, owner_id, title, created_at"#,
        )
        .bind(owner_id)
        .bind(title)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_sqlx)?;
        sqlx::query(
            r#"INSERT INTO board_members (board_id, user_id, role) VALUES ($1, $2, 'owner')"#,
        )
        .bind(row.id)
        .bind(owner_id)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx)?;
        tx.commit().await.map_err(map_sqlx)?;
        Ok(Board {
            id: row.id,
            owner_id: row.owner_id,
            title: row.title,
            created_at: row.created_at,
        })
    }

    async fn find_board(&self, id: Uuid) -> DomainResult<Option<Board>> {
        let row = sqlx::query_as::<_, BoardRow>(
            "SELECT id, owner_id, title, created_at FROM boards WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(row.map(|r| Board {
            id: r.id,
            owner_id: r.owner_id,
            title: r.title,
            created_at: r.created_at,
        }))
    }

    async fn list_boards_for_user(&self, user_id: Uuid) -> DomainResult<Vec<Board>> {
        let rows = sqlx::query_as::<_, BoardRow>(
            r#"SELECT b.id, b.owner_id, b.title, b.created_at
               FROM boards b
               INNER JOIN board_members m ON m.board_id = b.id
               WHERE m.user_id = $1
               ORDER BY b.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(rows
            .into_iter()
            .map(|r| Board {
                id: r.id,
                owner_id: r.owner_id,
                title: r.title,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn update_board_title(&self, board_id: Uuid, title: &str) -> DomainResult<()> {
        let res = sqlx::query("UPDATE boards SET title = $1 WHERE id = $2")
            .bind(title)
            .bind(board_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }

    async fn delete_board(&self, board_id: Uuid) -> DomainResult<()> {
        let res = sqlx::query("DELETE FROM boards WHERE id = $1")
            .bind(board_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }

    async fn get_member(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<Option<BoardMember>> {
        let row = sqlx::query_as::<_, BoardMemberRow>(
            "SELECT board_id, user_id, role FROM board_members WHERE board_id = $1 AND user_id = $2",
        )
        .bind(board_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(row.map(|r| BoardMember {
            board_id: r.board_id,
            user_id: r.user_id,
            role: r.role,
        }))
    }

    async fn upsert_member(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        role: BoardRole,
    ) -> DomainResult<()> {
        sqlx::query(
            r#"INSERT INTO board_members (board_id, user_id, role) VALUES ($1, $2, $3)
               ON CONFLICT (board_id, user_id) DO UPDATE SET role = EXCLUDED.role"#,
        )
        .bind(board_id)
        .bind(user_id)
        .bind(role)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(())
    }

    async fn remove_member(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<()> {
        let res = sqlx::query("DELETE FROM board_members WHERE board_id = $1 AND user_id = $2")
            .bind(board_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        if res.rows_affected() == 0 {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }

    async fn list_members(&self, board_id: Uuid) -> DomainResult<Vec<BoardMember>> {
        let rows = sqlx::query_as::<_, BoardMemberRow>(
            "SELECT board_id, user_id, role FROM board_members WHERE board_id = $1",
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(rows
            .into_iter()
            .map(|r| BoardMember {
                board_id: r.board_id,
                user_id: r.user_id,
                role: r.role,
            })
            .collect())
    }

    async fn list_elements(&self, board_id: Uuid) -> DomainResult<Vec<BoardElement>> {
        let rows = sqlx::query_as::<_, BoardElementRow>(
            r#"SELECT id, board_id, element_type, payload, z_index, created_at
               FROM board_elements WHERE board_id = $1 ORDER BY z_index, created_at"#,
        )
        .bind(board_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(rows
            .into_iter()
            .map(|r| BoardElement {
                id: r.id,
                board_id: r.board_id,
                element_type: r.element_type,
                payload: r.payload,
                z_index: r.z_index,
                created_at: r.created_at,
            })
            .collect())
    }

    async fn insert_element(
        &self,
        board_id: Uuid,
        element_type: &str,
        payload: serde_json::Value,
        z_index: i32,
    ) -> DomainResult<BoardElement> {
        let row = sqlx::query_as::<_, BoardElementRow>(
            r#"INSERT INTO board_elements (board_id, element_type, payload, z_index)
               VALUES ($1, $2, $3, $4)
               RETURNING id, board_id, element_type, payload, z_index, created_at"#,
        )
        .bind(board_id)
        .bind(element_type)
        .bind(payload)
        .bind(z_index)
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx)?;
        Ok(BoardElement {
            id: row.id,
            board_id: row.board_id,
            element_type: row.element_type,
            payload: row.payload,
            z_index: row.z_index,
            created_at: row.created_at,
        })
    }

    async fn delete_element(&self, board_id: Uuid, element_id: Uuid) -> DomainResult<bool> {
        let res = sqlx::query("DELETE FROM board_elements WHERE board_id = $1 AND id = $2")
            .bind(board_id)
            .bind(element_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(res.rows_affected() > 0)
    }

    async fn clear_elements(&self, board_id: Uuid) -> DomainResult<()> {
        sqlx::query("DELETE FROM board_elements WHERE board_id = $1")
            .bind(board_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx)?;
        Ok(())
    }
}
