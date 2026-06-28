#[cfg(test)]
mod test_support;

pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod ws_protocol;

pub use ws_protocol::parse_client_ws_message;

/// Helpers for fuzz targets — тонкие обёртки над синхронной валидацией,
/// которые не требуют I/O и не могут паниковать.
pub mod fuzz_helpers {
    use crate::infrastructure::password;

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

    /// Десериализует JSON → RegisterInput и проверяет длину пароля.
    pub fn fuzz_register_body(s: &str) {
        if let Ok(body) = serde_json::from_str::<RegisterInput>(s) {
            // та же проверка, что в AuthService::register
            let _ = body.password.len() >= 8;
            // проверяем, что хэширование не паникует на произвольном пароле
            // (используем только быструю проверку длины, не вызываем Argon2 —
            //  он слишком медленный для фаззера)
            let _ = !body.email.is_empty() && !body.display_name.is_empty();
        }
    }

    /// Десериализует JSON → LoginInput.
    pub fn fuzz_login_body(s: &str) {
        if let Ok(body) = serde_json::from_str::<LoginInput>(s) {
            // Проверяем путь verify_password на невалидном хэше (нет I/O)
            let _ = password::verify_password(&body.password, "not-a-valid-phc-string");
        }
    }
}
