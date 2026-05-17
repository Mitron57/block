//! In-memory fakes for application-layer unit tests.
use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    Board, BoardElement, BoardMember, BoardRepository, BoardRole, DomainError, DomainResult, User,
    UserRepository,
};

pub struct FakeUserRepo {
    inner: Mutex<HashMap<String, User>>,
}

impl FakeUserRepo {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, email: &str, display_name: &str) -> User {
        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            password_hash: "hash".into(),
            display_name: display_name.to_string(),
            created_at: Utc::now(),
        };
        self.inner.lock().unwrap().insert(email.to_string(), user.clone());
        user
    }
}

#[async_trait]
impl UserRepository for FakeUserRepo {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> DomainResult<User> {
        let mut g = self.inner.lock().unwrap();
        if g.contains_key(email) {
            return Err(DomainError::Conflict("email taken".into()));
        }
        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            display_name: display_name.to_string(),
            created_at: Utc::now(),
        };
        g.insert(email.to_string(), user.clone());
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> DomainResult<Option<User>> {
        Ok(self.inner.lock().unwrap().get(email).cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<User>> {
        Ok(self
            .inner
            .lock()
            .unwrap()
            .values()
            .find(|u| u.id == id)
            .cloned())
    }
}

pub struct FakeBoardRepo {
    boards: Mutex<HashMap<Uuid, Board>>,
    members: Mutex<HashMap<(Uuid, Uuid), BoardMember>>,
    elements: Mutex<HashMap<Uuid, Vec<BoardElement>>>,
}

impl FakeBoardRepo {
    pub fn new() -> Self {
        Self {
            boards: Mutex::new(HashMap::new()),
            members: Mutex::new(HashMap::new()),
            elements: Mutex::new(HashMap::new()),
        }
    }

    fn member_key(board_id: Uuid, user_id: Uuid) -> (Uuid, Uuid) {
        (board_id, user_id)
    }
}

#[async_trait]
impl BoardRepository for FakeBoardRepo {
    async fn create_board(&self, owner_id: Uuid, title: &str) -> DomainResult<Board> {
        let board = Board {
            id: Uuid::new_v4(),
            owner_id,
            title: title.to_string(),
            created_at: Utc::now(),
        };
        self.boards.lock().unwrap().insert(board.id, board.clone());
        self.members.lock().unwrap().insert(
            Self::member_key(board.id, owner_id),
            BoardMember {
                board_id: board.id,
                user_id: owner_id,
                role: BoardRole::Owner,
            },
        );
        Ok(board)
    }

    async fn find_board(&self, id: Uuid) -> DomainResult<Option<Board>> {
        Ok(self.boards.lock().unwrap().get(&id).cloned())
    }

    async fn list_boards_for_user(&self, user_id: Uuid) -> DomainResult<Vec<Board>> {
        let members = self.members.lock().unwrap();
        let boards = self.boards.lock().unwrap();
        Ok(members
            .values()
            .filter(|m| m.user_id == user_id)
            .filter_map(|m| boards.get(&m.board_id).cloned())
            .collect())
    }

    async fn update_board_title(&self, board_id: Uuid, title: &str) -> DomainResult<()> {
        let mut boards = self.boards.lock().unwrap();
        let b = boards.get_mut(&board_id).ok_or(DomainError::NotFound)?;
        b.title = title.to_string();
        Ok(())
    }

    async fn delete_board(&self, board_id: Uuid) -> DomainResult<()> {
        self.boards.lock().unwrap().remove(&board_id);
        self.members
            .lock()
            .unwrap()
            .retain(|(bid, _), _| *bid != board_id);
        self.elements.lock().unwrap().remove(&board_id);
        Ok(())
    }

    async fn get_member(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<Option<BoardMember>> {
        Ok(self
            .members
            .lock()
            .unwrap()
            .get(&Self::member_key(board_id, user_id))
            .cloned())
    }

    async fn upsert_member(
        &self,
        board_id: Uuid,
        user_id: Uuid,
        role: BoardRole,
    ) -> DomainResult<()> {
        self.members.lock().unwrap().insert(
            Self::member_key(board_id, user_id),
            BoardMember {
                board_id,
                user_id,
                role,
            },
        );
        Ok(())
    }

    async fn remove_member(&self, board_id: Uuid, user_id: Uuid) -> DomainResult<()> {
        self.members
            .lock()
            .unwrap()
            .remove(&Self::member_key(board_id, user_id));
        Ok(())
    }

    async fn list_members(&self, board_id: Uuid) -> DomainResult<Vec<BoardMember>> {
        Ok(self
            .members
            .lock()
            .unwrap()
            .values()
            .filter(|m| m.board_id == board_id)
            .cloned()
            .collect())
    }

    async fn list_elements(&self, board_id: Uuid) -> DomainResult<Vec<BoardElement>> {
        Ok(self
            .elements
            .lock()
            .unwrap()
            .get(&board_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn insert_element(
        &self,
        board_id: Uuid,
        element_type: &str,
        payload: serde_json::Value,
        z_index: i32,
    ) -> DomainResult<BoardElement> {
        let el = BoardElement {
            id: Uuid::new_v4(),
            board_id,
            element_type: element_type.to_string(),
            payload,
            z_index,
            created_at: Utc::now(),
        };
        self.elements
            .lock()
            .unwrap()
            .entry(board_id)
            .or_default()
            .push(el.clone());
        Ok(el)
    }

    async fn delete_element(&self, board_id: Uuid, element_id: Uuid) -> DomainResult<bool> {
        let mut map = self.elements.lock().unwrap();
        let Some(list) = map.get_mut(&board_id) else {
            return Ok(false);
        };
        let before = list.len();
        list.retain(|e| e.id != element_id);
        Ok(list.len() < before)
    }

    async fn clear_elements(&self, board_id: Uuid) -> DomainResult<()> {
        self.elements.lock().unwrap().insert(board_id, vec![]);
        Ok(())
    }
}
