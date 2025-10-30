use axum::{Router, extract::State, routing::get};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    compression::CompressionLayer,
    decompression::RequestDecompressionLayer,
    normalize_path::NormalizePathLayer,
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::info;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, registry, util::SubscriberInitExt};

mod bookmarks;

fn setup_tracing_subscriber() {
    registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn create_db_pool() -> Result<PgPool, sqlx::Error> {
    use std::env::var;

    let protocol = var("CONEXUS_DB_PROTOCOL").unwrap_or_else(|_| "postgresql".into());
    let host = var("CONEXUS_DB_HORT").unwrap_or_else(|_| "localhost".into());
    let port = var("CONEXUS_DB_PORT").unwrap_or_else(|_| "5432".into());
    let user = var("CONEXUS_DB_USER").expect("Database username");
    let password = var("CONEXUS_DB_PASSWORD").expect("Database password");
    let database = var("CONEXUS_DB_NAME").expect("Database name");

    PgPoolOptions::new()
        .connect(&format!(
            "{protocol}://{user}:{password}@{host}:{port}/{database}"
        ))
        .await
}

async fn health_check_handler(State(pool): State<PgPool>) -> String {
    let pg_version = sqlx::query_scalar!("SELECT version()")
        .fetch_one(&pool)
        .await
        .unwrap()
        .unwrap();
    format!("PosgreSQL version: {pg_version}")
}

fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check_handler))
        .nest("/bookmarks", bookmarks::create_router())
        .with_state(pool.clone())
        .layer(
            ServiceBuilder::new()
                .layer(NormalizePathLayer::trim_trailing_slash())
                .layer(RequestDecompressionLayer::new())
                .layer(CompressionLayer::new())
                .set_x_request_id(MakeRequestUuid::default())
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                )
                .propagate_x_request_id(),
        )
}

#[tokio::main]
async fn main() {
    setup_tracing_subscriber();

    let pool = create_db_pool().await.unwrap();
    let app = create_router(pool);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Starting server on port 3000...");
    axum::serve(listener, app).await.unwrap();
}
