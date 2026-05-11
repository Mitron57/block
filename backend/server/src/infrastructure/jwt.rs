use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::DomainError;

#[derive(Clone)]
pub struct JwtConfig {
    secret: Vec<u8>,
    ttl_hours: i64,
}

impl JwtConfig {
    pub fn new(secret: impl Into<String>, ttl_hours: i64) -> Self {
        Self {
            secret: secret.into().into_bytes(),
            ttl_hours,
        }
    }

    pub fn sign(&self, user_id: Uuid) -> Result<String, DomainError> {
        let exp = (Utc::now() + Duration::hours(self.ttl_hours)).timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )
        .map_err(|_| DomainError::Internal)
    }

    pub fn verify(&self, token: &str) -> Result<Uuid, DomainError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.secret),
            &Validation::default(),
        )
        .map_err(|_| DomainError::InvalidInput("invalid token".into()))?;
        Uuid::parse_str(&data.claims.sub).map_err(|_| DomainError::InvalidInput("invalid sub".into()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}
