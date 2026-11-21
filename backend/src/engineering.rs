use crate::{ApiError, ApiResult, audit, auth};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgPool;

#[derive(Serialize, sqlx::FromRow)]
struct Revision {
    id: i64,
    part_number: String,
    revision: String,
    description: String,
}

#[derive(Deserialize)]
struct NewRevision {
    part_number: String,
    revision: String,
    description: String,
}

async fn list(State(pool): State<PgPool>, headers: HeaderMap) -> ApiResult<Json<Vec<Revision>>> {
    auth::user(&pool, &headers).await?;
    Ok(Json(
        sqlx::query_as(
            "SELECT r.id, i.part_number, r.revision, r.description
             FROM engineering.item_revisions r JOIN engineering.items i ON i.id = r.item_id
             ORDER BY i.part_number, r.revision",
        )
        .fetch_all(&pool)
        .await?,
    ))
}

async fn create(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(input): Json<NewRevision>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let actor = auth::user(&pool, &headers).await?;
    actor.require("supervisor")?;
    let part_number = input.part_number.trim();
    let revision = input.revision.trim();
    let description = input.description.trim();
    if [(part_number, 80), (revision, 40), (description, 250)]
        .into_iter()
        .any(|(value, max)| !(1..=max).contains(&value.chars().count()) || value.contains('\0'))
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Vérifiez la référence article, la révision et la description.",
        ));
    }

    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO engineering.items (part_number) VALUES ($1)
         ON CONFLICT (part_number) DO NOTHING",
    )
    .bind(part_number)
    .execute(&mut *tx)
    .await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO engineering.item_revisions (item_id, revision, description)
         SELECT id, $2, $3 FROM engineering.items WHERE part_number = $1
         RETURNING id",
    )
    .bind(part_number)
    .bind(revision)
    .bind(description)
    .fetch_one(&mut *tx)
    .await?;
    audit(
        &mut tx,
        actor.id,
        "article.revision_creation",
        "revision_article",
        id,
        json!({"part_number": part_number, "revision": revision}),
    )
    .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(json!({"id": id}))))
}

pub fn routes() -> Router<PgPool> {
    Router::new().route("/item-revisions", get(list).post(create))
}
