use crate::{ApiError, ApiResult};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub display_name: String,
    pub role: String,
}

impl User {
    pub fn require(&self, role: &str) -> ApiResult<()> {
        if self.role == role {
            return Ok(());
        }
        Err(ApiError(
            StatusCode::FORBIDDEN,
            "Action interdite pour ce rôle.",
        ))
    }
}

fn unauthorized() -> ApiError {
    ApiError(
        StatusCode::UNAUTHORIZED,
        "Identifiants ou session invalides.",
    )
}

fn token_hash(value: &str) -> Vec<u8> {
    Sha256::digest(value.as_bytes()).to_vec()
}

fn bearer(headers: &HeaderMap) -> ApiResult<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(unauthorized)
}

pub async fn user(pool: &PgPool, headers: &HeaderMap) -> ApiResult<User> {
    sqlx::query_as(
        "SELECT u.id, u.display_name, u.role FROM core.users u
        JOIN core.login_sessions s ON s.user_id = u.id
        WHERE s.token_hash = $1 AND u.active AND s.expires_at > now()",
    )
    .bind(token_hash(bearer(headers)?))
    .fetch_optional(pool)
    .await?
    .ok_or_else(unauthorized)
}

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

async fn login(
    State(pool): State<PgPool>,
    Json(input): Json<Credentials>,
) -> ApiResult<Json<Value>> {
    if input.username.len() > 80 || input.password.len() > 256 {
        return Err(unauthorized());
    }
    let (id, hash): (i64, String) =
        sqlx::query_as("SELECT id, password_hash FROM core.users WHERE username = $1 AND active")
            .bind(input.username)
            .fetch_optional(&pool)
            .await?
            .ok_or_else(unauthorized)?;
    let verified = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).is_ok_and(|hash| {
            Argon2::default()
                .verify_password(input.password.as_bytes(), &hash)
                .is_ok()
        })
    })
    .await
    .map_err(|_| ApiError(StatusCode::INTERNAL_SERVER_ERROR, "Erreur interne."))?;
    if !verified {
        return Err(unauthorized());
    }
    let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM core.login_sessions WHERE user_id = $1 AND expires_at <= now()")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO core.login_sessions VALUES ($1, $2, now() + interval '8 hours')")
        .bind(token_hash(&token))
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({"token": token})))
}

async fn logout(State(pool): State<PgPool>, headers: HeaderMap) -> ApiResult<StatusCode> {
    sqlx::query("DELETE FROM core.login_sessions WHERE token_hash = $1")
        .bind(token_hash(bearer(&headers)?))
        .execute(&pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(State(pool): State<PgPool>, headers: HeaderMap) -> ApiResult<Json<User>> {
    Ok(Json(user(&pool, &headers).await?))
}

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

pub async fn seed(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    for (username, name, role, env) in [
        (
            "superviseur",
            "Superviseur",
            "supervisor",
            "SUPERVISOR_PASSWORD",
        ),
        (
            "operateur-a",
            "Opérateur A",
            "operator",
            "OPERATOR_A_PASSWORD",
        ),
        (
            "operateur-b",
            "Opérateur B",
            "operator",
            "OPERATOR_B_PASSWORD",
        ),
    ] {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM core.users WHERE username = $1)")
                .bind(username)
                .fetch_one(pool)
                .await?;
        if exists {
            continue;
        }
        let password = std::env::var(env)?;
        if !(16..=256).contains(&password.len()) {
            return Err(format!("{env} doit contenir entre 16 et 256 octets").into());
        }
        let salt = SaltString::encode_b64(Uuid::new_v4().as_bytes())?;
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        sqlx::query(
            "INSERT INTO core.users (username, display_name, role, password_hash)
            VALUES ($1, $2, $3, $4) ON CONFLICT (username) DO NOTHING",
        )
        .bind(username)
        .bind(name)
        .bind(role)
        .bind(hash)
        .execute(pool)
        .await?;
    }
    Ok(())
}
