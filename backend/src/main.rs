mod auth;
mod engineering;
mod manufacturing;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::{Value, json};
use sqlx::{
    PgPool, Postgres, Transaction,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::error::Error;

struct ApiError(StatusCode, &'static str);
type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"message": self.1}))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        let constraint = error
            .as_database_error()
            .and_then(|error| error.constraint());
        match constraint {
            Some("one_active_session_per_operator") => {
                Self(StatusCode::CONFLICT, "Vous avez déjà un pointage actif.")
            }
            Some("work_orders_number_key") => {
                Self(StatusCode::CONFLICT, "Ce numéro d’OF existe déjà.")
            }
            Some("item_revisions_item_revision_key") => Self(
                StatusCode::CONFLICT,
                "Cette révision existe déjà pour cet article.",
            ),
            Some("work_orders_item_revision_id_fkey") => {
                Self(StatusCode::BAD_REQUEST, "Révision d’article introuvable.")
            }
            _ => {
                tracing::error!(%error, "Erreur PostgreSQL");
                Self(StatusCode::INTERNAL_SERVER_ERROR, "Erreur interne.")
            }
        }
    }
}

async fn audit(
    tx: &mut Transaction<'_, Postgres>,
    actor: i64,
    action: &str,
    entity: &str,
    id: i64,
    payload: Value,
) -> ApiResult<()> {
    sqlx::query(
        "INSERT INTO core.audit_events (actor_id, action, entity_type, entity_id, payload)
        VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(actor)
    .bind(action)
    .bind(entity)
    .bind(id)
    .bind(payload)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn health(State(pool): State<PgPool>) -> ApiResult<Json<Value>> {
    sqlx::query("SELECT 1").execute(&pool).await?;
    Ok(Json(json!({"status": "ok"})))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(PgConnectOptions::new())
        .await?;
    sqlx::migrate!().run(&pool).await?;
    auth::seed(&pool).await?;
    let api = Router::new()
        .route("/health", get(health))
        .merge(auth::routes())
        .merge(engineering::routes())
        .merge(manufacturing::routes());
    let app = Router::new().nest("/api/v1", api).with_state(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("API disponible sur le port 3000");
    axum::serve(listener, app).await?;
    Ok(())
}
