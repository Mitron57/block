use std::sync::Arc;

use serde::Serialize;
use uuid::Uuid;

use crate::domain::{DomainError, DomainResult, UserRepository};
use crate::infrastructure::{password, JwtConfig};

pub struct AuthService {
    users: Arc<dyn UserRepository>,
    jwt: JwtConfig,
}

impl AuthService {
    pub fn new(users: Arc<dyn UserRepository>, jwt: JwtConfig) -> Self {
        Self { users, jwt }
    }

    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> DomainResult<(Uuid, String)> {
        if password.len() < 8 {
            return Err(DomainError::InvalidInput(
                "password must be at least 8 characters".into(),
            ));
        }
        let hash = password::hash_password(password).map_err(|_| DomainError::Internal)?;
        let user = self
            .users
            .create_user(email, &hash, display_name)
            .await?;
        let token = self.jwt.sign(user.id)?;
        Ok((user.id, token))
    }

    pub async fn login(&self, email: &str, password: &str) -> DomainResult<(Uuid, String)> {
        let user = self
            .users
            .find_by_email(email)
            .await?
            .ok_or(DomainError::NotFound)?;
        if !password::verify_password(password, &user.password_hash) {
            return Err(DomainError::Forbidden);
        }
        let token = self.jwt.sign(user.id)?;
        Ok((user.id, token))
    }

    pub fn verify_token(&self, token: &str) -> DomainResult<Uuid> {
        self.jwt.verify(token)
    }

    pub async fn me(&self, user_id: Uuid) -> DomainResult<UserDto> {
        let u = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(DomainError::NotFound)?;
        Ok(UserDto {
            id: u.id,
            email: u.email,
            display_name: u.display_name,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::domain::repository::UserRepository;
    use crate::domain::DomainError;
    use crate::infrastructure::{password, JwtConfig};
    use crate::test_support::FakeUserRepo;

    use super::AuthService;

    fn service() -> AuthService {
        AuthService::new(
            Arc::new(FakeUserRepo::new()),
            JwtConfig::new("test-secret-32-characters-minimum!!", 24),
        )
    }

    #[tokio::test]
    async fn register_rejects_short_password() {
        let svc = service();
        let err = svc
            .register("a@b.com", "short", "Alice")
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn register_login_and_verify_token() {
        let svc = service();
        let (id, token) = svc
            .register("alice@example.com", "password123", "Alice")
            .await
            .unwrap();
        assert_eq!(svc.verify_token(&token).unwrap(), id);

        let (id2, token2) = svc
            .login("alice@example.com", "password123")
            .await
            .unwrap();
        assert_eq!(id, id2);
        assert_eq!(svc.verify_token(&token2).unwrap(), id);

        let me = svc.me(id).await.unwrap();
        assert_eq!(me.email, "alice@example.com");
    }

    #[tokio::test]
    async fn login_rejects_wrong_password() {
        let svc = service();
        svc.register("alice@example.com", "password123", "Alice")
            .await
            .unwrap();
        let err = svc
            .login("alice@example.com", "wrong-password")
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Forbidden));
    }

    #[tokio::test]
    async fn login_unknown_email_not_found() {
        let svc = service();
        let err = svc
            .login("nobody@example.com", "password123")
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::NotFound));
    }

    #[tokio::test]
    async fn verify_token_rejects_invalid() {
        let svc = service();
        assert!(matches!(
            svc.verify_token("invalid"),
            Err(DomainError::InvalidInput(_))
        ));
    }

    #[tokio::test]
    async fn password_hash_used_on_register() {
        let users = Arc::new(FakeUserRepo::new());
        let svc = AuthService::new(
            users.clone(),
            JwtConfig::new("test-secret-32-characters-minimum!!", 24),
        );
        svc.register("bob@example.com", "password123", "Bob")
            .await
            .unwrap();
        let stored = users
            .find_by_email("bob@example.com")
            .await
            .unwrap()
            .unwrap();
        assert!(password::verify_password("password123", &stored.password_hash));
        assert!(!password::verify_password("other", &stored.password_hash));
    }
}
