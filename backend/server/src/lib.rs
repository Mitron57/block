#[cfg(test)]
mod test_support;

pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod ws_protocol;

pub use ws_protocol::parse_client_ws_message;

/// Helpers for fuzz targets — тонкие обёртки над синхронной десериализацией,
/// которые не требуют I/O и не могут паниковать.
pub mod fuzz_helpers {
    use crate::domain::BoardRole;

    // ── Auth bodies ──────────────────────────────────────────────────────────

    #[derive(serde::Deserialize)]
    pub struct RegisterInput {
        pub email: String,
        pub password: String,
        pub display_name: String,
    }

    #[derive(serde::Deserialize)]
    pub struct LoginInput {
        pub email: String,
        pub password: String,
    }

    pub fn fuzz_register_body(s: &str) {
        if let Ok(body) = serde_json::from_str::<RegisterInput>(s) {
            let _ = body.password.len() >= 8;
            let _ = !body.email.is_empty() && !body.display_name.is_empty();
        }
    }

    pub fn fuzz_login_body(s: &str) {
        if let Ok(body) = serde_json::from_str::<LoginInput>(s) {
            let _ = !body.email.is_empty();
            let _ = body.password.len();
        }
    }

    // ── Board / member bodies ────────────────────────────────────────────────

    #[derive(serde::Deserialize)]
    pub struct BoardCreateInput {
        pub title: String,
    }

    #[derive(serde::Deserialize)]
    pub struct BoardUpdateInput {
        pub title: String,
    }

    #[derive(serde::Deserialize)]
    pub struct AddMemberInput {
        pub email: String,
        pub role: BoardRole,
    }

    #[derive(serde::Deserialize)]
    pub struct SetRoleInput {
        pub role: BoardRole,
    }

    /// Фаззит все четыре тела запросов board/member-эндпоинтов.
    pub fn fuzz_board_bodies(s: &str) {
        if let Ok(b) = serde_json::from_str::<BoardCreateInput>(s) {
            let _ = b.title.trim().is_empty();
        }
        if let Ok(b) = serde_json::from_str::<BoardUpdateInput>(s) {
            let _ = b.title.len();
        }
        if let Ok(b) = serde_json::from_str::<AddMemberInput>(s) {
            let _ = b.email.contains('@');
            let _ = b.role.can_edit_board();
        }
        if let Ok(b) = serde_json::from_str::<SetRoleInput>(s) {
            let _ = b.role.can_manage_members();
        }
    }

    // ── Element body ─────────────────────────────────────────────────────────

    #[derive(serde::Deserialize)]
    pub struct CreateElementInput {
        pub element_type: String,
        pub payload: serde_json::Value,
        #[serde(default)]
        pub z_index: i32,
    }

    /// Фаззит тело POST /api/boards/{id}/elements.
    /// Особый интерес — поле payload: serde_json::Value,
    /// которое принимает произвольный JSON без фиксированной схемы.
    pub fn fuzz_element_body(s: &str) {
        if let Ok(b) = serde_json::from_str::<CreateElementInput>(s) {
            let _ = b.element_type.is_empty();
            let _ = b.z_index;
            let _ = serde_json::to_string(&b.payload);
        }
    }
}
