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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::domain::repository::BoardRepository;
    use crate::domain::{BoardRole, DomainError};
    use crate::test_support::{FakeBoardRepo, FakeUserRepo};

    use super::BoardService;

    struct Fixture {
        boards: Arc<FakeBoardRepo>,
        users: Arc<FakeUserRepo>,
        svc: BoardService,
        owner_id: uuid::Uuid,
        viewer_id: uuid::Uuid,
        board_id: uuid::Uuid,
    }

    async fn fixture() -> Fixture {
        let boards = Arc::new(FakeBoardRepo::new());
        let users = Arc::new(FakeUserRepo::new());
        let owner = users.insert("owner@example.com", "Owner");
        let viewer = users.insert("viewer@example.com", "Viewer");
        let svc = BoardService::new(boards.clone(), users.clone());
        let board = svc.create_board(owner.id, "Test board").await.unwrap();
        boards
            .upsert_member(board.id, viewer.id, BoardRole::Viewer)
            .await
            .unwrap();
        Fixture {
            boards,
            users,
            svc,
            owner_id: owner.id,
            viewer_id: viewer.id,
            board_id: board.id,
        }
    }

    #[tokio::test]
    async fn create_board_rejects_empty_title() {
        let boards = Arc::new(FakeBoardRepo::new());
        let users = Arc::new(FakeUserRepo::new());
        let owner = users.insert("o@e.com", "O");
        let svc = BoardService::new(boards, users);
        let err = svc.create_board(owner.id, "   ").await.unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn viewer_cannot_rename_board() {
        let f = fixture().await;
        let err = f
            .svc
            .update_board(f.board_id, f.viewer_id, "Hacked")
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn viewer_cannot_add_element() {
        let f = fixture().await;
        let err = f
            .svc
            .add_element(
                f.board_id,
                f.viewer_id,
                "stroke",
                serde_json::json!({}),
                0,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn editor_cannot_delete_board() {
        let f = fixture().await;
        f.boards
            .upsert_member(f.board_id, f.viewer_id, BoardRole::Editor)
            .await
            .unwrap();
        let err = f
            .svc
            .delete_board(f.board_id, f.viewer_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn owner_can_add_member_by_email() {
        let f = fixture().await;
        let bob = f.users.insert("bob@example.com", "Bob");
        f.svc
            .add_member_by_email(f.board_id, f.owner_id, "bob@example.com", BoardRole::Editor)
            .await
            .unwrap();
        let role = f
            .boards
            .get_member(f.board_id, bob.id)
            .await
            .unwrap()
            .unwrap()
            .role;
        assert_eq!(role, BoardRole::Editor);
    }

    #[tokio::test]
    async fn cannot_assign_owner_role_via_api() {
        let f = fixture().await;
        f.users.insert("bob@example.com", "Bob");
        let err = f
            .svc
            .add_member_by_email(f.board_id, f.owner_id, "bob@example.com", BoardRole::Owner)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn viewer_cannot_add_members() {
        let f = fixture().await;
        f.users.insert("bob@example.com", "Bob");
        let err = f
            .svc
            .add_member_by_email(f.board_id, f.viewer_id, "bob@example.com", BoardRole::Editor)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn cannot_change_owner_member_role() {
        let f = fixture().await;
        let err = f
            .svc
            .set_member_role(f.board_id, f.owner_id, f.owner_id, BoardRole::Editor)
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn owner_can_remove_editor() {
        let f = fixture().await;
        f.boards
            .upsert_member(f.board_id, f.viewer_id, BoardRole::Editor)
            .await
            .unwrap();
        f.svc
            .remove_member(f.board_id, f.owner_id, f.viewer_id)
            .await
            .unwrap();
        assert!(
            f.boards
                .get_member(f.board_id, f.viewer_id)
                .await
                .unwrap()
                .is_none()
        );
    }
}
