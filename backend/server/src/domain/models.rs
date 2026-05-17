use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "board_role", rename_all = "lowercase")]
pub enum BoardRole {
    Owner,
    Editor,
    Viewer,
}

impl BoardRole {
    pub fn can_edit_board(self) -> bool {
        matches!(self, BoardRole::Owner | BoardRole::Editor)
    }

    pub fn can_manage_members(self) -> bool {
        matches!(self, BoardRole::Owner)
    }
}

#[cfg(test)]
mod tests {
    use super::BoardRole;

    #[test]
    fn viewer_cannot_edit_board() {
        assert!(!BoardRole::Viewer.can_edit_board());
        assert!(BoardRole::Editor.can_edit_board());
        assert!(BoardRole::Owner.can_edit_board());
    }

    #[test]
    fn only_owner_manages_members() {
        assert!(BoardRole::Owner.can_manage_members());
        assert!(!BoardRole::Editor.can_manage_members());
        assert!(!BoardRole::Viewer.can_manage_members());
    }

    #[test]
    fn board_role_deserializes_lowercase() {
        let editor: BoardRole = serde_json::from_str("\"editor\"").unwrap();
        let viewer: BoardRole = serde_json::from_str("\"viewer\"").unwrap();
        let owner: BoardRole = serde_json::from_str("\"owner\"").unwrap();
        assert_eq!(editor, BoardRole::Editor);
        assert_eq!(viewer, BoardRole::Viewer);
        assert_eq!(owner, BoardRole::Owner);
    }

    #[test]
    fn board_role_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&BoardRole::Editor).unwrap(),
            "\"editor\""
        );
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Board {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BoardMember {
    pub board_id: Uuid,
    pub user_id: Uuid,
    pub role: BoardRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardElement {
    pub id: Uuid,
    pub board_id: Uuid,
    pub element_type: String,
    pub payload: serde_json::Value,
    pub z_index: i32,
    pub created_at: DateTime<Utc>,
}
