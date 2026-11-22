use crate::{ApiError, ApiResult, audit, auth};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgPool;

#[derive(Serialize, sqlx::FromRow)]
struct Order {
    id: i64,
    number: String,
    description: String,
    quantity: i32,
    status: String,
    total_seconds: f64,
    part_number: Option<String>,
    revision: Option<String>,
}

#[derive(Deserialize)]
struct NewOrder {
    item_revision_id: i64,
    number: String,
    description: String,
    quantity: i32,
}

#[derive(Serialize, sqlx::FromRow)]
struct Session {
    id: i64,
    work_order_id: i64,
    operator_name: String,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
}

async fn orders(State(pool): State<PgPool>, headers: HeaderMap) -> ApiResult<Json<Vec<Order>>> {
    auth::user(&pool, &headers).await?;
    Ok(Json(sqlx::query_as("SELECT o.id, o.number, o.description, o.quantity, o.status, i.part_number, r.revision,
        COALESCE(SUM(EXTRACT(EPOCH FROM (s.ended_at - s.started_at))), 0)::float8 AS total_seconds
        FROM manufacturing.work_orders o LEFT JOIN manufacturing.work_sessions s ON s.work_order_id = o.id
        LEFT JOIN engineering.item_revisions r ON r.id = o.item_revision_id
        LEFT JOIN engineering.items i ON i.id = r.item_id
        GROUP BY o.id, i.part_number, r.revision ORDER BY o.id DESC").fetch_all(&pool).await?))
}

async fn create(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(input): Json<NewOrder>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let actor = auth::user(&pool, &headers).await?;
    actor.require("supervisor")?;
    let number = input.number.trim();
    let description = input.description.trim();
    if !(1..=80).contains(&number.chars().count())
        || !(1..=250).contains(&description.chars().count())
        || number.contains('\0')
        || description.contains('\0')
        || input.quantity <= 0
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Vérifiez le numéro, la description et la quantité.",
        ));
    }
    let mut tx = pool.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO manufacturing.work_orders (number, description, quantity, item_revision_id)
        VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(number)
    .bind(description)
    .bind(input.quantity)
    .bind(input.item_revision_id)
    .fetch_one(&mut *tx)
    .await?;
    audit(
        &mut tx,
        actor.id,
        "of.creation",
        "of",
        id,
        json!({"number": number, "item_revision_id": input.item_revision_id}),
    )
    .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"id": id}))))
}

async fn sessions(State(pool): State<PgPool>, headers: HeaderMap) -> ApiResult<Json<Vec<Session>>> {
    let actor = auth::user(&pool, &headers).await?;
    Ok(Json(sqlx::query_as("SELECT s.id, s.work_order_id, u.display_name AS operator_name, s.started_at, s.ended_at
        FROM manufacturing.work_sessions s JOIN core.users u ON u.id = s.operator_id
        WHERE $1 OR s.operator_id = $2 ORDER BY s.id DESC")
        .bind(actor.role == "supervisor").bind(actor.id).fetch_all(&pool).await?))
}

async fn start(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(order): Path<i64>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let actor = auth::user(&pool, &headers).await?;
    actor.require("operator")?;
    let mut tx = pool.begin().await?;
    let status: Option<String> =
        sqlx::query_scalar("SELECT status FROM manufacturing.work_orders WHERE id = $1 FOR SHARE")
            .bind(order)
            .fetch_optional(&mut *tx)
            .await?;
    match status.as_deref() {
        None => return Err(ApiError(StatusCode::NOT_FOUND, "OF introuvable.")),
        Some("open") => (),
        _ => {
            return Err(ApiError(
                StatusCode::CONFLICT,
                "Cet OF ne permet plus de pointer.",
            ));
        }
    }
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO manufacturing.work_sessions (operator_id, work_order_id)
        VALUES ($1, $2) RETURNING id",
    )
    .bind(actor.id)
    .bind(order)
    .fetch_one(&mut *tx)
    .await?;
    audit(
        &mut tx,
        actor.id,
        "pointage.debut",
        "pointage",
        id,
        json!({"work_order_id": order}),
    )
    .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"id": id}))))
}

async fn stop(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let actor = auth::user(&pool, &headers).await?;
    actor.require("operator")?;
    let mut tx = pool.begin().await?;
    let order: Option<i64> = sqlx::query_scalar(
        "UPDATE manufacturing.work_sessions
        SET ended_at = GREATEST(clock_timestamp(), started_at + interval '1 microsecond')
        WHERE id = $1 AND operator_id = $2 AND ended_at IS NULL RETURNING work_order_id",
    )
    .bind(id)
    .bind(actor.id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(order) = order {
        audit(
            &mut tx,
            actor.id,
            "pointage.fin",
            "pointage",
            id,
            json!({"work_order_id": order}),
        )
        .await?;
    } else {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM manufacturing.work_sessions WHERE id = $1 AND operator_id = $2)")
            .bind(id).bind(actor.id).fetch_one(&mut *tx).await?;
        if !exists {
            return Err(ApiError(StatusCode::NOT_FOUND, "Pointage introuvable."));
        }
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn events(State(pool): State<PgPool>, headers: HeaderMap) -> ApiResult<Json<Vec<Value>>> {
    auth::user(&pool, &headers).await?.require("supervisor")?;
    Ok(Json(sqlx::query_scalar("SELECT jsonb_build_object('id', a.id, 'actor', u.display_name, 'action', a.action,
        'entity_id', a.entity_id, 'occurred_at', a.occurred_at, 'payload', a.payload)
        FROM core.audit_events a JOIN core.users u ON u.id = a.actor_id ORDER BY a.id DESC LIMIT 200")
        .fetch_all(&pool).await?))
}

pub fn routes() -> Router<PgPool> {
    Router::new()
        .route("/orders", get(orders).post(create))
        .route("/orders/{id}/sessions", post(start))
        .route("/sessions", get(sessions))
        .route("/sessions/{id}/stop", post(stop))
        .route("/audit", get(events))
}
