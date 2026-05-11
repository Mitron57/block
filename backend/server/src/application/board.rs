use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{Board, BoardElement, BoardRepository, BoardRole, DomainError, DomainResult, UserRepository};

pub struct BoardService {
    boards: Arc<dyn BoardRepository>,
    users: Arc<dyn UserRepository>,
}

impl BoardService {
    pub fn new(boards: Arc<dyn BoardRepository>, users: Arc<dyn UserRepository>) -> Self {
        Self { boards, users }
    }

    async fn member_role(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<BoardRole> {
        let m = self
            .boards
            .get_member(board_id, user_id)
            .await?
            .ok_or(DomainError::Forbidden)?;
        Ok(m.role)
    }

    pub async fn create_board(&self, owner_id: Uuid, title: &str) -> DomainResult<Board> {
        if title.trim().is_empty() {
            return Err(DomainError::InvalidInput("title required".into()));
        }
        self.boards.create_board(owner_id, title.trim()).await
    }

    pub async fn list_boards(&self, user_id: Uuid) -> DomainResult<Vec<Board>> {
        self.boards.list_boards_for_user(user_id).await
    }

    pub async fn get_board(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<Board> {
        self.member_role(board_id, user_id).await?;
        self.boards
            .find_board(board_id)
            .await?
            .ok_or(DomainError::NotFound)
    }

    pub async fn update_board(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        title: &str,
    ) -> DomainResult<()> {
        let role = self.member_role(board_id, user_id).await?;
        if !role.can_edit_board() {
            return Err(DomainError::Forbidden);
        }
        if title.trim().is_empty() {
            return Err(DomainError::InvalidInput("title required".into()));
        }
        self.boards.update_board_title(board_id, title.trim()).await
    }

    pub async fn delete_board(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<()> {
        let role = self.member_role(board_id, user_id).await?;
        if !role.can_manage_members() {
            return Err(DomainError::Forbidden);
        }
        self.boards.delete_board(board_id).await
    }

    pub async fn list_members(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> DomainResult<Vec<MemberDto>> {
        self.member_role(board_id, user_id).await?;
        let members = self.boards.list_members(board_id).await?;
        let mut out = Vec::new();
        for m in members {
            let u = self
                .users
                .find_by_id(m.user_id)
                .await?
                .ok_or(DomainError::Internal)?;
            out.push(MemberDto {
                user_id: m.user_id,
                email: u.email,
                display_name: u.display_name,
                role: m.role,
            });
        }
        Ok(out)
    }

    pub async fn add_member_by_email(
        &self,
        board_id: Uuid,
        actor_id: Uuid,
        email: &str,
        role: BoardRole,
    ) -> DomainResult<()> {
        let actor = self.member_role(board_id, actor_id).await?;
        if !actor.can_manage_members() {
            return Err(DomainError::Forbidden);
        }
        if matches!(role, BoardRole::Owner) {
            return Err(DomainError::InvalidInput(
                "cannot assign owner role".into(),
            ));
        }
        let target = self
            .users
            .find_by_email(email)
            .await?
            .ok_or(DomainError::NotFound)?;
        self.boards
            .upsert_member(board_id, target.id, role)
            .await
    }

    pub async fn set_member_role(
        &self,
        board_id: Uuid,
        actor_id: Uuid,
        target_user_id: Uuid,
        role: BoardRole,
    ) -> DomainResult<()> {
        let actor = self.member_role(board_id, actor_id).await?;
        if !actor.can_manage_members() {
            return Err(DomainError::Forbidden);
        }
        if matches!(role, BoardRole::Owner) {
            return Err(DomainError::InvalidInput(
                "cannot assign owner role".into(),
            ));
        }
        let existing = self
            .boards
            .get_member(board_id, target_user_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        if matches!(existing.role, BoardRole::Owner) {
            return Err(DomainError::Forbidden);
        }
        self.boards
            .upsert_member(board_id, target_user_id, role)
            .await
    }

    pub async fn remove_member(
        &self,
        board_id: Uuid,
        actor_id: Uuid,
        target_user_id: Uuid,
    ) -> DomainResult<()> {
        let actor = self.member_role(board_id, actor_id).await?;
        if !actor.can_manage_members() {
            return Err(DomainError::Forbidden);
        }
        let existing = self
            .boards
            .get_member(board_id, target_user_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        if matches!(existing.role, BoardRole::Owner) {
            return Err(DomainError::Forbidden);
        }
        self.boards.remove_member(board_id, target_user_id).await
    }

    pub async fn list_elements(
        &self,
        board_id: Uuid,
        user_id: Uuid,
    ) -> DomainResult<Vec<BoardElement>> {
        self.member_role(board_id, user_id).await?;
        self.boards.list_elements(board_id).await
    }

    pub async fn add_element(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        element_type: &str,
        payload: serde_json::Value,
        z_index: i32,
    ) -> DomainResult<BoardElement> {
        let role = self.member_role(board_id, user_id).await?;
        if !role.can_edit_board() {
            return Err(DomainError::Forbidden);
        }
        self.boards
            .insert_element(board_id, element_type, payload, z_index)
            .await
    }

    pub async fn remove_element(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        element_id: Uuid,
    ) -> DomainResult<()> {
        let role = self.member_role(board_id, user_id).await?;
        if !role.can_edit_board() {
            return Err(DomainError::Forbidden);
        }
        let ok = self.boards.delete_element(board_id, element_id).await?;
        if !ok {
            return Err(DomainError::NotFound);
        }
        Ok(())
    }

    pub async fn clear_elements(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<()> {
        let role = self.member_role(board_id, user_id).await?;
        if !role.can_edit_board() {
            return Err(DomainError::Forbidden);
        }
        self.boards.clear_elements(board_id).await
    }
}

#[derive(Debug, Serialize)]
pub struct MemberDto {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: BoardRole,
}

#[derive(Debug, Deserialize)]
pub struct CreateElementBody {
    pub element_type: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub z_index: i32,
}
