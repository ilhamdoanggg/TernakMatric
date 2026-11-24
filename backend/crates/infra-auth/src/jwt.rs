use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use shared_kernel::{AppError, UserId};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub ttl_minutes: i64,
}

impl JwtConfig {
    pub fn from_env() -> Self {
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "ternakmatric-dev-secret-change-in-prod".to_string());
        let ttl_minutes = std::env::var("JWT_TTL_MINUTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60 * 24);
        Self {
            secret,
            ttl_minutes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,
    pub email: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Clone)]
pub struct JwtService {
    cfg: JwtConfig,
}

impl JwtService {
    pub fn new(cfg: JwtConfig) -> Self {
        Self { cfg }
    }

    pub fn issue_token(&self, user_id: UserId, email: &str) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.cfg.ttl_minutes);
        let claims = JwtClaims {
            sub: user_id.0,
            email: email.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.cfg.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("jwt encode failed: {e}")))
    }

    pub fn verify_token(&self, token: &str) -> Result<JwtClaims, AppError> {
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.cfg.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| AppError::Unauthorized(format!("invalid token: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_issue_and_verify_roundtrip() {
        let svc = JwtService::new(JwtConfig {
            secret: "test-secret".into(),
            ttl_minutes: 60,
        });
        let uid = UserId::new();
        let token = svc.issue_token(uid, "peternak@ternakmatric.id").unwrap();
        let claims = svc.verify_token(&token).unwrap();
        assert_eq!(claims.sub, uid.0);
        assert_eq!(claims.email, "peternak@ternakmatric.id");
    }
}
