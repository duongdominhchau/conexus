use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
pub struct Bookmark {
    id: Uuid,
    url: String,
    description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    updated_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewBookmark {
    url: String,
    description: Option<String>,
}

pub async fn list_bookmarks(State(pool): State<PgPool>) -> Json<Vec<Bookmark>> {
    let result = sqlx::query_as!(Bookmark, "select * from bookmarks")
        .fetch_all(&pool)
        .await
        .unwrap();
    Json(result)
}

pub async fn create_bookmark(
    State(pool): State<PgPool>,
    Json(record): Json<NewBookmark>,
) -> Json<Bookmark> {
    let result = sqlx::query_as!(
        Bookmark,
        "insert into bookmarks(id, url, description) values($1, $2, $3) returning *",
        Uuid::now_v7(),
        record.url,
        record.description
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    match result {
        Some(bookmark) => Json(bookmark),
        None => todo!(),
    }
}

pub async fn get_bookmark(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> Json<Bookmark> {
    let result = sqlx::query_as!(Bookmark, "select * from bookmarks where id = $1", id)
        .fetch_one(&pool)
        .await
        .unwrap();
    Json(result)
}

pub fn create_router() -> Router<PgPool> {
    Router::new()
        .route("/", get(list_bookmarks).post(create_bookmark))
        .route("/{id}", get(get_bookmark))
}
