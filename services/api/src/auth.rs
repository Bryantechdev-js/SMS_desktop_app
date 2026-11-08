use crate::AppState;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, String);
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let token = value
            .strip_prefix("Bearer ")
            .filter(|v| !v.is_empty())
            .ok_or((StatusCode::UNAUTHORIZED, "Authentication required".into()))?;
        let token_hash = hash_token(token);
        let row = sqlx::query("SELECT u.id,u.username,u.display_name FROM auth_sessions s JOIN app_users u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.revoked_at IS NULL AND s.expires_at > now() AND u.is_active=true")
            .bind(token_hash).fetch_optional(&state.db).await.map_err(|e| { tracing::error!(error=%e,"session lookup failed"); (StatusCode::INTERNAL_SERVER_ERROR,"Authentication service unavailable".into()) })?
            .ok_or((StatusCode::UNAUTHORIZED,"Session is invalid or expired".into()))?;
        let user = Self {
            id: row.get("id"),
            username: row.get("username"),
            display_name: row.get("display_name"),
        };
        // Update activity without extending the absolute expiry.
        let _ = sqlx::query("UPDATE auth_sessions SET last_seen_at=now() WHERE token_hash=$1 AND last_seen_at < now()-interval '5 minutes'").bind(hash_token(token)).execute(&state.db).await;
        Ok(user)
    }
}

pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
pub fn new_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}
pub fn hash_password(password: &str) -> Result<String, String> {
    if password.chars().count() < 12 || password.len() > 256 {
        return Err(
            "Password must contain at least 12 characters and no more than 256 bytes".into(),
        );
    }
    let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())
        .map_err(|_| "Could not create password salt".to_string())?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| "Could not hash password".into())
}
pub fn verify_password(password: &str, encoded: &str) -> bool {
    PasswordHash::new(encoded)
        .map(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

pub async fn require_permission(
    state: &AppState,
    user_id: Uuid,
    permission: &str,
) -> Result<(), (StatusCode, String)> {
    let allowed: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_roles ur JOIN role_permissions rp ON rp.role_id=ur.role_id JOIN permissions p ON p.id=rp.permission_id WHERE ur.user_id=$1 AND p.code=$2)")
        .bind(user_id).bind(permission).fetch_one(&state.db).await.map_err(|e| { tracing::error!(error=%e,"authorization lookup failed"); (StatusCode::INTERNAL_SERVER_ERROR,"Authorization service unavailable".into()) })?;
    if allowed {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            "You do not have permission to perform this action".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_verifies_without_storing_plaintext() {
        let password = "ComplexeSMS!Secure2026";
        let encoded = hash_password(password).expect("valid password should hash");
        assert_ne!(encoded, password);
        assert!(verify_password(password, &encoded));
        assert!(!verify_password("incorrect-password", &encoded));
    }

    #[test]
    fn password_policy_rejects_short_passwords() {
        assert!(hash_password("short").is_err());
    }

    #[test]
    fn session_tokens_are_random_and_only_digests_are_persisted() {
        let first = new_token();
        let second = new_token();
        assert_ne!(first, second);
        assert_eq!(hash_token(&first).len(), 64);
        assert_ne!(hash_token(&first), first);
    }
}
