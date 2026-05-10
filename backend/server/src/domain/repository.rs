use async_trait::async_trait;
use uuid::Uuid;

use super::models::{Board, BoardElement, BoardMember, BoardRole, User};
use super::error::DomainResult;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> DomainResult<User>;
    async fn find_by_email(&self, email: &str) -> DomainResult<Option<User>>;
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<User>>;
}

#[async_trait]
pub trait BoardRepository: Send + Sync {
    async fn create_board(&self, owner_id: Uuid, title: &str) -> DomainResult<Board>;
    async fn find_board(&self, id: Uuid) -> DomainResult<Option<Board>>;
    async fn list_boards_for_user(&self, user_id: Uuid) -> DomainResult<Vec<Board>>;
    async fn update_board_title(&self, board_id: Uuid, title: &str) -> DomainResult<()>;
    async fn delete_board(&self, board_id: Uuid) -> DomainResult<()>;

    async fn get_member(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<Option<BoardMember>>;
    async fn upsert_member(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        role: BoardRole,
    ) -> DomainResult<()>;
    async fn remove_member(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<()>;
    async fn list_members(&self, board_id: Uuid) -> DomainResult<Vec<BoardMember>>;

    async fn list_elements(&self, board_id: Uuid) -> DomainResult<Vec<BoardElement>>;
    async fn insert_element(
        &self,
        board_id: Uuid,
        element_type: &str,
        payload: serde_json::Value,
        z_index: i32,
    ) -> DomainResult<BoardElement>;
    async fn delete_element(&self, board_id: Uuid, element_id: Uuid) -> DomainResult<bool>;
    async fn clear_elements(&self, board_id: Uuid) -> DomainResult<()>;
}
