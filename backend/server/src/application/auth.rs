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
